mod commands;
mod state;

use state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![
            commands::get_cues,
            commands::get_status,
            commands::get_active_playbacks,
            commands::go,
            commands::back,
            commands::pause,
            commands::panic_stop,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
