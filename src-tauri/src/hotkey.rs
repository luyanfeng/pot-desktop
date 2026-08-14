use crate::config::{get, set};
use crate::window::{input_translate, ocr_recognize, ocr_translate, selection_translate};
use crate::APP;
use log::{info, warn};
use tauri::AppHandle;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

fn get_hotkey_config(name: &str) -> String {
    match get(name) {
        Some(v) => v.as_str().unwrap().to_string(),
        None => {
            set(name, "");
            String::new()
        }
    }
}

fn register(app_handle: &AppHandle, name: &str, key: &str) -> Result<(), String> {
    let hotkey = if key.is_empty() {
        get_hotkey_config(name)
    } else {
        key.to_string()
    };

    if !hotkey.is_empty() {
        match app_handle.global_shortcut().register(hotkey.as_str()) {
            Ok(()) => {
                info!("Registered global shortcut: {} for {}", hotkey, name);
            }
            Err(e) => {
                warn!("Failed to register global shortcut: {} {:?}", hotkey, e);
                return Err(e.to_string());
            }
        };
    }
    Ok(())
}

// Register global shortcuts
pub fn register_shortcut(shortcut: &str) -> Result<(), String> {
    let app_handle = APP.get().unwrap();
    match shortcut {
        "hotkey_selection_translate" => register(
            app_handle,
            "hotkey_selection_translate",
            "",
        )?,
        "hotkey_input_translate" => {
            register(app_handle, "hotkey_input_translate", "")?
        }
        "hotkey_ocr_recognize" => register(app_handle, "hotkey_ocr_recognize", "")?,
        "hotkey_ocr_translate" => register(app_handle, "hotkey_ocr_translate", "")?,
        "all" => {
            register(
                app_handle,
                "hotkey_selection_translate",
                "",
            )?;
            register(app_handle, "hotkey_input_translate", "")?;
            register(app_handle, "hotkey_ocr_recognize", "")?;
            register(app_handle, "hotkey_ocr_translate", "")?;
        }
        _ => {}
    }
    Ok(())
}

#[tauri::command]
pub fn register_shortcut_by_frontend(name: &str, shortcut: &str) -> Result<(), String> {
    let app_handle = APP.get().unwrap();
    match name {
        "hotkey_selection_translate" => {
            register(app_handle, "hotkey_selection_translate", shortcut)?
        }
        "hotkey_input_translate" => {
            register(app_handle, "hotkey_input_translate", shortcut)?
        }
        "hotkey_ocr_recognize" => {
            register(app_handle, "hotkey_ocr_recognize", shortcut)?
        }
        "hotkey_ocr_translate" => {
            register(app_handle, "hotkey_ocr_translate", shortcut)?
        }
        _ => {}
    }
    Ok(())
}

// Dispatch global shortcut events by matching the shortcut string
pub fn handle_shortcut_event(app: &AppHandle, shortcut: &Shortcut, event: tauri_plugin_global_shortcut::ShortcutEvent) {
    if event.state() != ShortcutState::Pressed {
        return;
    }
    // Resolve the config key name for the pressed shortcut by comparing with stored configs
    // Note: 配置字符串(Ctrl+6)与事件 Shortcut 的 Display(control+Digit6)格式不同,
    // 因此将配置字符串解析为 Shortcut 后按值比较
    let name = [
        "hotkey_selection_translate",
        "hotkey_input_translate",
        "hotkey_ocr_recognize",
        "hotkey_ocr_translate",
    ]
    .iter()
    .find(|name| {
        let config_key = get_hotkey_config(name);
        if config_key.is_empty() {
            return false;
        }
        match Shortcut::try_from(config_key.as_str()) {
            Ok(configured) => configured == *shortcut,
            Err(_) => false,
        }
    })
    .copied();

    match name {
        Some("hotkey_selection_translate") => selection_translate(),
        Some("hotkey_input_translate") => input_translate(),
        Some("hotkey_ocr_recognize") => ocr_recognize(),
        Some("hotkey_ocr_translate") => ocr_translate(),
        _ => {
            let _ = app;
            warn!("Global shortcut event for unknown key: {}", shortcut);
        }
    }
}
