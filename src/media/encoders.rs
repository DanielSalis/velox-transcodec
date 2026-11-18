use std::{fs, path::PathBuf};
use tokio::process::Command;

use crate::domain::{
    EncoderCapability, EncoderOptions, EncoderTarget, EncoderUnavailable, H264Profile, JobFailure,
    JobFailureKind, NvencPreset, OutputCapabilities, ProResProfile, Quality, QualityMode,
    SpeedPreset, X264Preset,
};

use super::{MediaEngine, failure, process};

const PAD: &str = "pad=ceil(iw/2)*2:ceil(ih/2)*2";
const VAAPI_FILTER: &str = "pad=ceil(iw/2)*2:ceil(ih/2)*2,format=nv12,hwupload";
const PRORES_ENCODER: &str = "prores_ks";

impl MediaEngine {
    /// Ordem de preferência: nós VA-API, índices NVENC e x264. Apenas `support.is_ok()`
    /// identifica uma opção disponível para seleção.
    pub async fn detect_encoders(&self) -> Result<OutputCapabilities, JobFailure> {
        let listing = process::capture(
            process::command(&self.ffmpeg).args(["-hide_banner", "-encoders"]),
            10,
            256 * 1024,
            JobFailureKind::EncoderUnavailable,
        )
        .await?;
        if !listing.status.success() {
            return Err(failure(
                JobFailureKind::EncoderUnavailable,
                process::diagnostic_or_status(listing.status, listing.stderr),
            ));
        }
        let listing = String::from_utf8_lossy(&listing.stdout);
        let present = |name| {
            listing.lines().any(|line| {
                let mut fields = line.split_whitespace();
                fields.next().is_some_and(|flags| flags.starts_with('V'))
                    && fields.next() == Some(name)
            })
        };

        let mut candidates = render_nodes()
            .into_iter()
            .map(|render_node| EncoderTarget::Vaapi { render_node })
            .collect::<Vec<_>>();
        let nvenc = present("h264_nvenc");
        let gpu_indices = if nvenc {
            self.nvenc_indices().await
        } else {
            vec![0]
        };
        candidates.extend(
            gpu_indices
                .into_iter()
                .map(|gpu_index| EncoderTarget::Nvenc { gpu_index }),
        );
        candidates.push(EncoderTarget::X264);
        let mut capabilities = Vec::with_capacity(candidates.len());
        for target in candidates {
            let support = if present(encoder_name(&target)) {
                self.probe_encoder(&target).await
            } else {
                Err(EncoderUnavailable::MissingFromFfmpeg)
            };
            capabilities.push(EncoderCapability { target, support });
        }
        let prores = if present(PRORES_ENCODER) {
            self.probe_prores().await
        } else {
            Err(EncoderUnavailable::MissingFromFfmpeg)
        };
        Ok(OutputCapabilities {
            h264: capabilities,
            prores,
        })
    }

    async fn nvenc_indices(&self) -> Vec<u32> {
        let output = process::capture(
            process::command(&self.ffmpeg).args([
                "-hide_banner",
                "-loglevel",
                "info",
                "-nostdin",
                "-f",
                "lavfi",
                "-i",
                "color=s=640x360:r=24",
                "-frames:v",
                "1",
                "-c:v",
                "h264_nvenc",
                "-gpu",
                "list",
                "-f",
                "null",
                "-",
            ]),
            8,
            1024,
            JobFailureKind::EncoderUnavailable,
        )
        .await;
        // FFmpeg termina deliberadamente sem codificar quando -gpu list é solicitado.
        let mut indices: Vec<u32> = output
            .iter()
            .flat_map(|result| result.stderr.lines())
            .filter_map(|line| {
                let (_, suffix) = line.split_once("GPU #")?;
                suffix.split_whitespace().next()?.parse().ok()
            })
            .collect();
        indices.sort_unstable();
        indices.dedup();
        if indices.is_empty() {
            indices.push(0);
        }
        indices
    }

    async fn probe_encoder(
        &self,
        target: &EncoderTarget,
    ) -> Result<EncoderOptions, EncoderUnavailable> {
        use H264Profile::{Baseline, High, Main};
        let (quality_mode, candidates, default) = encoder_options(target);
        let mut profiles = Vec::new();
        let mut diagnostic = String::new();
        for profile in [High, Main, Baseline] {
            match self.smoke_encode(target, default, profile).await {
                Ok(()) => profiles.push(profile),
                Err(err) if diagnostic.is_empty() => diagnostic = err,
                Err(_) => (),
            }
        }
        let Some(&profile) = profiles.first() else {
            return Err(EncoderUnavailable::ProbeFailed { detail: diagnostic });
        };
        let mut presets = Vec::new();
        for &preset in candidates {
            if preset == default || self.smoke_encode(target, preset, profile).await.is_ok() {
                presets.push(preset);
            }
        }
        Ok(EncoderOptions {
            quality_mode,
            presets,
            profiles,
        })
    }

    async fn smoke_encode(
        &self,
        target: &EncoderTarget,
        preset: SpeedPreset,
        profile: H264Profile,
    ) -> Result<(), String> {
        let mut command = self.smoke_command();
        add_global_video_args(&mut command, target);
        add_smoke_source(&mut command);
        add_video_args(&mut command, target, preset, profile, Quality::DEFAULT, 1);
        finish_smoke_encode(command).await
    }

    async fn probe_prores(&self) -> Result<Vec<ProResProfile>, EncoderUnavailable> {
        let mut profiles = Vec::new();
        let mut diagnostic = None;
        for profile in ProResProfile::ALL {
            let mut command = self.smoke_command();
            add_smoke_source(&mut command);
            add_prores_args(&mut command, profile, 1);
            match finish_smoke_encode(command).await {
                Ok(()) => profiles.push(profile),
                Err(error) => {
                    diagnostic.get_or_insert(error);
                }
            }
        }
        if profiles.is_empty() {
            Err(EncoderUnavailable::ProbeFailed {
                detail: diagnostic.unwrap_or_default(),
            })
        } else {
            Ok(profiles)
        }
    }

    fn smoke_command(&self) -> Command {
        let mut command = process::command(&self.ffmpeg);
        command.args([
            "-hide_banner",
            "-v",
            "error",
            "-nostdin",
            "-filter_threads",
            "1",
        ]);
        command
    }
}

fn add_smoke_source(command: &mut Command) {
    command.args([
        "-f",
        "lavfi",
        "-i",
        "color=s=640x360:r=24",
        "-frames:v",
        "4",
    ]);
}

async fn finish_smoke_encode(mut command: Command) -> Result<(), String> {
    command.args(["-f", "null", "-"]);
    let output = process::capture(&mut command, 8, 1024, JobFailureKind::EncoderUnavailable)
        .await
        .map_err(|error| error.detail)?;
    if output.status.success() {
        Ok(())
    } else {
        Err(process::diagnostic_or_status(output.status, output.stderr))
    }
}

fn render_nodes() -> Vec<PathBuf> {
    let mut nodes: Vec<PathBuf> = fs::read_dir("/dev/dri")
        .into_iter()
        .flatten()
        .filter_map(Result::ok)
        .filter(|entry| {
            entry
                .file_name()
                .to_str()
                .and_then(|name| name.strip_prefix("renderD"))
                .is_some_and(|id| id.parse::<u32>().is_ok())
        })
        .map(|entry| entry.path())
        .collect();
    nodes.sort();
    nodes
}

fn encoder_options(target: &EncoderTarget) -> (QualityMode, &'static [SpeedPreset], SpeedPreset) {
    use SpeedPreset::{Automatic, Nvenc, X264};
    match target {
        EncoderTarget::Vaapi { .. } => (QualityMode::ConstantQuantizer, &[Automatic], Automatic),
        EncoderTarget::Nvenc { .. } => (
            QualityMode::ConstantQuality,
            &[
                Nvenc(NvencPreset::P1),
                Nvenc(NvencPreset::P2),
                Nvenc(NvencPreset::P3),
                Nvenc(NvencPreset::P4),
                Nvenc(NvencPreset::P5),
                Nvenc(NvencPreset::P6),
                Nvenc(NvencPreset::P7),
            ],
            Nvenc(NvencPreset::P4),
        ),
        EncoderTarget::X264 => (
            QualityMode::Crf,
            &[
                X264(X264Preset::Ultrafast),
                X264(X264Preset::Superfast),
                X264(X264Preset::Veryfast),
                X264(X264Preset::Faster),
                X264(X264Preset::Fast),
                X264(X264Preset::Medium),
                X264(X264Preset::Slow),
                X264(X264Preset::Slower),
                X264(X264Preset::Veryslow),
            ],
            X264(X264Preset::Medium),
        ),
    }
}

fn encoder_name(target: &EncoderTarget) -> &'static str {
    match target {
        EncoderTarget::Vaapi { .. } => "h264_vaapi",
        EncoderTarget::Nvenc { .. } => "h264_nvenc",
        EncoderTarget::X264 => "libx264",
    }
}

fn preset_name(preset: SpeedPreset) -> Option<&'static str> {
    Some(match preset {
        SpeedPreset::Automatic => return None,
        SpeedPreset::Nvenc(preset) => match preset {
            NvencPreset::P1 => "p1",
            NvencPreset::P2 => "p2",
            NvencPreset::P3 => "p3",
            NvencPreset::P4 => "p4",
            NvencPreset::P5 => "p5",
            NvencPreset::P6 => "p6",
            NvencPreset::P7 => "p7",
        },
        SpeedPreset::X264(preset) => match preset {
            X264Preset::Ultrafast => "ultrafast",
            X264Preset::Superfast => "superfast",
            X264Preset::Veryfast => "veryfast",
            X264Preset::Faster => "faster",
            X264Preset::Fast => "fast",
            X264Preset::Medium => "medium",
            X264Preset::Slow => "slow",
            X264Preset::Slower => "slower",
            X264Preset::Veryslow => "veryslow",
        },
    })
}

pub(super) fn add_global_video_args(command: &mut Command, target: &EncoderTarget) {
    if let EncoderTarget::Vaapi { render_node } = target {
        command.arg("-vaapi_device").arg(render_node);
    }
}

pub(super) fn add_prores_args(command: &mut Command, profile: ProResProfile, cpu_threads: usize) {
    let profile = match profile {
        ProResProfile::Proxy => "proxy",
        ProResProfile::Lt => "lt",
        ProResProfile::Standard => "standard",
        ProResProfile::Hq => "hq",
    };
    command.args([
        "-c:v",
        PRORES_ENCODER,
        "-profile:v",
        profile,
        "-vf",
        "pad=ceil(iw/2)*2:ih",
        "-pix_fmt",
        "yuv422p10le",
        "-threads:v",
        &cpu_threads.to_string(),
    ]);
}

pub(super) fn add_video_args(
    command: &mut Command,
    target: &EncoderTarget,
    preset: SpeedPreset,
    profile: H264Profile,
    quality: Quality,
    cpu_threads: usize,
) {
    let profile = match profile {
        H264Profile::Baseline if matches!(target, EncoderTarget::Vaapi { .. }) => {
            "constrained_baseline"
        }
        H264Profile::Baseline => "baseline",
        H264Profile::Main => "main",
        H264Profile::High => "high",
    };
    let quality = quality.value().to_string();
    command.args(["-c:v", encoder_name(target), "-profile:v", profile]);
    if let Some(preset) = preset_name(preset) {
        command.args(["-preset", preset]);
    }
    match target {
        EncoderTarget::Vaapi { .. } => {
            command.args(["-vf", VAAPI_FILTER, "-rc_mode", "CQP", "-qp", &quality]);
        }
        EncoderTarget::Nvenc { gpu_index } => {
            command.args([
                "-vf",
                PAD,
                "-pix_fmt",
                "yuv420p",
                "-gpu",
                &gpu_index.to_string(),
                "-rc",
                "vbr",
                "-cq",
                &quality,
                "-b:v",
                "0",
            ]);
            if profile == "baseline" {
                command.args(["-bf", "0"]);
            }
        }
        EncoderTarget::X264 => {
            command.args([
                "-vf",
                PAD,
                "-pix_fmt",
                "yuv420p",
                "-crf",
                &quality,
                "-threads:v",
                &cpu_threads.to_string(),
            ]);
        }
    }
}
