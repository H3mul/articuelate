use crate::audio::AtomicCueMetrics;
use crate::model::CueId;
use rodio::Source;
use std::sync::Arc;
use std::time::Duration;

pub struct TelemetrySource<S> {
    inner: S,
    metrics: Arc<AtomicCueMetrics>,

    channels: u16,
    sample_rate: u32,
    samples_processed: usize,
    accumulated_left_peak: f32,
    accumulated_right_peak: f32,
    total_samples_played: u64,
}

impl<S> TelemetrySource<S>
where
    S: Source<Item = f32>,
{
    pub fn new(inner: S, _cue_id: CueId, metrics: Arc<AtomicCueMetrics>) -> Self {
        let channels = inner.channels();
        let sample_rate = inner.sample_rate();

        Self {
            inner,
            metrics,
            channels,
            sample_rate,
            samples_processed: 0,
            accumulated_left_peak: 0.0,
            accumulated_right_peak: 0.0,
            total_samples_played: 0,
        }
    }

    fn push_telemetry(&mut self) {
        let frame_count = self.total_samples_played / (self.channels as u64);
        let position_sec = frame_count as f32 / self.sample_rate as f32;

        self.metrics.update(
            position_sec as f64,
            self.inner
                .total_duration()
                .map(|duration| duration.as_secs_f64())
                .unwrap_or(0.0),
            self.accumulated_left_peak,
            self.accumulated_right_peak,
        );

        // Reset accumulation metrics for the next 16ms window
        self.accumulated_left_peak = 0.0;
        self.accumulated_right_peak = 0.0;
        self.samples_processed = 0;
    }
}

impl<S> Iterator for TelemetrySource<S>
where
    S: Source<Item = f32>,
{
    type Item = f32;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let sample = self.inner.next()?;
        let abs_sample = sample.abs();

        // 1. Peak Metering across channels
        if self.channels == 1 {
            self.accumulated_left_peak = self.accumulated_left_peak.max(abs_sample);
            self.accumulated_right_peak = self.accumulated_left_peak;
        } else if self.samples_processed % (self.channels as usize) == 0 {
            self.accumulated_left_peak = self.accumulated_left_peak.max(abs_sample);
        } else {
            self.accumulated_right_peak = self.accumulated_right_peak.max(abs_sample);
        }

        self.samples_processed += 1;
        self.total_samples_played += 1;

        // 2. Emit telemetry packet roughly every 60Hz (~800 samples at 48kHz stereo)
        let telemetry_interval_samples =
            ((self.sample_rate as usize * self.channels as usize) / 60).max(1);
        if self.samples_processed >= telemetry_interval_samples {
            self.push_telemetry();
        }

        Some(sample)
    }
}

impl<S> Source for TelemetrySource<S>
where
    S: Source<Item = f32>,
{
    fn current_frame_len(&self) -> Option<usize> {
        self.inner.current_frame_len()
    }

    fn channels(&self) -> u16 {
        self.channels
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn total_duration(&self) -> Option<Duration> {
        self.inner.total_duration()
    }
}
