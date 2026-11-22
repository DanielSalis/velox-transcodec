use std::{
    io,
    process::{ExitStatus, Stdio},
    time::Duration,
};
use tokio::{
    io::{AsyncRead, AsyncReadExt},
    process::Command,
};

use super::failure;
use crate::domain::{JobFailure, JobFailureKind};

const DIAGNOSTIC_LIMIT: usize = 16 * 1024;

pub(super) fn command(executable: &std::path::Path) -> Command {
    let mut command = Command::new(executable);
    #[cfg(target_os = "windows")]
    command.creation_flags(0x08000000); // CREATE_NO_WINDOW
    command
        .env("LC_ALL", "C")
        .env("AV_LOG_FORCE_NOCOLOR", "1")
        .kill_on_drop(true)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    command
}

pub(super) struct Captured {
    pub status: ExitStatus,
    pub stdout: Vec<u8>,
    pub stderr: String,
}

/// Drena a saída inteira sem reter um volume ilimitado de dados.
async fn bounded(mut stream: impl AsyncRead + Unpin, limit: usize) -> io::Result<Vec<u8>> {
    let mut bytes = Vec::new();
    (&mut stream)
        .take(limit as u64 + 1)
        .read_to_end(&mut bytes)
        .await?;
    tokio::io::copy(&mut stream, &mut tokio::io::sink()).await?;
    Ok(bytes)
}

pub(super) async fn diagnostic(stderr: impl AsyncRead + Unpin) -> io::Result<String> {
    let mut bytes = bounded(stderr, DIAGNOSTIC_LIMIT).await?;
    let truncated = bytes.len() > DIAGNOSTIC_LIMIT;
    bytes.truncate(DIAGNOSTIC_LIMIT);
    let mut text = String::from_utf8_lossy(&bytes).trim().to_owned();
    if truncated {
        text.push_str("\n[diagnóstico truncado]");
    }
    Ok(text)
}

pub(super) fn diagnostic_or_status(status: ExitStatus, stderr: String) -> String {
    if stderr.is_empty() {
        status.to_string()
    } else {
        stderr
    }
}

pub(super) fn spawn_failure(error: io::Error) -> JobFailure {
    failure(JobFailureKind::DependencyUnavailable, error)
}

pub(super) async fn capture(
    command: &mut Command,
    seconds: u64,
    stdout_limit: usize,
    kind: JobFailureKind,
) -> Result<Captured, JobFailure> {
    let mut child = command.spawn().map_err(spawn_failure)?;
    let stdout = child.stdout.take().expect("stdout do FFmpeg");
    let stderr = child.stderr.take().expect("stderr do FFmpeg");
    let completed = tokio::time::timeout(Duration::from_secs(seconds), async {
        tokio::try_join!(
            child.wait(),
            bounded(stdout, stdout_limit),
            diagnostic(stderr)
        )
    })
    .await;
    let (status, stdout, stderr) = match completed {
        Ok(Ok(result)) => result,
        Ok(Err(err)) => {
            let _ = child.kill().await;
            return Err(failure(kind, err));
        }
        Err(_) => {
            let _ = child.kill().await;
            return Err(failure(kind, format!("O processo excedeu {seconds} s.")));
        }
    };
    if stdout.len() > stdout_limit {
        return Err(failure(
            kind,
            "A saída do processo excedeu o limite de leitura.",
        ));
    }
    Ok(Captured {
        status,
        stdout,
        stderr,
    })
}
