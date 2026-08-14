use crate::config::{get, set};
use crate::window::updater_window;
use log::{info, warn};
use tauri_plugin_updater::UpdaterExt;

pub fn check_update(app_handle: tauri::AppHandle) {
    let enable = match get("check_update") {
        Some(v) => v.as_bool().unwrap(),
        None => {
            set("check_update", true);
            true
        }
    };
    if enable {
        tauri::async_runtime::spawn(async move {
            let updater = match app_handle.updater() {
                Ok(u) => u,
                Err(e) => {
                    warn!("Failed to init updater: {}", e);
                    return;
                }
            };
            match updater.check().await {
                Ok(update) => {
                    if update.is_some() {
                        info!("New version available");
                        updater_window();
                    }
                }
                Err(e) => {
                    warn!("Failed to check update: {}", e);
                }
            }
        });
    }
}
