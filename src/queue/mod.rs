mod batch;

use std::{
    collections::{HashMap, HashSet, VecDeque},
    path::PathBuf,
};

use futures_util::{StreamExt, stream::FuturesUnordered};
use tokio::{sync::mpsc, task::JoinHandle};
use tokio_util::sync::CancellationToken;

use crate::{
    domain::{
        BatchId, BatchSummary, CommandError, ItemState, JobFailure, JobFailureKind, MediaId,
        MediaInfo, OutputCapabilities, OutputSettings, QueueCommand, QueueEvent, VideoOutput,
    },
    media::MediaEngine,
};

const PROBE_CONCURRENCY: usize = 4;

/// A interface deve consumir `events` continuamente. Progresso intermediário pode ser
/// coalescido sob carga. Estados terminais são enviados com backpressure.
pub struct Queue {
    pub commands: mpsc::Sender<QueueCommand>,
    pub events: mpsc::Receiver<QueueEvent>,
}

/// Inicia a fila no runtime Tokio corrente. Fechar o receiver encerra o serviço e
/// cancela a execução ativa. Fechar o sender também cancela e permite drenar os eventos finais.
pub fn spawn(engine: MediaEngine) -> Queue {
    let (commands, inbox) = mpsc::channel(32);
    let (outbox, events) = mpsc::channel(128);
    tokio::spawn(Service::new(engine, outbox).run(inbox));
    Queue { commands, events }
}

struct ActiveBatch {
    id: BatchId,
    cancellation: CancellationToken,
    task: JoinHandle<BatchSummary>,
}

struct Service {
    engine: MediaEngine,
    outbox: mpsc::Sender<QueueEvent>,
    media: HashMap<MediaId, MediaInfo>,
    next_media: u64,
    next_batch: u64,
    probe_backlog: VecDeque<(MediaId, PathBuf)>,
    capabilities: Option<OutputCapabilities>,
    wants_detection: bool,
    active: Option<ActiveBatch>,
}

impl Service {
    fn new(engine: MediaEngine, outbox: mpsc::Sender<QueueEvent>) -> Self {
        Self {
            engine,
            outbox,
            media: HashMap::new(),
            next_media: 1,
            next_batch: 1,
            probe_backlog: VecDeque::new(),
            capabilities: None,
            wants_detection: false,
            active: None,
        }
    }

    async fn run(mut self, mut inbox: mpsc::Receiver<QueueCommand>) {
        let mut probes = FuturesUnordered::new();
        let mut detection = None;
        loop {
            while probes.len() < PROBE_CONCURRENCY {
                let Some((id, path)) = self.probe_backlog.pop_front() else {
                    break;
                };
                let engine = self.engine.clone();
                probes.push(async move { (id, engine.probe(id, &path).await) });
            }
            if self.wants_detection && detection.is_none() && self.active.is_none() {
                self.wants_detection = false;
                self.capabilities = None;
                let engine = self.engine.clone();
                detection = Some(tokio::spawn(async move { engine.detect_encoders().await }));
            }

            tokio::select! {
                () = self.outbox.closed() => break,
                command = inbox.recv() => {
                    let Some(command) = command else { break };
                    self.handle(command).await;
                }
                Some((id, result)) = probes.next() => {
                    let event = match result {
                        Ok(info) => {
                            self.media.insert(id, info.clone());
                            QueueEvent::MediaProbed(info)
                        }
                        Err(error) => QueueEvent::ItemChanged {
                            media_id: id,
                            state: ItemState::ProbeFailed(error),
                        },
                    };
                    let _ = self.outbox.send(event).await;
                }
                result = async { detection.as_mut().expect("sondagem ativa").await }, if detection.is_some() => {
                    detection = None;
                    let result = result.map_err(|error| JobFailure {
                        kind: JobFailureKind::EncoderUnavailable,
                        detail: error.to_string(),
                    }).and_then(|result| result);
                    let event = match result {
                        Ok(capabilities) => {
                            self.capabilities = Some(capabilities.clone());
                            QueueEvent::EncoderCapabilities(capabilities)
                        }
                        Err(error) => QueueEvent::EncoderDetectionFailed(error),
                    };
                    let _ = self.outbox.send(event).await;
                }
                result = async { (&mut self.active.as_mut().expect("lote ativo").task).await }, if self.active.is_some() => {
                    self.active = None;
                    if let Ok(summary) = result {
                        let _ = self.outbox.send(QueueEvent::BatchFinished(summary)).await;
                    }
                }
            }
        }

        drop(probes);
        if let Some(detection) = detection {
            detection.abort();
            let _ = detection.await;
        }
        if let Some(active) = self.active.take() {
            active.cancellation.cancel();
            if let Ok(summary) = active.task.await {
                let _ = self.outbox.send(QueueEvent::BatchFinished(summary)).await;
            }
        }
    }

    async fn handle(&mut self, command: QueueCommand) {
        match command {
            QueueCommand::AddFiles { paths } => {
                for path in paths {
                    let media_id = MediaId(self.next_media);
                    self.next_media += 1;
                    let event = QueueEvent::ItemAdded {
                        media_id,
                        path: path.clone(),
                    };
                    if self.outbox.send(event).await.is_err() {
                        break;
                    }
                    self.probe_backlog.push_back((media_id, path));
                }
            }
            QueueCommand::DetectEncoders => self.wants_detection = true,
            QueueCommand::StartBatch { items, settings } => {
                match self.start(items, settings).await {
                    Ok(active) => self.active = Some(active),
                    Err(error) => {
                        let _ = self.outbox.send(QueueEvent::Rejected(error)).await;
                    }
                }
            }
            QueueCommand::CancelBatch { batch_id } => {
                if let Some(active) = self.active.as_ref().filter(|active| active.id == batch_id) {
                    active.cancellation.cancel();
                } else {
                    let _ = self
                        .outbox
                        .send(QueueEvent::Rejected(CommandError::BatchNotRunning {
                            batch_id,
                        }))
                        .await;
                }
            }
        }
    }

    async fn start(
        &mut self,
        items: Vec<MediaId>,
        mut settings: OutputSettings,
    ) -> Result<ActiveBatch, CommandError> {
        if let Some(active) = &self.active {
            return Err(CommandError::BatchAlreadyRunning {
                batch_id: active.id,
            });
        }
        if items.is_empty() {
            return Err(CommandError::EmptySelection);
        }
        let mut seen = HashSet::new();
        let media = items
            .into_iter()
            .filter(|id| seen.insert(*id))
            .map(|id| {
                self.media
                    .get(&id)
                    .cloned()
                    .ok_or(CommandError::ItemNotReady { media_id: id })
            })
            .collect::<Result<Vec<_>, _>>()?;
        self.validate_output(&settings.video)?;
        let destination = &settings.destination;
        settings.destination = tokio::fs::canonicalize(destination)
            .await
            .ok()
            .filter(|path| path.is_dir())
            .ok_or_else(|| CommandError::DestinationUnavailable {
                path: destination.clone(),
            })?;
        let id = BatchId(self.next_batch);
        self.next_batch += 1;
        let cancellation = CancellationToken::new();
        let task = tokio::spawn(batch::run(
            id,
            media,
            settings,
            self.engine.clone(),
            cancellation.clone(),
            self.outbox.clone(),
        ));
        Ok(ActiveBatch {
            id,
            cancellation,
            task,
        })
    }

    fn validate_output(&self, video: &VideoOutput) -> Result<(), CommandError> {
        match video {
            VideoOutput::H264(settings) => {
                let target = &settings.encoder;
                let options = self
                    .capabilities
                    .as_ref()
                    .and_then(|caps| caps.h264.iter().find(|cap| &cap.target == target))
                    .and_then(|cap| cap.support.as_ref().ok())
                    .ok_or_else(|| CommandError::EncoderUnavailable {
                        target: target.clone(),
                    })?;
                if !options.presets.contains(&settings.preset)
                    || !options.profiles.contains(&settings.profile)
                {
                    return Err(CommandError::UnsupportedEncoderSettings {
                        target: target.clone(),
                    });
                }
            }
            VideoOutput::ProRes { profile, .. } => {
                let profiles = self
                    .capabilities
                    .as_ref()
                    .and_then(|caps| caps.prores.as_ref().ok())
                    .ok_or(CommandError::ProResUnavailable)?;
                if !profiles.contains(profile) {
                    return Err(CommandError::UnsupportedProResProfile);
                }
            }
        }
        Ok(())
    }
}
