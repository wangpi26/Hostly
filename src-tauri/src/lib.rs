mod hosts;
pub mod storage;
pub mod cli;
pub mod elevation;

#[cfg(target_os = "windows")]
use window_vibrancy::apply_mica;
use tauri::Manager;


#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            // Check CLI args
            if cli::run_cli(Some(&app.handle())) {
                std::process::exit(0);
            }

            let window = app.get_webview_window("main").unwrap();

            #[cfg(target_os = "windows")]
            {
                let _ = apply_mica(&window, Some(true));
            }
            
            // Apply Window Settings
            let ctx = storage::Context::Tauri(&app.handle());
            if let Ok(config) = storage::load_config_internal(&ctx) {
                if let (Some(w), Some(h)) = (config.window_width, config.window_height) {
                    if w > 0.0 && h > 0.0 {
                        let _ = window.set_size(tauri::Size::Logical(tauri::LogicalSize { width: w, height: h }));
                    }
                }
            }
            
            // Background Scheduler
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                // Wait for app to startup
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                loop {
                    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                    storage::check_auto_updates(&handle);
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            hosts::get_system_hosts,
            hosts::save_system_hosts,
            hosts::check_write_permission,
            hosts::hostly_open_url,
            storage::load_config,
            storage::load_common_config,
            storage::save_common_config,
            storage::list_profiles,
            storage::create_profile,
            storage::save_profile_content,
            storage::delete_profile,
            storage::rename_profile,
            storage::toggle_profile_active,
            storage::set_multi_select,
            storage::apply_config,
            storage::import_file,
            storage::export_file,
            storage::import_data,
            storage::export_data,
            storage::import_switchhosts,
            storage::update_remote_config,
            storage::trigger_profile_update,
            storage::set_theme,
            storage::save_window_config,
            storage::save_sidebar_config,
        ])
        .build(tauri::generate_context!())
        .expect("构建 Tauri 应用失败")
        .run(|app, event| {
            let mut should_show = matches!(&event, tauri::RunEvent::Ready);
            #[cfg(target_os = "macos")]
            {
                should_show |= matches!(&event, tauri::RunEvent::Reopen { has_visible_windows: false, .. });
            }
            if should_show {
                let window = app.get_webview_window("main").or_else(|| {
                    let config = app.config().app.windows.first()?;
                    tauri::WebviewWindowBuilder::from_config(app, config).ok()?.build().ok()
                });
                if let Some(window) = window {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
        });
}
