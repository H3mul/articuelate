use serde::Serialize;
use tauri::State;

use crate::cue::Cue;
use crate::engine::ActivePlayback;
use crate::state::AppState;

/// Backend telemetry snapshot for the status bar.
#[derive(Serialize)]
pub struct StatusInfo {
    pub connected: bool,
    pub device_name: String,
    pub cpu_usage: f32,
    pub dsp_usage: f32,
}

/// Return a snapshot of the full flat cue chain.
#[tauri::command]
pub fn get_cues(state: State<AppState>) -> Vec<Cue> {
    state.cues.lock().unwrap().clone()
}

/// Return engine/connection telemetry for the status bar.
#[tauri::command]
pub fn get_status(state: State<AppState>) -> StatusInfo {
    let engine = state.engine.lock().unwrap();
    StatusInfo {
        connected: engine.is_connected(),
        device_name: engine.audio_device_name().to_string(),
        cpu_usage: engine.cpu_usage(),
        dsp_usage: engine.dsp_usage(),
    }
}

/// Return the list of currently-playing audio layers for the media panel.
#[tauri::command]
pub fn get_active_playbacks(state: State<AppState>) -> Vec<ActivePlayback> {
    state.engine.lock().unwrap().active_playbacks()
}

/// Fire the next standby cue (transport GO).
#[tauri::command]
pub fn go(state: State<AppState>) {
    state.engine.lock().unwrap().fire_next();
}

/// Step back to the previous cue (TODO: real implementation).
#[tauri::command]
pub fn back(_state: State<AppState>) {
    // TODO: step back to the previous cue.
}

/// Pause the active playback (TODO: real implementation).
#[tauri::command]
pub fn pause(_state: State<AppState>) {
    // TODO: pause the active playback.
}

/// Stop all audio immediately (panic / stop all).
#[tauri::command]
pub fn panic_stop(state: State<AppState>) {
    state.engine.lock().unwrap().stop_all();
}
