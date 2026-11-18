use std::{collections::HashMap, path::PathBuf, rc::Rc, time::Duration};

use slint::{Model, ModelRc, SharedString, VecModel};
use tokio::sync::mpsc;
use velox_transcode::domain::{
    AudioMode, BatchId, BatchProgress, BatchSummary, CommandError, Container, EncoderOptions,
    EncoderTarget, EncoderUnavailable, H264Profile, H264Settings, ItemState, JobFailure,
    JobFailureKind, JobOutput, MediaId, MediaInfo, NvencPreset, OutputCapabilities, OutputSettings,
    ProResAudio, ProResProfile, ProgressSnapshot, Quality, QualityMode, QueueCommand, QueueEvent,
    SpeedPreset, VideoOutput, X264Preset,
};

use crate::{DesktopWindow, FileRowView};

enum Phase {
    Probing,
    Ready,
    Queued,
    Running(ProgressSnapshot),
    Succeeded(JobOutput),
    Failed(JobFailure),
    Cancelled,
}

struct FileEntry {
    id: MediaId,
    name: SharedString,
    info: Option<MediaInfo>,
    phase: Phase,
    included: bool,
}

impl FileEntry {
    fn row(&self) -> FileRowView {
        let (state, progress, show_progress) = match &self.phase {
            Phase::Probing => ("Lendo metadados…".into(), 0.0, false),
            Phase::Ready => ("Pronto".into(), 0.0, false),
            Phase::Queued => ("Na fila".into(), 0.0, false),
            Phase::Running(progress) => (
                progress_text(*progress),
                progress.percent.unwrap_or_default() as f32 / 100.0,
                progress.percent.is_some(),
            ),
            Phase::Succeeded(output) => (
                output.reduction_percent.map_or_else(
                    || "Concluído".into(),
                    |reduction| format!("Concluído · {}", size_change(reduction)),
                ),
                1.0,
                true,
            ),
            Phase::Failed(_) => ("Falhou · consulte os detalhes".into(), 0.0, false),
            Phase::Cancelled => ("Cancelado".into(), 0.0, false),
        };
        let (codec, resolution, duration) = self.info.as_ref().map_or_else(
            || ("—".into(), "—".into(), "—".into()),
            |info| {
                (
                    codec_label(&info.video.codec),
                    format!("{}×{}", info.video.width, info.video.height),
                    info.duration.map(time_label).unwrap_or_else(|| "—".into()),
                )
            },
        );
        FileRowView {
            name: self.name.clone(),
            codec: codec.into(),
            resolution: resolution.into(),
            duration: duration.into(),
            state: state.into(),
            progress,
            show_progress,
            included: self.included
                || matches!(self.phase, Phase::Succeeded(_))
                || self.info.is_none(),
        }
    }

    fn ready(&self) -> bool {
        self.included && self.info.is_some()
    }
}

struct Encoder {
    target: EncoderTarget,
    options: EncoderOptions,
}

#[derive(Default)]
enum RunState {
    #[default]
    Idle,
    Starting,
    Active {
        id: BatchId,
        cancelling: bool,
    },
    Finished(BatchSummary),
}

impl RunState {
    fn busy(&self) -> bool {
        matches!(self, Self::Starting | Self::Active { .. })
    }
}

pub(super) struct Desktop {
    commands: mpsc::Sender<QueueCommand>,
    rows: Rc<VecModel<FileRowView>>,
    files: Vec<FileEntry>,
    index: HashMap<MediaId, usize>,
    selected: Option<usize>,
    encoders: Vec<Encoder>,
    prores_profiles: Vec<ProResProfile>,
    destination: Option<PathBuf>,
    run: RunState,
    closing: bool,
    connected: bool,
}

impl Desktop {
    pub(super) fn new(window: &DesktopWindow, commands: mpsc::Sender<QueueCommand>) -> Self {
        let rows = Rc::new(VecModel::default());
        window.set_files(ModelRc::from(rows.clone()));
        window.set_quality_minimum(i32::from(Quality::MIN));
        window.set_quality_maximum(i32::from(Quality::MAX));
        window.set_quality(i32::from(Quality::DEFAULT.value()));
        Self {
            commands,
            rows,
            files: Vec::new(),
            index: HashMap::new(),
            selected: None,
            encoders: Vec::new(),
            prores_profiles: Vec::new(),
            destination: None,
            run: RunState::Idle,
            closing: false,
            connected: true,
        }
    }

    pub(super) fn add_files(&self, window: &DesktopWindow, paths: Vec<PathBuf>) -> bool {
        if !paths.is_empty() && self.send(window, QueueCommand::AddFiles { paths }) {
            window.set_queue_message("".into());
            true
        } else {
            false
        }
    }

    pub(super) fn set_destination(&mut self, window: &DesktopWindow, path: PathBuf) {
        window.set_destination(path.to_string_lossy().as_ref().into());
        window.set_queue_message("".into());
        self.destination = Some(path);
        self.refresh_controls(window);
    }

    pub(super) fn detect(&mut self, window: &DesktopWindow) {
        if self.send(window, QueueCommand::DetectEncoders) {
            self.encoders.clear();
            self.prores_profiles.clear();
            window.set_encoders(model(Vec::new()));
            window.set_presets(model(Vec::new()));
            window.set_preset_descriptions(model(Vec::new()));
            window.set_profiles(model(Vec::new()));
            window.set_profile_descriptions(model(Vec::new()));
            window.set_prores_profiles(model(Vec::new()));
            window.set_quality_code("".into());
            window.set_quality_explanation("".into());
            window.set_detecting(true);
            window.set_encoder_message("".into());
            window.set_prores_message("".into());
            self.refresh_controls(window);
        }
    }

    pub(super) fn select_codec(&self, window: &DesktopWindow) {
        window.set_queue_message("".into());
        self.refresh_controls(window);
    }

    pub(super) fn select_encoder(&self, window: &DesktopWindow, index: i32) {
        let Some(encoder) = at(&self.encoders, index) else {
            return;
        };
        let options = &encoder.options;
        let (code, explanation) = match options.quality_mode {
            QualityMode::Crf => ("CRF", "O x264 ajusta a compressão conforme a cena."),
            QualityMode::ConstantQuality => (
                "CQ",
                "O NVENC varia o bitrate para atingir a meta de qualidade.",
            ),
            QualityMode::ConstantQuantizer => ("QP", "O VA-API usa um nível fixo de quantização."),
        };
        window.set_quality_code(code.into());
        window.set_quality_explanation(explanation.into());

        let preferred = match encoder.target {
            EncoderTarget::Vaapi { .. } => SpeedPreset::Automatic,
            EncoderTarget::Nvenc { .. } => SpeedPreset::Nvenc(NvencPreset::P4),
            EncoderTarget::X264 => SpeedPreset::X264(X264Preset::Medium),
        };
        let preset = options
            .presets
            .iter()
            .position(|preset| *preset == preferred)
            .unwrap_or_default();
        let profile = options
            .profiles
            .iter()
            .position(|profile| *profile == H264Profile::High)
            .unwrap_or_default();
        let (presets, preset_descriptions) = options
            .presets
            .iter()
            .map(|&preset| {
                let (label, description) = preset_text(preset);
                (label.into(), description.into())
            })
            .unzip();
        let (profiles, profile_descriptions) = options
            .profiles
            .iter()
            .map(|profile| {
                let (label, description) = match profile {
                    H264Profile::Baseline => ("Baseline", "Aparelhos antigos"),
                    H264Profile::Main => ("Main", "Compatibilidade intermediária"),
                    H264Profile::High => ("High", "Aparelhos modernos"),
                };
                (label.into(), description.into())
            })
            .unzip();
        window.set_presets(model(presets));
        window.set_preset_descriptions(model(preset_descriptions));
        window.set_profiles(model(profiles));
        window.set_profile_descriptions(model(profile_descriptions));
        window.set_preset_index(preset as i32);
        window.set_profile_index(profile as i32);
    }

    pub(super) fn inspect(&mut self, window: &DesktopWindow, index: i32) {
        let Ok(index) = usize::try_from(index) else {
            return;
        };
        if index < self.files.len() {
            self.selected = Some(index);
            window.set_selected_file(index as i32);
            self.refresh_details(window);
        }
    }

    pub(super) fn toggle_inclusion(&mut self, window: &DesktopWindow) {
        let Some(index) = self.selected else { return };
        let file = &mut self.files[index];
        if !self.run.busy() && file.info.is_some() {
            file.included = !file.included;
            window.set_queue_message("".into());
            self.refresh_row(window, index);
            self.refresh_controls(window);
        }
    }

    pub(super) fn recover_audio(&mut self, window: &DesktopWindow) {
        if let Some(index) = self.selected {
            self.files[index].included = true;
            self.refresh_row(window, index);
            self.refresh_controls(window);
        }
        let label = if window.get_prores() {
            window.set_prores_audio_index(0);
            "PCM 24 bits"
        } else {
            window.set_audio_index(1);
            "AAC 192 kbps"
        };
        window.set_queue_message(
            format!("{label} selecionado. Inicie a fila para tentar novamente.").into(),
        );
    }

    pub(super) fn toggle_queue(&mut self, window: &DesktopWindow) {
        if self.run.busy() {
            self.cancel(window);
        } else if let Some(settings) = self.settings(window) {
            let items = self
                .files
                .iter()
                .filter(|file| file.ready())
                .map(|file| file.id)
                .collect::<Vec<_>>();
            if !items.is_empty() && self.send(window, QueueCommand::StartBatch { items, settings })
            {
                self.run = RunState::Starting;
                window.set_queue_message("".into());
                window.set_queue_percent(0);
                window.set_queue_fraction(0.0);
                self.refresh_controls(window);
            }
        }
    }

    pub(super) fn defer_close(&mut self, window: &DesktopWindow) -> bool {
        if !self.run.busy() {
            return false;
        }
        self.closing = true;
        self.cancel(window);
        window.set_queue_message("Encerrando a fila antes de fechar…".into());
        true
    }

    fn cancel(&mut self, window: &DesktopWindow) {
        if let RunState::Active {
            id,
            cancelling: false,
        } = self.run
            && self.send(window, QueueCommand::CancelBatch { batch_id: id })
        {
            self.run = RunState::Active {
                id,
                cancelling: true,
            };
            self.refresh_controls(window);
        }
    }

    fn settings(&self, window: &DesktopWindow) -> Option<OutputSettings> {
        let destination = self.destination.clone()?;
        let video = if window.get_prores() {
            VideoOutput::ProRes {
                profile: *at(&self.prores_profiles, window.get_prores_profile_index())?,
                audio: *at(
                    &[ProResAudio::Pcm24, ProResAudio::Copy],
                    window.get_prores_audio_index(),
                )?,
            }
        } else {
            let encoder = at(&self.encoders, window.get_encoder_index())?;
            VideoOutput::H264(H264Settings {
                encoder: encoder.target.clone(),
                quality: Quality::new(u8::try_from(window.get_quality()).ok()?)?,
                preset: *at(&encoder.options.presets, window.get_preset_index())?,
                profile: *at(&encoder.options.profiles, window.get_profile_index())?,
                audio: *at(
                    &[AudioMode::Copy, AudioMode::Aac192, AudioMode::Aac320],
                    window.get_audio_index(),
                )?,
                container: *at(
                    &[Container::Mp4, Container::Mkv, Container::Mov],
                    window.get_container_index(),
                )?,
            })
        };
        Some(OutputSettings { video, destination })
    }

    fn send(&self, window: &DesktopWindow, command: QueueCommand) -> bool {
        match self.commands.try_send(command) {
            Ok(()) => true,
            Err(mpsc::error::TrySendError::Full(_)) => {
                window.set_queue_message(
                    "A fila está recebendo outros comandos. Tente novamente.".into(),
                );
                false
            }
            Err(mpsc::error::TrySendError::Closed(_)) => {
                window.set_queue_message(
                    "A fila está indisponível. Reabra o Velox para tentar novamente.".into(),
                );
                false
            }
        }
    }

    pub(super) fn disconnected(&mut self, window: &DesktopWindow) {
        self.connected = false;
        self.run = RunState::Idle;
        window.set_detecting(false);
        window.set_queue_message("A fila foi encerrada. Reabra o Velox para continuar.".into());
        self.refresh_controls(window);
        self.close_if_requested();
    }

    pub(super) fn apply(
        &mut self,
        window: &DesktopWindow,
        event: QueueEvent,
    ) -> Option<BatchSummary> {
        match event {
            QueueEvent::ItemAdded { media_id, path } => {
                let index = self.files.len();
                let name = path
                    .file_name()
                    .unwrap_or(path.as_os_str())
                    .to_string_lossy();
                let file = FileEntry {
                    id: media_id,
                    name: name.as_ref().into(),
                    info: None,
                    phase: Phase::Probing,
                    included: true,
                };
                self.index.insert(media_id, index);
                self.rows.push(file.row());
                self.files.push(file);
                if self.selected.is_none() {
                    self.inspect(window, index as i32);
                }
            }
            QueueEvent::MediaProbed(info) => {
                if let Some(&index) = self.index.get(&info.id) {
                    let file = &mut self.files[index];
                    file.info = Some(info);
                    file.phase = Phase::Ready;
                    self.refresh_row(window, index);
                }
            }
            QueueEvent::EncoderCapabilities(capabilities) => {
                self.set_encoders(window, capabilities)
            }
            QueueEvent::EncoderDetectionFailed(error) => {
                eprintln!("Falha na sondagem dos encoders: {}", error.detail);
                window.set_detecting(false);
                let message = "O FFmpeg não pôde verificar os encoders. Confira a instalação.";
                window.set_encoder_message(message.into());
                window.set_prores_message(message.into());
            }
            QueueEvent::BatchStarted { batch_id, .. } => {
                self.run = RunState::Active {
                    id: batch_id,
                    cancelling: false,
                };
                if self.closing {
                    self.cancel(window);
                }
            }
            QueueEvent::ItemChanged { media_id, state } => {
                self.change_item(window, media_id, state)
            }
            QueueEvent::ItemProgress {
                media_id, progress, ..
            } => {
                if let Some(&index) = self.index.get(&media_id) {
                    self.files[index].phase = Phase::Running(progress);
                    self.rows.set_row_data(index, self.files[index].row());
                }
                return None;
            }
            QueueEvent::BatchProgress { progress, .. } => {
                set_batch_progress(window, progress, &self.run);
                return None;
            }
            QueueEvent::BatchFinished(summary) => {
                self.run = RunState::Finished(summary);
                window.set_queue_message("".into());
                self.refresh_controls(window);
                self.close_if_requested();
                return (!self.closing).then_some(summary);
            }
            QueueEvent::Rejected(error) => {
                window.set_queue_message(command_message(&error).into());
                if matches!(self.run, RunState::Starting)
                    || matches!(error, CommandError::BatchNotRunning { .. }) && self.run.busy()
                {
                    self.run = RunState::Idle;
                    self.close_if_requested();
                }
            }
        }
        self.refresh_controls(window);
        None
    }

    fn close_if_requested(&self) {
        if self.closing {
            let _ = slint::quit_event_loop();
        }
    }

    fn set_encoders(&mut self, window: &DesktopWindow, capabilities: OutputCapabilities) {
        let mut available = Vec::new();
        for capability in capabilities.h264 {
            match capability.support {
                Ok(options) => available.push(Encoder {
                    target: capability.target,
                    options,
                }),
                Err(error) => eprintln!("Encoder {:?} indisponível: {error:?}", capability.target),
            }
        }
        self.encoders = available;
        window.set_encoders(model(
            self.encoders
                .iter()
                .map(|encoder| encoder_label(&encoder.target).into())
                .collect(),
        ));
        window.set_encoder_index(0);
        self.select_encoder(window, 0);
        window.set_detecting(false);
        window.set_encoder_message(
            if self.encoders.is_empty() {
                "Nenhum encoder H.264 disponível. Confira os codecs e drivers do FFmpeg."
            } else {
                ""
            }
            .into(),
        );

        let (profiles, message) = match capabilities.prores {
            Ok(profiles) => (profiles, ""),
            Err(EncoderUnavailable::MissingFromFfmpeg) => (
                Vec::new(),
                "O FFmpeg instalado não inclui o encoder ProRes (prores_ks).",
            ),
            Err(EncoderUnavailable::ProbeFailed { detail }) => {
                eprintln!("Encoder ProRes indisponível: {detail}");
                (
                    Vec::new(),
                    "O FFmpeg encontrou prores_ks, mas falhou ao testar a codificação ProRes.",
                )
            }
        };
        let default = profiles
            .iter()
            .position(|profile| *profile == ProResProfile::Standard);
        window.set_prores_profiles(model(
            profiles
                .iter()
                .map(|profile| {
                    SharedString::from(match profile {
                        ProResProfile::Proxy => "422 Proxy",
                        ProResProfile::Lt => "422 LT",
                        ProResProfile::Standard => "422",
                        ProResProfile::Hq => "422 HQ",
                    })
                })
                .collect(),
        ));
        window.set_prores_profile_index(default.unwrap_or_default() as i32);
        window.set_prores_message(message.into());
        self.prores_profiles = profiles;
    }

    fn change_item(&mut self, window: &DesktopWindow, media_id: MediaId, change: ItemState) {
        let Some(&index) = self.index.get(&media_id) else {
            return;
        };
        let file = &mut self.files[index];
        file.phase = match change {
            ItemState::ProbeFailed(error) => {
                file.included = false;
                Phase::Failed(error)
            }
            ItemState::Queued { .. } => Phase::Queued,
            ItemState::Running { .. } => Phase::Running(ProgressSnapshot::default()),
            ItemState::Succeeded { output, .. } => {
                file.included = false;
                Phase::Succeeded(output)
            }
            ItemState::Failed { error, .. } => Phase::Failed(error),
            ItemState::Cancelled { .. } => Phase::Cancelled,
        };
        self.refresh_row(window, index);
    }

    fn refresh_row(&self, window: &DesktopWindow, index: usize) {
        self.rows.set_row_data(index, self.files[index].row());
        if self.selected == Some(index) {
            self.refresh_details(window);
        }
    }

    fn refresh_details(&self, window: &DesktopWindow) {
        let Some(file) = self.selected.and_then(|index| self.files.get(index)) else {
            return;
        };
        let info = file.info.as_ref();
        let fps = info.and_then(|info| info.video.frame_rate).map(|rate| {
            decimal(
                rate.numerator.get() as f64 / rate.denominator.get() as f64,
                2,
            )
        });
        let bitrate = info
            .and_then(|info| info.video.bit_rate.or(info.container_bit_rate))
            .map(bitrate_label);
        let size = info.map(|info| match &file.phase {
            Phase::Succeeded(output) => format!(
                "{} → {}",
                byte_label(info.source_bytes),
                byte_label(output.bytes)
            ),
            _ => byte_label(info.source_bytes),
        });
        let (message, recover) = match &file.phase {
            Phase::Failed(error) => (
                failure_message(error),
                error.kind == JobFailureKind::AudioCopyIncompatible,
            ),
            Phase::Succeeded(output) => (format!("Salvo em {}", output.path.display()), false),
            _ => (String::new(), false),
        };
        window.set_detail_fps(fps.unwrap_or_else(|| "—".into()).into());
        window.set_detail_bitrate(bitrate.unwrap_or_else(|| "—".into()).into());
        window.set_detail_size(size.unwrap_or_else(|| "—".into()).into());
        window.set_detail_message(message.into());
        window.set_detail_recover_audio(recover);
        window.set_detail_can_toggle(info.is_some());
        window.set_detail_included(file.included);
    }

    fn refresh_controls(&self, window: &DesktopWindow) {
        let ready = self.files.iter().filter(|file| file.ready()).count();
        let busy = self.run.busy();
        window.set_busy(busy);
        window.set_queue_action(if busy { "Cancelar" } else { "Iniciar fila" }.into());
        let enabled = match self.run {
            RunState::Starting => false,
            RunState::Active { cancelling, .. } => !cancelling,
            _ => self.connected && ready > 0 && self.settings(window).is_some(),
        };
        window.set_queue_action_enabled(enabled);
        let status = match self.run {
            RunState::Starting => "Iniciando a fila…".into(),
            RunState::Active {
                cancelling: true, ..
            } => "Cancelando a fila…".into(),
            RunState::Active {
                cancelling: false, ..
            } => return,
            RunState::Finished(summary) => batch_summary(summary),
            RunState::Idle if self.files.is_empty() => "Adicione arquivos para começar.".into(),
            RunState::Idle if ready == 0 => "Nenhum vídeo pronto incluído na fila.".into(),
            RunState::Idle if self.destination.is_none() => "Escolha uma pasta de destino.".into(),
            RunState::Idle if ready == 1 => "1 vídeo pronto para converter.".into(),
            RunState::Idle => format!("{ready} vídeos prontos para converter."),
        };
        window.set_queue_status(status.into());
    }
}

fn at<T>(values: &[T], index: i32) -> Option<&T> {
    values.get(usize::try_from(index).ok()?)
}

fn model(values: Vec<SharedString>) -> ModelRc<SharedString> {
    ModelRc::from(Rc::new(VecModel::from(values)))
}

fn set_batch_progress(window: &DesktopWindow, progress: BatchProgress, run: &RunState) {
    window.set_queue_fraction((progress.percent / 100.0) as f32);
    window.set_queue_percent(progress.percent.floor() as i32);
    if matches!(
        run,
        RunState::Active {
            cancelling: false,
            ..
        }
    ) {
        let label = if progress.total_items == 1 {
            "arquivo processado"
        } else {
            "arquivos processados"
        };
        let mut status = format!(
            "{} de {} {label}",
            progress.terminal_items, progress.total_items
        );
        if let Some(eta) = progress.eta {
            status.push_str(&format!(" · ETA {}", time_label(eta)));
        }
        window.set_queue_status(status.into());
    }
}

pub(super) fn batch_summary(summary: BatchSummary) -> String {
    let succeeded = if summary.succeeded == 1 {
        "concluído"
    } else {
        "concluídos"
    };
    let failed = if summary.failed == 1 {
        "falha"
    } else {
        "falhas"
    };
    let cancelled = if summary.cancelled == 1 {
        "cancelado"
    } else {
        "cancelados"
    };
    format!(
        "{} {succeeded} · {} {failed} · {} {cancelled}",
        summary.succeeded, summary.failed, summary.cancelled
    )
}

fn codec_label(codec: &str) -> String {
    match codec {
        "h264" => "H.264".into(),
        "prores" => "ProRes".into(),
        "unknown" => "—".into(),
        _ => codec.to_ascii_uppercase(),
    }
}

fn progress_text(progress: ProgressSnapshot) -> String {
    let mut status = "Codificando".to_owned();
    if let Some(percent) = progress.percent {
        status.push_str(&format!(" · {}%", percent.floor()));
    }
    if let Some(fps) = progress.fps {
        status.push_str(&format!(" · {} FPS", decimal(fps, 1)));
    }
    if let Some(eta) = progress.eta {
        status.push_str(&format!(" · ETA {}", time_label(eta)));
    }
    status
}

fn size_change(reduction: f64) -> String {
    if reduction < 0.0 {
        format!("+{}% de tamanho", decimal(-reduction, 1))
    } else {
        format!("−{}% de tamanho", decimal(reduction, 1))
    }
}

fn decimal(number: f64, places: usize) -> String {
    let text = format!("{number:.places$}");
    if places == 0 {
        return text;
    }
    text.trim_end_matches('0')
        .trim_end_matches('.')
        .replace('.', ",")
}

fn time_label(time: Duration) -> String {
    let seconds = time.as_secs();
    let (hours, minutes, seconds) = (seconds / 3600, seconds / 60 % 60, seconds % 60);
    if hours == 0 {
        format!("{minutes:02}:{seconds:02}")
    } else {
        format!("{hours:02}:{minutes:02}:{seconds:02}")
    }
}

fn bitrate_label(bits: u64) -> String {
    if bits >= 1_000_000 {
        format!("{} Mb/s", decimal(bits as f64 / 1_000_000.0, 1))
    } else {
        format!("{} kb/s", decimal(bits as f64 / 1000.0, 1))
    }
}

fn byte_label(bytes: u64) -> String {
    for (scale, unit) in [(1_000_000_000, "GB"), (1_000_000, "MB"), (1000, "kB")] {
        if bytes >= scale {
            return format!("{} {unit}", decimal(bytes as f64 / scale as f64, 1));
        }
    }
    format!("{bytes} B")
}

fn encoder_label(target: &EncoderTarget) -> String {
    match target {
        EncoderTarget::Vaapi { render_node } => format!(
            "VA-API · {}",
            render_node
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
        ),
        EncoderTarget::Nvenc { gpu_index } => format!("NVENC · NVIDIA GPU {gpu_index}"),
        EncoderTarget::X264 => "x264 · CPU".into(),
    }
}

fn preset_text(preset: SpeedPreset) -> (&'static str, &'static str) {
    match preset {
        SpeedPreset::Automatic => ("Automático", "Gerenciado pelo encoder"),
        SpeedPreset::X264(preset) => match preset {
            X264Preset::Ultrafast => ("ultrafast", "Mais rápido"),
            X264Preset::Superfast => ("superfast", "Muito rápido"),
            X264Preset::Veryfast => ("veryfast", "Muito rápido"),
            X264Preset::Faster => ("faster", "Rápido"),
            X264Preset::Fast => ("fast", "Rápido"),
            X264Preset::Medium => ("medium", "Equilibrado"),
            X264Preset::Slow => ("slow", "Lento"),
            X264Preset::Slower => ("slower", "Muito lento"),
            X264Preset::Veryslow => ("veryslow", "Mais lento"),
        },
        SpeedPreset::Nvenc(preset) => match preset {
            NvencPreset::P1 => ("P1", "Mais rápido"),
            NvencPreset::P2 => ("P2", "Muito rápido"),
            NvencPreset::P3 => ("P3", "Rápido"),
            NvencPreset::P4 => ("P4", "Equilibrado"),
            NvencPreset::P5 => ("P5", "Lento"),
            NvencPreset::P6 => ("P6", "Muito lento"),
            NvencPreset::P7 => ("P7", "Mais lento"),
        },
    }
}

fn failure_message(error: &JobFailure) -> String {
    if error.kind == JobFailureKind::AudioCopyIncompatible {
        return "O áudio original é incompatível com este container. Escolha outra opção de áudio para tentar novamente.".into();
    }
    let title = match error.kind {
        JobFailureKind::DependencyUnavailable => "Ferramenta de mídia indisponível",
        JobFailureKind::InputUnavailable => "Arquivo de entrada indisponível",
        JobFailureKind::InvalidMedia => "Vídeo inválido ou incompatível",
        JobFailureKind::EncoderUnavailable => "Encoder indisponível",
        JobFailureKind::TranscodeFailed => "Falha ao converter",
        JobFailureKind::OutputUnavailable => "Não foi possível salvar a saída",
        JobFailureKind::AudioCopyIncompatible => unreachable!(),
    };
    error
        .detail
        .lines()
        .next()
        .filter(|line| !line.is_empty())
        .map_or_else(|| title.into(), |line| format!("{title}. {line}"))
}

fn command_message(error: &CommandError) -> &'static str {
    match error {
        CommandError::EmptySelection => "Inclua ao menos um vídeo na fila.",
        CommandError::BatchAlreadyRunning { .. } => "Já há uma fila em execução.",
        CommandError::BatchNotRunning { .. } => "Esta fila já foi encerrada.",
        CommandError::ItemNotReady { .. } => {
            "Aguarde a leitura dos metadados dos arquivos incluídos."
        }
        CommandError::EncoderUnavailable { .. } => {
            "O encoder não está disponível. Verifique os encoders ou escolha outro."
        }
        CommandError::UnsupportedEncoderSettings { .. } => {
            "Escolha um perfil e preset disponíveis para o encoder."
        }
        CommandError::ProResUnavailable => {
            "O encoder ProRes está indisponível. Verifique os encoders do FFmpeg."
        }
        CommandError::UnsupportedProResProfile => "Escolha um perfil ProRes disponível.",
        CommandError::AudioCopyIncompatible { .. } => {
            "Escolha outra opção para converter este áudio."
        }
        CommandError::DestinationUnavailable { .. } => {
            "A pasta de destino está indisponível. Escolha outra pasta."
        }
    }
}
