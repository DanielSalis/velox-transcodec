use std::{path::PathBuf, time::Duration};

use super::{Container, EncoderCapability, EncoderTarget, MediaId, MediaInfo, OutputSettings};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct BatchId(pub u64);

#[derive(Debug)]
pub enum QueueCommand {
    AddFiles {
        paths: Vec<PathBuf>,
    },
    DetectEncoders,
    StartBatch {
        items: Vec<MediaId>,
        settings: OutputSettings,
    },
    CancelBatch {
        batch_id: BatchId,
    },
}

#[derive(Debug)]
pub enum QueueEvent {
    ItemAdded {
        media_id: MediaId,
        path: PathBuf,
    },
    MediaProbed(MediaInfo),
    EncoderCapabilities(Vec<EncoderCapability>),
    EncoderDetectionFailed(JobFailure),
    BatchStarted {
        batch_id: BatchId,
        items: Vec<MediaId>,
    },
    ItemChanged {
        media_id: MediaId,
        state: ItemState,
    },
    ItemProgress {
        batch_id: BatchId,
        media_id: MediaId,
        progress: ProgressSnapshot,
    },
    BatchProgress {
        batch_id: BatchId,
        progress: BatchProgress,
    },
    BatchFinished(BatchSummary),
    Rejected(CommandError),
}

#[derive(Debug)]
pub enum ItemState {
    ProbeFailed(JobFailure),
    Queued {
        batch_id: BatchId,
    },
    Running {
        batch_id: BatchId,
    },
    Succeeded {
        batch_id: BatchId,
        output: JobOutput,
    },
    Failed {
        batch_id: BatchId,
        error: JobFailure,
    },
    Cancelled {
        batch_id: BatchId,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JobFailure {
    pub kind: JobFailureKind,
    pub detail: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum JobFailureKind {
    DependencyUnavailable,
    InputUnavailable,
    InvalidMedia,
    EncoderUnavailable,
    AudioCopyIncompatible,
    TranscodeFailed,
    OutputUnavailable,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CommandError {
    EmptySelection,
    BatchAlreadyRunning {
        batch_id: BatchId,
    },
    BatchNotRunning {
        batch_id: BatchId,
    },
    ItemNotReady {
        media_id: MediaId,
    },
    EncoderUnavailable {
        target: EncoderTarget,
    },
    UnsupportedEncoderSettings {
        target: EncoderTarget,
    },
    AudioCopyIncompatible {
        media_id: MediaId,
        container: Container,
    },
    DestinationUnavailable {
        path: PathBuf,
    },
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ProgressSnapshot {
    pub processed: Duration,
    pub fps: Option<f64>,
    pub speed: Option<f64>,
    /// Percentual no intervalo de 0 a 100, se a duração estiver disponível.
    pub percent: Option<f64>,
    pub eta: Option<Duration>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct BatchProgress {
    /// Percentual entre 0 e 100. Pode usar contagem se a duração for desconhecida.
    pub percent: f64,
    pub terminal_items: usize,
    pub total_items: usize,
    pub eta: Option<Duration>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct JobOutput {
    pub path: PathBuf,
    pub bytes: u64,
    /// Pode ser negativo quando o arquivo de saída for maior.
    pub reduction_percent: Option<f64>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BatchSummary {
    pub batch_id: BatchId,
    pub succeeded: usize,
    pub failed: usize,
    pub cancelled: usize,
}
