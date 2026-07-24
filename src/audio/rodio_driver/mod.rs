mod telemetry_source;

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use async_trait::async_trait;
use cpal::traits::{DeviceTrait, HostTrait};

use rodio::Source;

use crate::audio::AudioEvent;
use crate::audio::{AudioDriver, AudioDriverError, AudioTelemetry, DriverCapabilities};
use crate::model::CueId;

use telemetry_source::TelemetrySource;

pub struct RodioDriver {
    _stream: Option<rodio::OutputStream>,
    stream_handle: Option<rodio::OutputStreamHandle>,
    sinks: HashMap<CueId, rodio::Sink>,
    telemetry: Arc<AudioTelemetry>,
    audio_event_tx: tokio::sync::mpsc::Sender<AudioEvent>,
}

impl RodioDriver {
    pub fn new(
        telemetry: Arc<AudioTelemetry>,
        audio_event_tx: tokio::sync::mpsc::Sender<AudioEvent>,
    ) -> Self {
        let (stream, stream_handle) = rodio::OutputStream::try_default().ok().unzip();
        Self {
            _stream: stream,
            stream_handle,
            sinks: HashMap::new(),
            telemetry,
            audio_event_tx,
        }
    }

    fn create_default_stream()
    -> Result<(rodio::OutputStream, rodio::OutputStreamHandle, String), AudioDriverError> {
        let (stream, handle) = rodio::OutputStream::try_default().map_err(|e| {
            AudioDriverError::DeviceError(format!("Failed to open default output stream: {e}"))
        })?;
        let name = cpal::default_host()
            .default_output_device()
            .and_then(|d| d.name().ok())
            .unwrap_or_else(|| "default".into());
        Ok((stream, handle, name))
    }

    fn create_stream_for_device(
        device_name: &str,
    ) -> Result<(rodio::OutputStream, rodio::OutputStreamHandle, String), AudioDriverError> {
        let host = cpal::default_host();
        let device = host
            .output_devices()
            .map_err(|e| {
                AudioDriverError::DeviceError(format!("Failed to enumerate output devices: {e}"))
            })?
            .find(|d| d.name().map(|name| name == device_name).unwrap_or(false))
            .ok_or_else(|| {
                AudioDriverError::DeviceError(format!("Output device not found: {device_name}"))
            })?;
        let name = device.name().map_err(|e| {
            AudioDriverError::DeviceError(format!("Failed to get device name: {e}"))
        })?;
        let supported = device.default_output_config().map_err(|e| {
            AudioDriverError::DeviceError(format!("Failed to get output config: {e}"))
        })?;
        let (stream, handle) =
            rodio::OutputStream::try_from_device_config(&device, supported.into()).map_err(
                |e| AudioDriverError::DeviceError(format!("Failed to create output stream: {e}")),
            )?;
        Ok((stream, handle, name))
    }
}

#[async_trait(?Send)]
impl AudioDriver for RodioDriver {
    fn capabilities(&self) -> DriverCapabilities {
        DriverCapabilities {
            supports_matrix_routing: false,
            supports_hot_swapping: true,
        }
    }

    fn output_devices(&self) -> Vec<String> {
        cpal::default_host()
            .output_devices()
            .into_iter()
            .flatten()
            .filter_map(|d| d.name().ok())
            .collect()
    }

    async fn set_device(&mut self, device_name: String) -> Result<(), AudioDriverError> {
        eprintln!("[RodioDriver] Switching audio device to: {device_name}");

        // 1. Stop all active sinks (old hardware endpoint closing)
        for (cue_id, sink) in self.sinks.drain() {
            sink.stop();
            self.telemetry.remove(cue_id);
        }

        // 2. Drop old stream and handle
        self._stream = None;
        self.stream_handle = None;

        // 3. Build new stream for target device
        match Self::create_stream_for_device(&device_name) {
            Ok((new_stream, new_handle, _resolved_name)) => {
                self._stream = Some(new_stream);
                self.stream_handle = Some(new_handle);
                Ok(())
            }
            Err(err) => {
                let err_msg = format!("Failed to switch to device '{device_name}': {err}");

                // Notify execution engine of device failure
                let _ = self.audio_event_tx.try_send(AudioEvent::DeviceLost {
                    device_name: Some(device_name),
                    error_message: err_msg.clone(),
                });

                // Fall back to default stream so driver remains functional
                if let Ok((def_stream, def_handle, _def_name)) = Self::create_default_stream() {
                    self._stream = Some(def_stream);
                    self.stream_handle = Some(def_handle);
                }

                Err(AudioDriverError::DeviceError(err_msg))
            }
        }
    }

    async fn play_cue(
        &mut self,
        cue_id: CueId,
        file_path: PathBuf,
        volume_db: f32,
        looping: bool,
        _matrix: Option<Vec<Vec<f32>>>,
    ) -> Result<(), AudioDriverError> {
        let handle = self
            .stream_handle
            .as_ref()
            .ok_or_else(|| AudioDriverError::PlaybackError("No output device available".into()))?;
        if let Some(old) = self.sinks.remove(&cue_id) {
            old.stop();
        }

        let sink = rodio::Sink::try_new(handle)
            .map_err(|e| AudioDriverError::PlaybackError(e.to_string()))?;
        let file = std::fs::File::open(file_path)
            .map_err(|e| AudioDriverError::PlaybackError(e.to_string()))?;
        let decoder = rodio::Decoder::new(std::io::BufReader::new(file))
            .map_err(|e| AudioDriverError::PlaybackError(e.to_string()))?;
        let metrics = self.telemetry.metrics_for(cue_id);
        let telemetry_source = TelemetrySource::new(decoder.convert_samples(), cue_id, metrics);
        if looping {
            sink.append(telemetry_source.repeat_infinite());
        } else {
            sink.append(telemetry_source);
        }
        sink.set_volume(10.0_f32.powf(volume_db / 20.0));
        self.sinks.insert(cue_id, sink);
        Ok(())
    }

    async fn seek_cue(&mut self, cue_id: CueId, position_sec: f32) -> Result<(), AudioDriverError> {
        if let Some(sink) = self.sinks.get(&cue_id) {
            sink.try_seek(std::time::Duration::from_secs_f32(position_sec))
                .map_err(|e| AudioDriverError::PlaybackError(e.to_string()))?;
        }
        Ok(())
    }

    async fn stop_cue(&mut self, cue_id: CueId) -> Result<(), AudioDriverError> {
        if let Some(sink) = self.sinks.remove(&cue_id) {
            sink.stop();
        }
        Ok(())
    }
}
