use serde::Serialize;

/// Stub audio engine that will later wrap Kira.
pub struct AudioEngine {
    device_name: String,
    cpu_usage: f32,
    dsp_usage: f32,
    connected: bool,
    playbacks: Vec<ActivePlayback>,
}

impl AudioEngine {
    pub fn new() -> Self {
        Self {
            device_name: "USB Audio Device".into(),
            cpu_usage: 4.0,
            dsp_usage: 12.0,
            connected: true,
            playbacks: Vec::new(),
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
        if self.playbacks.is_empty() {
            self.playbacks = vec![
                ActivePlayback {
                    cue_number: 1.0,
                    label: "Wind_Loop.wav".into(),
                    volume_db: -12.0,
                    progress: 0.0,
                },
                ActivePlayback {
                    cue_number: 1.0,
                    label: "Rain_Heavy.wav".into(),
                    volume_db: -8.0,
                    progress: 0.0,
                },
            ];
        } else {
            // Advance the mock progress so the media panel feels alive.
            for pb in &mut self.playbacks {
                pb.progress = (pb.progress + 0.1).min(1.0);
            }
        }
    }

    /// Stop all audio immediately.
    pub fn stop_all(&mut self) {
        self.playbacks.clear();
    }

    /// Return a list of currently-playing audio entries for the media panel.
    pub fn active_playbacks(&self) -> Vec<ActivePlayback> {
        self.playbacks.clone()
    }
}

/// A single active audio layer shown in the media panel.
#[derive(Debug, Clone, Serialize)]
pub struct ActivePlayback {
    pub cue_number: f64,
    pub label: String,
    pub volume_db: f32,
    pub progress: f32,
}
