use std::path::PathBuf;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum EncoderTarget {
    Vaapi { render_node: PathBuf },
    Nvenc { gpu_index: u32 },
    X264,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EncoderCapability {
    pub target: EncoderTarget,
    pub support: Result<EncoderOptions, EncoderUnavailable>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EncoderUnavailable {
    MissingFromFfmpeg,
    ProbeFailed { detail: String },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EncoderOptions {
    pub quality_mode: QualityMode,
    pub presets: Vec<SpeedPreset>,
    pub profiles: Vec<H264Profile>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum QualityMode {
    Crf,
    ConstantQuality,
    ConstantQuantizer,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Quality(u8);

impl Quality {
    pub const MIN: u8 = 18;
    pub const MAX: u8 = 28;
    pub const DEFAULT: Self = Self(22);

    pub fn new(value: u8) -> Option<Self> {
        (Self::MIN..=Self::MAX)
            .contains(&value)
            .then_some(Self(value))
    }

    pub fn value(self) -> u8 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SpeedPreset {
    Automatic,
    X264(X264Preset),
    Nvenc(NvencPreset),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum X264Preset {
    Ultrafast,
    Superfast,
    Veryfast,
    Faster,
    Fast,
    Medium,
    Slow,
    Slower,
    Veryslow,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NvencPreset {
    P1,
    P2,
    P3,
    P4,
    P5,
    P6,
    P7,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum H264Profile {
    Baseline,
    Main,
    High,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AudioMode {
    Copy,
    Aac192,
    Aac320,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Container {
    Mp4,
    Mkv,
    Mov,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OutputSettings {
    pub encoder: EncoderTarget,
    pub quality: Quality,
    pub preset: SpeedPreset,
    pub profile: H264Profile,
    pub audio: AudioMode,
    pub container: Container,
    pub destination: PathBuf,
}

#[cfg(test)]
mod tests {
    use super::Quality;

    #[test]
    fn quality_uses_the_inclusive_product_range() {
        for value in 0..=u8::MAX {
            assert_eq!(
                Quality::new(value).map(Quality::value),
                (18..=28).contains(&value).then_some(value)
            );
        }
        assert_eq!(Quality::DEFAULT.value(), 22);
    }
}
