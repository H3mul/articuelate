/// Stub audio engine that will later wrap Kira.
pub struct AudioEngine {
    device_name: String,
    cpu_usage: f32,
    dsp_usage: f32,
    connected: bool,
}

impl AudioEngine {
    pub fn new() -> Self {
        Self {
            device_name: "USB Audio Device".into(),
            cpu_usage: 4.0,
            dsp_usage: 12.0,
            connected: true,
        }
    }

    pub fn audio_device_name(&self) -> &str {
        &self.device_name
    }

    pub fn is_connected(&self) -> bool {
        self.connected
    }

    pub fn cpu_usage(&self) -> f32 {
        self.cpu_usage
    }

    pub fn dsp_usage(&self) -> f32 {
        self.dsp_usage
    }

    /// Fire the next standby cue.
    pub fn fire_next(&mut self) {
        // TODO: real Kira playback
    }

    /// Stop all audio immediately.
    pub fn stop_all(&mut self) {
        // TODO: real Kira stop
    }

    /// Return a list of currently-playing audio entries for the media panel.
    pub fn active_playbacks(&self) -> Vec<ActivePlayback> {
        // Placeholder: return dummy data
        vec![
            ActivePlayback {
                cue_number: 1.0,
                label: "Wind_Loop.wav".into(),
                volume_db: -12.0,
                progress: 0.65,
            },
            ActivePlayback {
                cue_number: 1.0,
                label: "Rain_Heavy.wav".into(),
                volume_db: -8.0,
                progress: 0.42,
            },
        ]
    }
}

/// A single active audio layer shown in the media panel.
pub struct ActivePlayback {
    pub cue_number: f64,
    pub label: String,
    pub volume_db: f32,
    pub progress: f32,
}