#![allow(dead_code)]

use crate::model::CueId;
use async_trait::async_trait;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DriverCapabilities {
    pub supports_matrix_routing: bool,
    pub supports_hot_swapping: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum AudioDriverError {
    #[error("Matrix routing is not supported by the active audio driver")]
    MatrixRoutingUnsupported,
    #[error("Device switch failed: {0}")]
    DeviceError(String),
    #[error("Audio playback error: {0}")]
    PlaybackError(String),
}

#[async_trait(?Send)]
pub trait AudioDriver {
    fn capabilities(&self) -> DriverCapabilities;
    fn output_devices(&self) -> Vec<String>;
    async fn set_device(&mut self, device_name: String) -> Result<(), AudioDriverError>;
    async fn play_cue(
        &mut self,
        cue_id: CueId,
        file_path: PathBuf,
        volume_db: f32,
        looping: bool,
        matrix_routing: Option<Vec<Vec<f32>>>,
    ) -> Result<(), AudioDriverError>;
    async fn seek_cue(&mut self, cue_id: CueId, position_sec: f32) -> Result<(), AudioDriverError>;
    async fn stop_cue(&mut self, cue_id: CueId) -> Result<(), AudioDriverError>;
}
