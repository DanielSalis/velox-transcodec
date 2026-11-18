use std::{num::NonZeroU32, path::PathBuf, time::Duration};

/// Identificador atribuído pela fila e preservado entre lotes da mesma sessão.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct MediaId(pub u64);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FrameRate {
    pub numerator: NonZeroU32,
    pub denominator: NonZeroU32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VideoStream {
    /// Índice absoluto do stream devolvido pelo ffprobe.
    pub index: u32,
    pub codec: String,
    pub width: u32,
    pub height: u32,
    pub frame_rate: Option<FrameRate>,
    pub bit_rate: Option<u64>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AudioStream {
    /// Índice absoluto do stream devolvido pelo ffprobe.
    pub index: u32,
    pub codec: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MediaInfo {
    pub id: MediaId,
    pub path: PathBuf,
    pub video: VideoStream,
    pub audio: Vec<AudioStream>,
    pub container_bit_rate: Option<u64>,
    pub duration: Option<Duration>,
    pub source_bytes: u64,
}
