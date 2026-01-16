// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod adif;
mod awards;
mod commands;
mod db;
mod fcc;
mod lotw;
mod qso_tracker;
mod reference;  // Authoritative DXCC/prefix data (replaces cty module)
mod udp;

use std::sync::Arc;
use tauri::{Manager, Emitter};
use tokio::sync::Mutex;
use commands::AppState;
use udp::UdpListenerState;

fn main() {
    // Initialize logging - default to info level for our crate
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("goqso=info,goqso::udp=debug")
    ).init();

    tauri::Builder::default()
        .plugin(tauri_plugin_sql::Builder::new().build())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
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
                        
                        // Start background FCC database sync
                        let app_handle_fcc = app_handle.clone();
                        tauri::async_runtime::spawn(async move {
                            fcc::sync_fcc_if_needed(&app_handle_fcc).await;
                        });
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
            commands::udp::start_udp_listener,
            commands::udp::stop_udp_listener,
            commands::udp::call_station,
            commands::udp::get_udp_status,
            // QSO Operations
            commands::qso::get_qsos,
            commands::qso::add_qso,
            commands::qso::update_qso,
            commands::qso::delete_qso,
            commands::qso::remove_duplicate_qsos,
            commands::qso::clear_all_qsos,
            commands::qso::add_test_qsos,
            // ADIF Import/Export
            commands::adif::import_adif,
            commands::adif::export_adif,
            // Callsign History & Status
            commands::qso::get_callsign_history,
            commands::qso::check_qso_status,
            // QSO Data Repair
            commands::qso::repair_qso_data,
            // LoTW Integration
            commands::adif::import_lotw_confirmations,
            commands::lotw::get_sync_status,
            commands::lotw::sync_lotw_download,
            commands::lotw::detect_tqsl_path,
            commands::lotw::upload_to_lotw,
            // Awards Progress
            commands::awards::get_dxcc_progress,
            commands::awards::get_was_progress,
            // CTY Lookup
            commands::settings::lookup_callsign,
            // Settings
            commands::settings::get_setting,
            commands::settings::set_setting,
            // Database
            commands::settings::is_db_ready,
            commands::settings::get_db_stats,
            // Band Activity
            commands::band_activity::get_recent_activity,
            commands::band_activity::prune_band_activity,
            // FCC Database
            commands::fcc::get_fcc_sync_status,
            commands::fcc::sync_fcc_database,
            commands::fcc::lookup_fcc_callsign,
            commands::fcc::lookup_fcc_callsigns,
            // Diagnostics
            commands::diagnostics::get_qso_diagnostics,
        ])
        .run(tauri::generate_context!())
        .expect("error while running GoQSO");
}
