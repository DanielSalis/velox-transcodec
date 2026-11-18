use std::{
    ffi::OsString,
    io,
    path::{Path, PathBuf},
    sync::Arc,
};

use tempfile::TempPath;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio_util::sync::CancellationToken;

use crate::domain::{
    AudioMode, Container, JobFailure, JobFailureKind, JobOutput, MediaInfo, OutputSettings,
    ProgressSnapshot,
};

use super::{MediaEngine, encoders, failure, local_file, process, progress::Parser};

pub(crate) struct VideoJob {
    pub media: MediaInfo,
    pub settings: Arc<OutputSettings>,
    pub cpu_threads: usize,
}

pub(crate) enum JobError {
    Failed(JobFailure),
    Cancelled,
}

impl From<JobFailure> for JobError {
    fn from(error: JobFailure) -> Self {
        Self::Failed(error)
    }
}

impl MediaEngine {
    pub(crate) async fn transcode(
        &self,
        job: VideoJob,
        cancellation: CancellationToken,
        mut on_progress: impl FnMut(ProgressSnapshot),
    ) -> Result<JobOutput, JobError> {
        if cancellation.is_cancelled() {
            return Err(JobError::Cancelled);
        }
        let VideoJob {
            media,
            settings,
            cpu_threads,
        } = job;
        let (input, source_bytes) = local_file(&media.path).await?;
        let (format, suffix) = container(settings.container);
        let temp = tempfile::Builder::new()
            .prefix(".velox-")
            .suffix(suffix)
            .tempfile_in(&settings.destination)
            .map_err(output_error)?
            .into_temp_path();

        let mut command = process::command(&self.ffmpeg);
        command.args([
            "-hide_banner",
            "-v",
            "error",
            "-nostdin",
            "-nostats",
            "-xerror",
            "-abort_on",
            "empty_output_stream",
            "-progress",
            "pipe:1",
            "-stats_period",
            "0.25",
            "-y",
            "-filter_threads",
            "2",
        ]);
        encoders::add_global_video_args(&mut command, &settings.encoder);
        command
            .args(["-threads", "2", "-protocol_whitelist", "file", "-i"])
            .arg(&input);
        command.args(["-map", &format!("0:{}", media.video.index), "-map", "0:a?"]);
        encoders::add_video_args(
            &mut command,
            &settings.encoder,
            settings.preset,
            settings.profile,
            settings.quality,
            cpu_threads,
        );
        match settings.audio {
            AudioMode::Copy => command.args(["-c:a", "copy"]),
            AudioMode::Aac192 => command.args(["-c:a", "aac", "-b:a", "192k"]),
            AudioMode::Aac320 => command.args(["-c:a", "aac", "-b:a", "320k"]),
        };
        if matches!(settings.container, Container::Mp4 | Container::Mov) {
            command.args(["-movflags", "+faststart"]);
        }
        command.args(["-f", format]).arg(&temp);

        let mut child = command.spawn().map_err(process::spawn_failure)?;
        let stdout = child.stdout.take().expect("stdout de progresso");
        let stderr = child.stderr.take().expect("stderr do FFmpeg");
        let progress = async {
            let mut parser = Parser::default();
            let mut lines = BufReader::new(stdout).lines();
            while let Some(line) = lines.next_line().await? {
                if let Some(snapshot) = parser.line(&line, media.duration) {
                    on_progress(snapshot);
                }
            }
            Ok(())
        };
        let completed = tokio::select! {
            biased;
            () = cancellation.cancelled() => {
                let _ = child.kill().await;
                return Err(JobError::Cancelled);
            }
            result = async { tokio::try_join!(child.wait(), progress, process::diagnostic(stderr)) } => result,
        };
        let (status, (), stderr) = match completed {
            Ok(result) => result,
            Err(err) => {
                let _ = child.kill().await;
                return Err(failure(JobFailureKind::TranscodeFailed, err).into());
            }
        };
        if !status.success() {
            let kind = if settings.audio == AudioMode::Copy && incompatible_audio(&stderr) {
                JobFailureKind::AudioCopyIncompatible
            } else {
                JobFailureKind::TranscodeFailed
            };
            return Err(failure(kind, process::diagnostic_or_status(status, stderr)).into());
        }

        let bytes = tokio::fs::metadata(&temp)
            .await
            .map_err(output_error)?
            .len();
        if bytes == 0 {
            return Err(failure(
                JobFailureKind::TranscodeFailed,
                "O FFmpeg produziu um arquivo vazio.",
            )
            .into());
        }
        if cancellation.is_cancelled() {
            return Err(JobError::Cancelled);
        }
        let path = publish(temp, &settings.destination, &input, suffix).map_err(output_error)?;
        let reduction_percent =
            (source_bytes != 0).then(|| 100.0 * (1.0 - bytes as f64 / source_bytes as f64));
        Ok(JobOutput {
            path,
            bytes,
            reduction_percent,
        })
    }
}

fn incompatible_audio(stderr: &str) -> bool {
    stderr.lines().any(|line| {
        line.contains("Could not find tag for codec")
            || line.contains("is not supported in container")
            || line.contains("is not currently supported in container")
    })
}

fn output_error(error: io::Error) -> JobFailure {
    failure(JobFailureKind::OutputUnavailable, error)
}

fn container(container: Container) -> (&'static str, &'static str) {
    match container {
        Container::Mp4 => ("mp4", ".mp4"),
        Container::Mkv => ("matroska", ".mkv"),
        Container::Mov => ("mov", ".mov"),
    }
}

fn publish(
    mut temp: TempPath,
    destination: &Path,
    input: &Path,
    suffix: &str,
) -> io::Result<PathBuf> {
    let stem = input
        .file_stem()
        .unwrap_or_else(|| std::ffi::OsStr::new("video"));
    for index in 1_u32.. {
        let mut name = OsString::from(stem);
        name.push("-h264");
        if index > 1 {
            name.push(format!("-{index}"));
        }
        name.push(suffix);
        let path = destination.join(name);
        match temp.persist_noclobber(&path) {
            Ok(()) => return Ok(path),
            Err(error) if error.error.kind() == io::ErrorKind::AlreadyExists => temp = error.path,
            Err(error) => return Err(error.error),
        }
    }
    unreachable!("sequência de nomes da saída")
}
