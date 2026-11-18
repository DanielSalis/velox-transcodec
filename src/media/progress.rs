use crate::domain::ProgressSnapshot;
use std::time::Duration;

#[derive(Default)]
pub(super) struct Parser {
    out_time_us: Option<Duration>,
    legacy_time_us: Option<Duration>,
    fps: Option<f64>,
    speed: Option<f64>,
    last_time: Duration,
}

impl Parser {
    pub(super) fn line(
        &mut self,
        line: &str,
        duration: Option<Duration>,
    ) -> Option<ProgressSnapshot> {
        let (key, value) = line.split_once('=')?;
        match key {
            "out_time_us" => self.out_time_us = micros(value),
            // Apesar do nome, versões do FFmpeg emitem microssegundos neste campo.
            "out_time_ms" => self.legacy_time_us = micros(value),
            "fps" => self.fps = positive(value),
            "speed" => self.speed = positive(value.trim_end_matches(['x', ' '])),
            "progress" if matches!(value, "continue" | "end") => {
                if let Some(time) = self
                    .out_time_us
                    .take()
                    .or_else(|| self.legacy_time_us.take())
                {
                    self.last_time = self.last_time.max(time);
                }
                self.legacy_time_us = None;
                let speed = self.speed.take();
                return Some(ProgressSnapshot {
                    processed: self.last_time,
                    percent: duration.map(|total| {
                        (self.last_time.as_secs_f64() / total.as_secs_f64() * 100.0).min(99.9)
                    }),
                    eta: duration.zip(speed).and_then(|(total, speed)| {
                        Duration::try_from_secs_f64(
                            total.saturating_sub(self.last_time).as_secs_f64() / speed,
                        )
                        .ok()
                    }),
                    fps: self.fps.take(),
                    speed,
                });
            }
            _ => (),
        }
        None
    }
}

fn micros(value: &str) -> Option<Duration> {
    value.trim().parse().ok().map(Duration::from_micros)
}

fn positive(value: &str) -> Option<f64> {
    value
        .trim()
        .parse::<f64>()
        .ok()
        .filter(|number| number.is_finite() && *number > 0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_blocks_and_legacy_units_without_inventing_metrics() {
        let mut parser = Parser::default();
        let duration = Some(Duration::from_secs(4));
        assert!(parser.line("out_time_ms=2000000", duration).is_none());
        parser.line("fps=120.0", duration);
        parser.line("speed=  2x", duration);
        let progress = parser.line("progress=continue", duration).unwrap();
        assert_eq!(progress.percent, Some(50.0));
        assert_eq!(progress.fps, Some(120.0));
        assert_eq!(progress.eta, Some(Duration::from_secs(1)));
        parser.line("out_time_us=1000000", duration);
        parser.line("out_time_ms=4000000", duration);
        let progress = parser.line("progress=end", duration).unwrap();
        assert_eq!(progress.processed, Duration::from_secs(2));
        assert_eq!(progress.fps, None);
        assert_eq!(progress.eta, None);
    }
}
