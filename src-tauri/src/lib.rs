mod commands;
mod mqtt;
mod state;
mod video;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app_state = state::app_state::AppState::default();

    tauri::Builder::default()
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            commands::mqtt::connect_mqtt,
            commands::mqtt::disconnect_mqtt,
            commands::mqtt::emit_mock_mode_sync,
            commands::mqtt::submit_keyboard_mouse_control,
            commands::mqtt::send_zero_keyboard_mouse_control,
            commands::mqtt::get_input_diagnostics,
            commands::video::start_video,
            commands::video::stop_video,
            commands::video::start_mock_video_source,
            commands::video::stop_mock_video_source,
            commands::video::start_mock_video,
            commands::video::stop_mock_video,
            commands::video::set_video_pipeline_config,
            commands::video::start_hero_lob_video,
            commands::video::start_mock_hero_lob_h264,
            commands::video::stop_mock_hero_lob_h264,
            commands::video::reset_video_stats,
            commands::video::get_latest_frame,
            commands::video::get_decoder_mode
        ])
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
