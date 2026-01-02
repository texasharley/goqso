// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod db;
mod udp;
mod lotw;
mod cty;
mod awards;

use tauri::Manager;

fn main() {
    env_logger::init();

    tauri::Builder::default()
        .plugin(tauri_plugin_sql::Builder::new().build())
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            #[cfg(debug_assertions)]
            {
                let window = app.get_webview_window("main").unwrap();
                window.open_devtools();
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // UDP Listener
            commands::start_udp_listener,
            commands::stop_udp_listener,
            commands::get_udp_status,
            // QSO Operations
            commands::get_qsos,
            commands::add_qso,
            commands::delete_qso,
            commands::import_adif,
            commands::export_adif,
            // LoTW Sync
            commands::sync_lotw_upload,
            commands::sync_lotw_download,
            commands::get_sync_status,
            commands::detect_tqsl_path,
            // Awards
            commands::get_dxcc_progress,
            commands::get_was_progress,
            commands::get_vucc_progress,
            // CTY Lookup
            commands::lookup_callsign,
            // Settings
            commands::get_setting,
            commands::set_setting,
        ])
        .run(tauri::generate_context!())
        .expect("error while running GoQSO");
}
