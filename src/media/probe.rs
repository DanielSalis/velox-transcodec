use std::{num::NonZeroU32, path::Path, time::Duration};

use serde::Deserialize;

use super::{MediaEngine, failure, local_file, process};
use crate::domain::{
    AudioStream, FrameRate, JobFailure, JobFailureKind, MediaId, MediaInfo, VideoStream,
};

#[derive(Deserialize)]
struct Probe {
    #[serde(default)]
    streams: Vec<Stream>,
    #[serde(default)]
    format: Format,
}

#[derive(Default, Deserialize)]
struct Format {
    duration: Option<String>,
    bit_rate: Option<String>,
}

#[derive(Deserialize)]
struct Stream {
    index: u32,
    codec_type: Option<String>,
    codec_name: Option<String>,
    width: Option<u32>,
    height: Option<u32>,
    avg_frame_rate: Option<String>,
    r_frame_rate: Option<String>,
    bit_rate: Option<String>,
    duration: Option<String>,
    #[serde(default)]
    disposition: Disposition,
}

#[derive(Default, Deserialize)]
struct Disposition {
    #[serde(default)]
    attached_pic: u8,
}

impl MediaEngine {
    pub async fn probe(&self, id: MediaId, path: &Path) -> Result<MediaInfo, JobFailure> {
        let (path, source_bytes) = local_file(path).await?;
        let output = process::capture(
            process::command(&self.ffprobe).args([
                "-v", "error", "-protocol_whitelist", "file", "-of", "json",
                "-show_entries", "stream=index,codec_type,codec_name,width,height,avg_frame_rate,r_frame_rate,bit_rate,duration:stream_disposition=attached_pic:format=duration,bit_rate",
                "-i",
            ]).arg(&path),
            20,
            1024 * 1024,
            JobFailureKind::InvalidMedia,
        ).await?;
        if !output.status.success() {
            return Err(failure(
                JobFailureKind::InvalidMedia,
                process::diagnostic_or_status(output.status, output.stderr),
            ));
        }
        let probe = serde_json::from_slice(&output.stdout)
            .map_err(|err| failure(JobFailureKind::InvalidMedia, err))?;
        decode(probe, id, path, source_bytes)
    }
}

fn positive_bitrate(value: Option<&str>) -> Option<u64> {
    value?.parse().ok().filter(|rate| *rate > 0)
}

fn duration(value: Option<&str>) -> Option<Duration> {
    Duration::try_from_secs_f64(value?.parse().ok()?)
        .ok()
        .filter(|t| !t.is_zero())
}

fn frame_rate(value: Option<&str>) -> Option<FrameRate> {
    let (numerator, denominator) = value?.split_once('/')?;
    Some(FrameRate {
        numerator: numerator.parse::<NonZeroU32>().ok()?,
        denominator: denominator.parse::<NonZeroU32>().ok()?,
    })
}

fn decode(
    probe: Probe,
    id: MediaId,
    path: std::path::PathBuf,
    source_bytes: u64,
) -> Result<MediaInfo, JobFailure> {
    let stream = probe
        .streams
        .iter()
        .find(|stream| {
            stream.codec_type.as_deref() == Some("video")
                && stream.disposition.attached_pic == 0
                && stream.width.is_some_and(|width| width > 0)
                && stream.height.is_some_and(|height| height > 0)
        })
        .ok_or_else(|| {
            failure(
                JobFailureKind::InvalidMedia,
                "Nenhuma trilha de vídeo utilizável.",
            )
        })?;
    let duration =
        duration(probe.format.duration.as_deref()).or_else(|| duration(stream.duration.as_deref()));
    let video = VideoStream {
        index: stream.index,
        codec: stream
            .codec_name
            .clone()
            .unwrap_or_else(|| "unknown".into()),
        width: stream.width.unwrap_or_default(),
        height: stream.height.unwrap_or_default(),
        frame_rate: frame_rate(stream.avg_frame_rate.as_deref())
            .or_else(|| frame_rate(stream.r_frame_rate.as_deref())),
        bit_rate: positive_bitrate(stream.bit_rate.as_deref()),
    };
    let audio = probe
        .streams
        .into_iter()
        .filter(|stream| stream.codec_type.as_deref() == Some("audio"))
        .map(|stream| AudioStream {
            index: stream.index,
            codec: stream.codec_name.unwrap_or_else(|| "unknown".into()),
        })
        .collect();
    Ok(MediaInfo {
        id,
        path,
        video,
        audio,
        duration,
        source_bytes,
        container_bit_rate: positive_bitrate(probe.format.bit_rate.as_deref()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skips_attached_cover_and_preserves_unknown_metadata() {
        let probe = serde_json::from_str(r#"{
            "streams": [
                {"index":0,"codec_type":"video","width":10,"height":10,"disposition":{"attached_pic":1}},
                {"index":1,"codec_type":"video","codec_name":"vp9","width":1920,"height":1080,"avg_frame_rate":"0/0","duration":"N/A"},
                {"index":2,"codec_type":"audio","codec_name":"opus"}
            ], "format":{"duration":"N/A","bit_rate":"N/A"}
        }"#).unwrap();
        let info = decode(probe, MediaId(1), "/test.mkv".into(), 200).unwrap();
        assert_eq!(info.video.index, 1);
        assert_eq!(info.audio[0].index, 2);
        assert!(
            info.duration.is_none()
                && info.video.bit_rate.is_none()
                && info.video.frame_rate.is_none()
                && info.container_bit_rate.is_none()
        );
    }
}
