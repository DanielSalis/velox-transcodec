mod encoders;
mod probe;
mod process;
mod progress;
mod transcode;

use std::path::PathBuf;

use crate::domain::{JobFailure, JobFailureKind};

/// Caminhos ou nomes no PATH dos executáveis utilizados pelo motor.
#[derive(Clone, Debug)]
pub struct MediaEngine {
    ffmpeg: PathBuf,
    ffprobe: PathBuf,
}

impl Default for MediaEngine {
    fn default() -> Self {
        Self::new("ffmpeg", "ffprobe")
    }
}

impl MediaEngine {
    pub fn new(ffmpeg: impl Into<PathBuf>, ffprobe: impl Into<PathBuf>) -> Self {
        Self {
            ffmpeg: ffmpeg.into(),
            ffprobe: ffprobe.into(),
        }
    }
}

pub(crate) use transcode::{JobError, VideoJob};

fn failure(kind: JobFailureKind, detail: impl ToString) -> JobFailure {
    JobFailure {
        kind,
        detail: detail.to_string(),
    }
}

async fn local_file(path: &std::path::Path) -> Result<(PathBuf, u64), JobFailure> {
    let path =
        std::path::absolute(path).map_err(|err| failure(JobFailureKind::InputUnavailable, err))?;
    let metadata = tokio::fs::metadata(&path)
        .await
        .map_err(|err| failure(JobFailureKind::InputUnavailable, err))?;
    if !metadata.is_file() {
        return Err(failure(
            JobFailureKind::InputUnavailable,
            "A entrada precisa ser um arquivo regular.",
        ));
    }
    Ok((path, metadata.len()))
}
