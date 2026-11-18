use std::{
    collections::{HashMap, VecDeque},
    sync::Arc,
    time::Duration,
};

use futures_util::{StreamExt, stream::FuturesUnordered};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use crate::{
    domain::{
        BatchId, BatchProgress, BatchSummary, EncoderTarget, H264Settings, ItemState, MediaId,
        MediaInfo, OutputSettings, ProgressSnapshot, QueueEvent, VideoOutput,
    },
    media::{JobError, MediaEngine, VideoJob},
};

struct Row {
    duration: Option<Duration>,
    progress: Option<ProgressSnapshot>,
    terminal: bool,
}

pub(super) async fn run(
    batch_id: BatchId,
    media: Vec<MediaInfo>,
    settings: OutputSettings,
    engine: MediaEngine,
    cancellation: CancellationToken,
    outbox: mpsc::Sender<QueueEvent>,
) -> BatchSummary {
    let mut summary = BatchSummary {
        batch_id,
        succeeded: 0,
        failed: 0,
        cancelled: 0,
    };
    let mut rows: HashMap<_, _> = media
        .iter()
        .map(|item| {
            (
                item.id,
                Row {
                    duration: item.duration,
                    progress: None,
                    terminal: false,
                },
            )
        })
        .collect();
    let _ = outbox
        .send(QueueEvent::BatchStarted {
            batch_id,
            items: media.iter().map(|media| media.id).collect(),
        })
        .await;
    for item in &media {
        item_changed(&outbox, item.id, ItemState::Queued { batch_id }).await;
    }
    let _ = outbox
        .send(QueueEvent::BatchProgress {
            batch_id,
            progress: aggregate(&rows),
        })
        .await;

    let cpus = std::thread::available_parallelism().map_or(1, |count| count.get());
    let concurrency = if matches!(
        settings.video,
        VideoOutput::H264(H264Settings {
            encoder: EncoderTarget::X264,
            ..
        }) | VideoOutput::ProRes { .. }
    ) {
        cpus.min(2)
    } else {
        1
    };
    let cpu_threads = (cpus / concurrency).clamp(1, 8);
    let settings = Arc::new(settings);
    let mut pending = VecDeque::from(media);
    let mut running = FuturesUnordered::new();
    let (progress_tx, mut progress_rx) = mpsc::channel(16);
    let mut cancelling = false;

    loop {
        if cancellation.is_cancelled() {
            cancelling = true;
            for item in pending.drain(..) {
                summary.cancelled += 1;
                rows.get_mut(&item.id).expect("linha pendente").terminal = true;
                item_changed(&outbox, item.id, ItemState::Cancelled { batch_id }).await;
            }
        }

        while !cancelling && running.len() < concurrency {
            let Some(item) = pending.pop_front() else {
                break;
            };
            let media_id = item.id;
            item_changed(&outbox, media_id, ItemState::Running { batch_id }).await;
            let job = VideoJob {
                media: item,
                settings: Arc::clone(&settings),
                cpu_threads,
            };
            let engine = engine.clone();
            let cancellation = cancellation.clone();
            let progress_tx = progress_tx.clone();
            running.push(async move {
                let result = engine
                    .transcode(job, cancellation, |progress| {
                        let _ = progress_tx.try_send((media_id, progress));
                    })
                    .await;
                (media_id, result)
            });
        }
        if running.is_empty() {
            break;
        }

        tokio::select! {
            biased;
            () = cancellation.cancelled(), if !cancelling => (),
            Some((media_id, progress)) = progress_rx.recv() => {
                publish_progress(&outbox, batch_id, media_id, progress, &mut rows);
            }
            Some((media_id, result)) = running.next() => {
                // Mesmo um encode curto pode concluir no mesmo poll em que publica o progresso.
                while let Ok((id, progress)) = progress_rx.try_recv() {
                    publish_progress(&outbox, batch_id, id, progress, &mut rows);
                }
                rows.get_mut(&media_id).expect("linha em execução").terminal = true;
                let state = match result {
                    Ok(output) => {
                        summary.succeeded += 1;
                        ItemState::Succeeded { batch_id, output }
                    }
                    Err(JobError::Failed(error)) => {
                        summary.failed += 1;
                        ItemState::Failed { batch_id, error }
                    }
                    Err(JobError::Cancelled) => {
                        summary.cancelled += 1;
                        ItemState::Cancelled { batch_id }
                    }
                };
                item_changed(&outbox, media_id, state).await;
                let _ = outbox.try_send(QueueEvent::BatchProgress { batch_id, progress: aggregate(&rows) });
            }
        }
    }

    let _ = outbox
        .send(QueueEvent::BatchProgress {
            batch_id,
            progress: aggregate(&rows),
        })
        .await;
    summary
}

async fn item_changed(outbox: &mpsc::Sender<QueueEvent>, media_id: MediaId, state: ItemState) {
    let _ = outbox
        .send(QueueEvent::ItemChanged { media_id, state })
        .await;
}

fn publish_progress(
    outbox: &mpsc::Sender<QueueEvent>,
    batch_id: BatchId,
    media_id: MediaId,
    progress: ProgressSnapshot,
    rows: &mut HashMap<MediaId, Row>,
) {
    let row = rows.get_mut(&media_id).expect("linha do progresso");
    if !row.terminal {
        row.progress = Some(progress);
        let _ = outbox.try_send(QueueEvent::ItemProgress {
            batch_id,
            media_id,
            progress,
        });
        let _ = outbox.try_send(QueueEvent::BatchProgress {
            batch_id,
            progress: aggregate(rows),
        });
    }
}

fn aggregate(rows: &HashMap<MediaId, Row>) -> BatchProgress {
    let terminal_items = rows.values().filter(|row| row.terminal).count();
    let total_items = rows.len();
    let percent = if terminal_items == total_items {
        100.0
    } else if rows.values().all(|row| row.duration.is_some()) {
        let (processed, total) = rows.values().fold((0.0, 0.0), |(processed, total), row| {
            let weight = row.duration.expect("duração conhecida").as_secs_f64();
            let fraction = if row.terminal {
                1.0
            } else {
                row.progress
                    .and_then(|progress| progress.percent)
                    .unwrap_or(0.0)
                    / 100.0
            };
            (processed + weight * fraction, total + weight)
        });
        (100.0 * processed / total).clamp(0.0, 100.0)
    } else {
        100.0 * terminal_items as f64 / total_items as f64
    };
    // ETA só quando cada item restante já tem uma amostra; itens pendentes não têm velocidade.
    let eta = rows
        .values()
        .filter(|row| !row.terminal)
        .try_fold(Duration::ZERO, |eta, row| Some(eta.max(row.progress?.eta?)));
    BatchProgress {
        percent,
        terminal_items,
        total_items,
        eta,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn weighs_known_durations_and_counts_items_when_duration_is_missing() {
        let mut rows = HashMap::from([
            (
                MediaId(1),
                Row {
                    duration: Some(Duration::from_secs(1)),
                    progress: None,
                    terminal: true,
                },
            ),
            (
                MediaId(2),
                Row {
                    duration: Some(Duration::from_secs(3)),
                    progress: Some(ProgressSnapshot {
                        percent: Some(50.0),
                        eta: Some(Duration::from_secs(2)),
                        ..ProgressSnapshot::default()
                    }),
                    terminal: false,
                },
            ),
        ]);
        let known = aggregate(&rows);
        assert_eq!(known.percent, 62.5);
        assert_eq!(known.eta, Some(Duration::from_secs(2)));

        let unknown = rows.get_mut(&MediaId(2)).unwrap();
        unknown.duration = None;
        unknown.progress = None;
        let counted = aggregate(&rows);
        assert_eq!(counted.percent, 50.0);
        assert_eq!(counted.eta, None);
    }
}
