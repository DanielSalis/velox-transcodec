use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    time::{Duration, Instant},
};

use tempfile::TempDir;
use velox_transcode::domain::OutputSettings;
use velox_transcode::{
    domain::{
        AudioMode, BatchSummary, CommandError, Container, EncoderTarget, EncoderUnavailable,
        H264Profile, H264Settings, ItemState, JobFailureKind, JobOutput, MediaId,
        OutputCapabilities, ProResAudio, ProResProfile, Quality, QueueCommand, QueueEvent,
        SpeedPreset, VideoOutput, X264Preset,
    },
    media::MediaEngine,
    queue::{self, Queue},
};

/// Estes testes de integração usam ffmpeg/ffprobe reais com os codecs do ambiente Fedora.
#[tokio::test]
#[ignore = "requer ffmpeg, ffprobe e libx264 no PATH"]
async fn transcodes_batches_and_preserves_sources_and_existing_outputs() {
    let dir = TempDir::new().unwrap();
    let input = dir.path().join("clip com espaço.mp4");
    let other_dir = dir.path().join("outro");
    let output = dir.path().join("destino");
    fs::create_dir(&other_dir).unwrap();
    fs::create_dir(&output).unwrap();

    fixture(&input, "640x360", "2", Some("aac"));
    let second = other_dir.join(input.file_name().unwrap());
    fs::copy(&input, &second).unwrap();
    let invalid = dir.path().join("inválido.mkv");
    fs::write(&invalid, "arquivo sem vídeo").unwrap();
    let existing = output.join("clip com espaço-h264.mp4");
    fs::write(&existing, "arquivo anterior").unwrap();

    let mut queue = queue::spawn(MediaEngine::default());
    let capabilities = detect(&mut queue).await.h264;
    assert!(capabilities.iter().any(|capability| {
        capability.target == EncoderTarget::X264 && capability.support.is_ok()
    }));
    let (first, second) =
        add_valid_pair_with_error(&mut queue, input.clone(), second, invalid).await;
    let base = cpu_settings(output.clone());
    queue
        .commands
        .send(QueueCommand::StartBatch {
            items: vec![first],
            settings: OutputSettings {
                video: VideoOutput::H264(H264Settings {
                    encoder: EncoderTarget::Vaapi {
                        render_node: "/dev/dri/renderD9999".into(),
                    },
                    ..cpu_video()
                }),
                ..base.clone()
            },
        })
        .await
        .unwrap();
    assert!(matches!(
        next(&mut queue).await,
        QueueEvent::Rejected(CommandError::EncoderUnavailable { .. })
    ));

    let started = Instant::now();
    start(&queue, vec![first, second, first], base.clone()).await;
    let (summary, outputs, peak_running, progress_count) = finished(&mut queue).await;
    assert_eq!(
        (summary.succeeded, summary.failed, summary.cancelled),
        (2, 0, 0)
    );
    assert!(progress_count > 0);
    let expected_peak = std::thread::available_parallelism().unwrap().get().min(2);
    assert_eq!(peak_running, expected_peak);
    eprintln!(
        "Lote x264 de 2 entradas reais a 640x360: {:.3} s",
        started.elapsed().as_secs_f64()
    );
    assert_eq!(fs::read_to_string(&existing).unwrap(), "arquivo anterior");
    assert!(input.exists());
    let mut names: Vec<_> = outputs
        .iter()
        .map(|out| out.path.file_name().unwrap().to_str().unwrap())
        .collect();
    names.sort_unstable();
    assert_eq!(
        names,
        ["clip com espaço-h264-2.mp4", "clip com espaço-h264-3.mp4"]
    );
    for out in outputs {
        assert_transcode(&out, "isom").await;
    }

    for (container, audio, brand) in [
        (Container::Mkv, AudioMode::Aac192, "matroska"),
        (Container::Mov, AudioMode::Aac320, "qt  "),
    ] {
        start(
            &queue,
            vec![first],
            OutputSettings {
                video: VideoOutput::H264(H264Settings {
                    container,
                    audio,
                    ..cpu_video()
                }),
                ..base.clone()
            },
        )
        .await;
        let (summary, outputs, _, _) = finished(&mut queue).await;
        assert_eq!(summary.succeeded, 1);
        assert_transcode(&outputs[0], brand).await;
    }

    for capability in capabilities
        .into_iter()
        .filter(|capability| capability.target != EncoderTarget::X264)
    {
        let Ok(options) = capability.support else {
            continue;
        };
        let target = capability.target;
        let preset = options
            .presets
            .iter()
            .copied()
            .find(|preset| *preset == SpeedPreset::Nvenc(velox_transcode::domain::NvencPreset::P4))
            .unwrap_or(options.presets[0]);
        start(
            &queue,
            vec![first],
            OutputSettings {
                video: VideoOutput::H264(H264Settings {
                    encoder: target.clone(),
                    preset,
                    profile: options.profiles[0],
                    ..cpu_video()
                }),
                ..base.clone()
            },
        )
        .await;
        let (summary, outputs, _, _) = finished(&mut queue).await;
        assert_eq!(summary.succeeded, 1, "{target:?}");
        assert_transcode(&outputs[0], "isom").await;
    }
    assert!(fs::read_dir(output).unwrap().all(|entry| {
        !entry
            .unwrap()
            .file_name()
            .to_string_lossy()
            .starts_with(".velox-")
    }));
}

#[tokio::test]
#[ignore = "requer ffmpeg, ffprobe e libx264 no PATH"]
async fn handles_runtime_failure_audio_copy_and_cancellation() {
    let dir = TempDir::new().unwrap();
    let output = dir.path().join("destino");
    fs::create_dir(&output).unwrap();
    let queue_engine = MediaEngine::default();
    let mut queue = queue::spawn(queue_engine);
    detect(&mut queue).await;

    let incompatible = dir.path().join("audio-wma.mkv");
    fixture(&incompatible, "640x360", "1", Some("wmav2"));
    let id = add_and_probe(&mut queue, incompatible.clone()).await;
    let settings = cpu_settings(output.clone());
    start(&queue, vec![id], settings.clone()).await;
    assert_eq!(
        failed(&mut queue).await,
        JobFailureKind::AudioCopyIncompatible
    );
    assert_eq!(fs::read_dir(&output).unwrap().count(), 0);
    start(
        &queue,
        vec![id],
        OutputSettings {
            video: VideoOutput::H264(H264Settings {
                audio: AudioMode::Aac192,
                ..cpu_video()
            }),
            ..settings.clone()
        },
    )
    .await;
    let (summary, outputs, _, _) = finished(&mut queue).await;
    assert_eq!(summary.succeeded, 1);
    assert_transcode(&outputs[0], "isom").await;
    fs::remove_file(&outputs[0].path).unwrap();

    fs::write(&incompatible, "mídia alterada depois da sondagem").unwrap();
    start(&queue, vec![id], settings.clone()).await;
    assert_eq!(failed(&mut queue).await, JobFailureKind::TranscodeFailed);
    assert_eq!(fs::read_dir(&output).unwrap().count(), 0);

    let long = dir.path().join("lote longo.mkv");
    fixture(&long, "1280x720", "14", Some("aac"));
    let one = add_and_probe(&mut queue, long.clone()).await;
    let two = add_and_probe(&mut queue, long.clone()).await;
    let three = add_and_probe(&mut queue, long).await;
    start(
        &queue,
        vec![one, two, three],
        OutputSettings {
            video: VideoOutput::H264(H264Settings {
                preset: SpeedPreset::X264(X264Preset::Veryslow),
                ..cpu_video()
            }),
            ..settings
        },
    )
    .await;
    loop {
        if let QueueEvent::ItemProgress { batch_id, .. } = next(&mut queue).await {
            queue
                .commands
                .send(QueueCommand::CancelBatch { batch_id })
                .await
                .unwrap();
            break;
        }
    }
    let (summary, outputs, _, _) = finished(&mut queue).await;
    assert_eq!(
        (summary.succeeded, summary.failed, summary.cancelled),
        (0, 0, 3)
    );
    assert!(outputs.is_empty());
    assert_eq!(fs::read_dir(output).unwrap().count(), 0);
}

#[tokio::test]
async fn reports_missing_ffmpeg_without_starting_a_batch() {
    let directory = TempDir::new().unwrap();
    let engine = MediaEngine::new(directory.path().join("ffmpeg"), "ffprobe");
    let mut queue = queue::spawn(engine);
    queue
        .commands
        .send(QueueCommand::DetectEncoders)
        .await
        .unwrap();
    match next(&mut queue).await {
        QueueEvent::EncoderDetectionFailed(error) => {
            assert_eq!(error.kind, JobFailureKind::DependencyUnavailable)
        }
        other => panic!("{other:?}"),
    }
}

#[tokio::test]
#[ignore = "requer ffmpeg, ffprobe e prores_ks no PATH"]
async fn transcodes_prores_profiles_and_recovers_from_audio_copy() {
    let dir = TempDir::new().unwrap();
    let input = dir.path().join("Cena de teste.mkv");
    let silent = dir.path().join("sem áudio.mkv");
    let incompatible = dir.path().join("áudio TTA.mkv");
    let output = dir.path().join("destino");
    fs::create_dir(&output).unwrap();
    fixture(&input, "640x360", "1", Some("aac"));
    fixture(&silent, "640x360", "1", None);
    fixture(&incompatible, "640x360", "1", Some("tta"));
    let existing = output.join("Cena de teste-prores.mov");
    fs::write(&existing, b"arquivo anterior").unwrap();

    let mut queue = queue::spawn(MediaEngine::default());
    assert_eq!(detect(&mut queue).await.prores.unwrap(), ProResProfile::ALL);
    let scene = add_and_probe(&mut queue, input.clone()).await;
    let silent = add_and_probe(&mut queue, silent).await;
    let tta = add_and_probe(&mut queue, incompatible).await;
    let profile_settings = |profile, audio| OutputSettings {
        video: VideoOutput::ProRes { profile, audio },
        destination: output.clone(),
    };

    for (profile, name, id, audio, expected_audio) in [
        (
            ProResProfile::Proxy,
            "Proxy",
            scene,
            ProResAudio::Pcm24,
            Some("pcm_s24le"),
        ),
        (
            ProResProfile::Lt,
            "LT",
            scene,
            ProResAudio::Copy,
            Some("aac"),
        ),
        (
            ProResProfile::Standard,
            "Standard",
            silent,
            ProResAudio::Pcm24,
            None,
        ),
        (
            ProResProfile::Hq,
            "HQ",
            scene,
            ProResAudio::Pcm24,
            Some("pcm_s24le"),
        ),
    ] {
        start(&queue, vec![id], profile_settings(profile, audio)).await;
        let (summary, outputs, _, progress) = finished(&mut queue).await;
        assert_eq!(summary.succeeded, 1);
        assert!(progress > 0);
        assert_prores(&outputs[0], name, expected_audio);
    }
    assert!(input.exists());
    assert!(output.join("Cena de teste-prores-2.mov").exists());
    assert_eq!(fs::read(&existing).unwrap(), b"arquivo anterior");

    start(
        &queue,
        vec![tta],
        profile_settings(ProResProfile::Standard, ProResAudio::Copy),
    )
    .await;
    assert_eq!(
        failed(&mut queue).await,
        JobFailureKind::AudioCopyIncompatible
    );
    start(
        &queue,
        vec![tta],
        profile_settings(ProResProfile::Standard, ProResAudio::Pcm24),
    )
    .await;
    let (summary, outputs, _, _) = finished(&mut queue).await;
    assert_eq!(summary.succeeded, 1);
    assert_prores(&outputs[0], "Standard", Some("pcm_s24le"));
    assert_no_temporaries(&output);

    let cancelled_dir = dir.path().join("cancelado");
    fs::create_dir(&cancelled_dir).unwrap();
    let long = dir.path().join("lote longo.mkv");
    fixture(&long, "1280x720", "12", None);
    let first = add_and_probe(&mut queue, long.clone()).await;
    let second = add_and_probe(&mut queue, long).await;
    start(
        &queue,
        vec![first, second],
        OutputSettings {
            destination: cancelled_dir.clone(),
            ..profile_settings(ProResProfile::Hq, ProResAudio::Pcm24)
        },
    )
    .await;
    loop {
        if let QueueEvent::ItemProgress { batch_id, .. } = next(&mut queue).await {
            queue
                .commands
                .send(QueueCommand::CancelBatch { batch_id })
                .await
                .unwrap();
            break;
        }
    }
    let (summary, _, _, _) = finished(&mut queue).await;
    assert_eq!(summary.cancelled, 2);
    assert_eq!(fs::read_dir(cancelled_dir).unwrap().count(), 0);
}

#[cfg(unix)]
#[tokio::test]
#[ignore = "requer ffmpeg/ffprobe para criar e sondar a entrada"]
async fn rejects_prores_when_ffmpeg_has_no_compatible_encoder() {
    use std::os::unix::fs::PermissionsExt;

    let dir = TempDir::new().unwrap();
    let ffmpeg = dir.path().join("ffmpeg-sem-prores");
    fs::write(&ffmpeg, "#!/bin/sh\nprintf '%s\\n' ' V..... prores_aw'\n").unwrap();
    fs::set_permissions(&ffmpeg, fs::Permissions::from_mode(0o700)).unwrap();
    let mut queue = queue::spawn(MediaEngine::new(ffmpeg, "ffprobe"));
    assert_eq!(
        detect(&mut queue).await.prores,
        Err(EncoderUnavailable::MissingFromFfmpeg)
    );
    let input = dir.path().join("entrada.mkv");
    fixture(&input, "640x360", "1", None);
    let id = add_and_probe(&mut queue, input).await;
    start(
        &queue,
        vec![id],
        OutputSettings {
            video: VideoOutput::ProRes {
                profile: ProResProfile::Standard,
                audio: ProResAudio::Pcm24,
            },
            destination: dir.path().into(),
        },
    )
    .await;
    assert!(matches!(
        next(&mut queue).await,
        QueueEvent::Rejected(CommandError::ProResUnavailable)
    ));
}

fn assert_prores(job: &JobOutput, profile: &str, audio: Option<&str>) {
    let output = Command::new("ffprobe")
        .args(["-v", "error", "-of", "json", "-show_entries",
            "stream=codec_name,codec_type,profile,pix_fmt,width,height,bits_per_sample:format_tags=major_brand"])
        .arg(&job.path)
        .output()
        .unwrap();
    assert!(output.status.success());
    let probe: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let streams = probe["streams"].as_array().unwrap();
    assert_eq!(probe["format"]["tags"]["major_brand"], "qt  ");
    assert_eq!(job.path.extension().unwrap(), "mov");
    let video = &streams[0];
    assert_eq!(video["codec_name"], "prores");
    assert_eq!(video["profile"], profile);
    assert_eq!(video["pix_fmt"], "yuv422p10le");
    assert_eq!(
        (video["width"].as_u64(), video["height"].as_u64()),
        (Some(640), Some(360))
    );
    assert_eq!(streams.len(), 1 + usize::from(audio.is_some()));
    if let Some(codec) = audio {
        assert_eq!(streams[1]["codec_name"], codec);
        if codec == "pcm_s24le" {
            assert_eq!(streams[1]["bits_per_sample"], 24);
        }
    }
}

fn assert_no_temporaries(output: &Path) {
    assert!(fs::read_dir(output).unwrap().all(|entry| {
        !entry
            .unwrap()
            .file_name()
            .to_string_lossy()
            .starts_with(".velox-")
    }));
}

fn fixture(path: &Path, size: &str, seconds: &str, codec: Option<&str>) {
    let mut command = Command::new("ffmpeg");
    command.args([
        "-v",
        "error",
        "-nostdin",
        "-f",
        "lavfi",
        "-i",
        &format!("testsrc2=s={size}:r=30"),
    ]);
    if let Some(codec) = codec {
        command.args([
            "-f",
            "lavfi",
            "-i",
            "sine=f=1000:sample_rate=48000",
            "-c:a",
            codec,
        ]);
    }
    let output = command
        .args([
            "-t",
            seconds,
            "-c:v",
            "mpeg4",
            "-q:v",
            "6",
            "-threads:v",
            "2",
        ])
        .arg(path)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "fixture: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn cpu_settings(destination: PathBuf) -> OutputSettings {
    OutputSettings {
        video: VideoOutput::H264(cpu_video()),
        destination,
    }
}

fn cpu_video() -> H264Settings {
    H264Settings {
        encoder: EncoderTarget::X264,
        quality: Quality::DEFAULT,
        preset: SpeedPreset::X264(X264Preset::Medium),
        profile: H264Profile::High,
        audio: AudioMode::Copy,
        container: Container::Mp4,
    }
}

async fn next(queue: &mut Queue) -> QueueEvent {
    tokio::time::timeout(Duration::from_secs(45), queue.events.recv())
        .await
        .expect("evento da fila excedeu 45 s")
        .expect("fila encerrada")
}

async fn detect(queue: &mut Queue) -> OutputCapabilities {
    queue
        .commands
        .send(QueueCommand::DetectEncoders)
        .await
        .unwrap();
    let QueueEvent::EncoderCapabilities(capabilities) = next(queue).await else {
        panic!("lista de encoders esperada")
    };
    for capability in &capabilities.h264 {
        eprintln!(
            "{}: {:?}",
            if capability.support.is_ok() {
                "disponível"
            } else {
                "indisponível"
            },
            capability.target
        );
    }
    capabilities
}

async fn add_and_probe(queue: &mut Queue, path: PathBuf) -> MediaId {
    queue
        .commands
        .send(QueueCommand::AddFiles { paths: vec![path] })
        .await
        .unwrap();
    let QueueEvent::ItemAdded { media_id, .. } = next(queue).await else {
        panic!("ItemAdded esperado")
    };
    let event = next(queue).await;
    assert!(matches!(event, QueueEvent::MediaProbed(info) if info.id == media_id));
    media_id
}

async fn add_valid_pair_with_error(
    queue: &mut Queue,
    first: PathBuf,
    second: PathBuf,
    invalid: PathBuf,
) -> (MediaId, MediaId) {
    queue
        .commands
        .send(QueueCommand::AddFiles {
            paths: vec![first, second, invalid],
        })
        .await
        .unwrap();
    let mut added = Vec::new();
    while added.len() < 3 {
        let QueueEvent::ItemAdded { media_id, .. } = next(queue).await else {
            panic!("ItemAdded esperado")
        };
        added.push(media_id);
    }
    let mut valid = 0;
    let mut invalid = false;
    for _ in 0..3 {
        match next(queue).await {
            QueueEvent::MediaProbed(info) => {
                assert!(added[..2].contains(&info.id));
                assert_eq!(info.video.codec, "mpeg4");
                assert_eq!((info.video.width, info.video.height), (640, 360));
                assert_eq!(info.video.frame_rate.unwrap().numerator.get(), 30);
                assert_eq!(info.audio[0].codec, "aac");
                assert!(info.duration.is_some() && info.container_bit_rate.is_some());
                assert!(info.source_bytes > 0);
                valid += 1;
            }
            QueueEvent::ItemChanged {
                media_id,
                state: ItemState::ProbeFailed(error),
            } => {
                assert_eq!(media_id, added[2]);
                assert_eq!(error.kind, JobFailureKind::InvalidMedia);
                invalid = true;
            }
            other => panic!("{other:?}"),
        }
    }
    assert_eq!(valid, 2);
    assert!(invalid);
    (added[0], added[1])
}

async fn start(queue: &Queue, items: Vec<MediaId>, settings: OutputSettings) {
    queue
        .commands
        .send(QueueCommand::StartBatch { items, settings })
        .await
        .unwrap();
}

async fn finished(queue: &mut Queue) -> (BatchSummary, Vec<JobOutput>, usize, usize) {
    let mut outputs = Vec::new();
    let mut running = 0_usize;
    let mut peak_running = 0;
    let mut progress_count = 0;
    let mut last_percent = 0.0;
    loop {
        match next(queue).await {
            QueueEvent::BatchFinished(summary) => {
                assert_eq!(last_percent, 100.0);
                return (summary, outputs, peak_running, progress_count);
            }
            QueueEvent::ItemChanged {
                state: ItemState::Running { .. },
                ..
            } => {
                running += 1;
                peak_running = peak_running.max(running);
            }
            QueueEvent::ItemProgress { .. } => progress_count += 1,
            QueueEvent::ItemChanged {
                state: ItemState::Succeeded { output, .. },
                ..
            } => {
                running = running.saturating_sub(1);
                outputs.push(output);
            }
            QueueEvent::ItemChanged {
                state: ItemState::Failed { error, .. },
                ..
            } => panic!("{error:?}"),
            QueueEvent::Rejected(error) => panic!("{error:?}"),
            QueueEvent::BatchProgress { progress, .. } => {
                assert!(progress.percent + 1e-9 >= last_percent);
                last_percent = progress.percent;
            }
            _ => (),
        }
    }
}

async fn failed(queue: &mut Queue) -> JobFailureKind {
    let mut kind = None;
    loop {
        match next(queue).await {
            QueueEvent::ItemChanged {
                state: ItemState::Failed { error, .. },
                ..
            } => kind = Some(error.kind),
            QueueEvent::BatchFinished(summary) => {
                assert_eq!(summary.failed, 1);
                return kind.unwrap();
            }
            QueueEvent::Rejected(error) => panic!("{error:?}"),
            _ => (),
        }
    }
}

async fn assert_transcode(job: &JobOutput, brand: &str) {
    let info = MediaEngine::default()
        .probe(MediaId(0), &job.path)
        .await
        .unwrap();
    assert_eq!(info.video.codec, "h264");
    assert_eq!(info.audio[0].codec, "aac");
    assert_eq!(info.source_bytes, job.bytes);
    assert!(job.reduction_percent.is_some_and(f64::is_finite));
    let output = Command::new("ffprobe")
        .args([
            "-v",
            "error",
            "-of",
            "default=nw=1",
            "-show_entries",
            "format=format_name:format_tags=major_brand",
        ])
        .arg(&job.path)
        .output()
        .unwrap();
    assert!(output.status.success());
    assert!(String::from_utf8(output.stdout).unwrap().contains(brand));
}
