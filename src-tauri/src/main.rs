// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod db;
mod udp;
mod lotw;
mod reference;  // Authoritative DXCC/prefix data (replaces cty module)
mod awards;

use std::sync::Arc;
use tauri::{Manager, Emitter};
use tokio::sync::Mutex;
use commands::AppState;
use udp::UdpListenerState;

fn main() {
    env_logger::init();

    tauri::Builder::default()
        .plugin(tauri_plugin_sql::Builder::new().build())
        .plugin(tauri_plugin_shell::init())
        .manage(AppState {
            db: Arc::new(Mutex::new(None)),
            udp_state: Arc::new(UdpListenerState::new()),
        })
        .setup(|app| {
            let app_handle = app.handle().clone();
            
            // Initialize database asynchronously
            tauri::async_runtime::spawn(async move {
                match db::init_db(&app_handle).await {
                    Ok(pool) => {
                        log::info!("Database initialized successfully");
                        
                        // Get stats to verify
                        let stats = db::get_db_stats(&pool).await.ok();
                        if let Some(ref s) = stats {
                            log::info!("Database stats: {} QSOs, {} DXCC entities, {} prefixes",
                                s.qso_count, s.entity_count, s.prefix_count);
                        }
                        
                        // Store pool in app state
                        let state = app_handle.state::<AppState>();
                        let mut db_guard = state.db.lock().await;
                        *db_guard = Some(pool);
                        drop(db_guard);
                        
                        // Notify frontend that database is ready
                        let _ = app_handle.emit("db-ready", serde_json::json!({
                            "success": true,
                            "stats": stats
                        }));
                    }
                    Err(e) => {
                        log::error!("Failed to initialize database: {}", e);
                        let _ = app_handle.emit("db-ready", serde_json::json!({
                            "success": false,
                            "error": e
                        }));
                    }
                }
            });
            
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
            commands::update_qso,
            commands::delete_qso,
            commands::import_adif,
            commands::export_adif,
            commands::add_test_qsos,
            // Callsign History & Status
            commands::get_callsign_history,
            commands::check_qso_status,
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
            // Database
            commands::is_db_ready,
            commands::get_db_stats,
        ])
        .run(tauri::generate_context!())
        .expect("error while running GoQSO");
}
