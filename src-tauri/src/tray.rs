use crate::clipboard::*;
use crate::config::{get, set};
use crate::window::config_window;
use crate::window::input_translate;
use crate::window::ocr_recognize;
use crate::window::ocr_translate;
use crate::window::updater_window;
use crate::APP;
use log::info;
use once_cell::sync::OnceCell;
use tauri::menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem, Submenu};
use tauri::tray::{TrayIcon, TrayIconBuilder, TrayIconEvent};
use tauri::Emitter;
use tauri::{AppHandle, Manager, Wry};

// Tray icon and dynamic menu items (must be held to avoid being dropped)
static TRAY_ICON: OnceCell<TrayIcon<Wry>> = OnceCell::new();
static CLIPBOARD_MONITOR_ITEM: OnceCell<CheckMenuItem<Wry>> = OnceCell::new();
static COPY_SOURCE_ITEM: OnceCell<CheckMenuItem<Wry>> = OnceCell::new();
static COPY_TARGET_ITEM: OnceCell<CheckMenuItem<Wry>> = OnceCell::new();
static COPY_SOURCE_TARGET_ITEM: OnceCell<CheckMenuItem<Wry>> = OnceCell::new();
static COPY_DISABLE_ITEM: OnceCell<CheckMenuItem<Wry>> = OnceCell::new();

pub fn init_tray(app: &AppHandle) -> tauri::Result<()> {
    let mut builder = TrayIconBuilder::new()
        .menu(&tray_menu(&app))
        .show_menu_on_left_click(false)
        .on_menu_event(tray_menu_event_handler)
        .on_tray_icon_event(tray_icon_event_handler);
    if let Some(icon) = app.default_window_icon() {
        builder = builder.icon(icon.clone());
    }
    let tray = builder.build(app)?;
    let _ = TRAY_ICON.set(tray);
    Ok(())
}

#[tauri::command]
pub fn update_tray(app_handle: tauri::AppHandle, mut language: String, mut copy_mode: String) {
    if language.is_empty() {
        language = match get("app_language") {
            Some(v) => v.as_str().unwrap().to_string(),
            None => {
                set("app_language", "en");
                "en".to_string()
            }
        };
    }
    if copy_mode.is_empty() {
        copy_mode = match get("translate_auto_copy") {
            Some(v) => v.as_str().unwrap().to_string(),
            None => {
                set("translate_auto_copy", "disable");
                "disable".to_string()
            }
        };
    }

    info!(
        "Update tray with language: {}, copy mode: {}",
        language, copy_mode
    );
    let menu = match language.as_str() {
        "en" => tray_menu_en(),
        "zh_cn" => tray_menu_zh_cn(),
        "zh_tw" => tray_menu_zh_tw(),
        "ja" => tray_menu_ja(),
        "ko" => tray_menu_ko(),
        "fr" => tray_menu_fr(),
        "de" => tray_menu_de(),
        "ru" => tray_menu_ru(),
        "pt_br" => tray_menu_pt_br(),
        "fa" => tray_menu_fa(),
        "uk" => tray_menu_uk(),
        _ => tray_menu_en(),
    };
    if let Some(tray) = TRAY_ICON.get() {
        let _ = tray.set_menu(Some(menu));
    }
    #[cfg(not(target_os = "linux"))]
    if let Some(tray) = TRAY_ICON.get() {
        let _ = tray.set_tooltip(Some(format!("pot {}", app_handle.package_info().version)));
    }

    let enable_clipboard_monitor = match get("clipboard_monitor") {
        Some(v) => v.as_bool().unwrap(),
        None => {
            set("clipboard_monitor", false);
            false
        }
    };
    if let Some(item) = CLIPBOARD_MONITOR_ITEM.get() {
        let _ = item.set_checked(enable_clipboard_monitor);
    }

    match copy_mode.as_str() {
        "source" => {
            if let Some(item) = COPY_SOURCE_ITEM.get() {
                let _ = item.set_checked(true);
            }
        }
        "target" => {
            if let Some(item) = COPY_TARGET_ITEM.get() {
                let _ = item.set_checked(true);
            }
        }
        "source_target" => {
            if let Some(item) = COPY_SOURCE_TARGET_ITEM.get() {
                let _ = item.set_checked(true);
            }
        }
        "disable" => {
            if let Some(item) = COPY_DISABLE_ITEM.get() {
                let _ = item.set_checked(true);
            }
        }
        _ => {}
    }
    let _ = &app_handle;
}

fn tray_menu_event_handler(app: &AppHandle, event: tauri::menu::MenuEvent) {
    match event.id().as_ref() {
        "input_translate" => on_input_translate_click(),
        "copy_source" => on_auto_copy_click(app, "source"),
        "clipboard_monitor" => on_clipboard_monitor_click(app),
        "copy_target" => on_auto_copy_click(app, "target"),
        "copy_source_target" => on_auto_copy_click(app, "source_target"),
        "copy_disable" => on_auto_copy_click(app, "disable"),
        "ocr_recognize" => on_ocr_recognize_click(),
        "ocr_translate" => on_ocr_translate_click(),
        "config" => on_config_click(),
        "check_update" => on_check_update_click(),
        "view_log" => on_view_log_click(app),
        "restart" => on_restart_click(app),
        "quit" => on_quit_click(app),
        _ => {}
    }
}

fn tray_icon_event_handler(tray: &TrayIcon<Wry>, _event: TrayIconEvent) {
    #[cfg(target_os = "windows")]
    {
        use tauri::tray::{MouseButton, MouseButtonState};
        if let TrayIconEvent::Click {
            button: MouseButton::Left,
            button_state: MouseButtonState::Up,
            ..
        } = _event
        {
            on_tray_click();
        }
    }
    let _ = tray;
}

#[cfg(target_os = "windows")]
fn on_tray_click() {
    let event = match get("tray_click_event") {
        Some(v) => v.as_str().unwrap().to_string(),
        None => {
            set("tray_click_event", "config");
            "config".to_string()
        }
    };
    match event.as_str() {
        "config" => config_window(),
        "translate" => input_translate(),
        "ocr_recognize" => ocr_recognize(),
        "ocr_translate" => ocr_translate(),
        "disable" => {}
        _ => config_window(),
    }
}
fn on_input_translate_click() {
    input_translate();
}
fn on_clipboard_monitor_click(app: &AppHandle) {
    let enable_clipboard_monitor = match get("clipboard_monitor") {
        Some(v) => v.as_bool().unwrap(),
        None => {
            set("clipboard_monitor", false);
            false
        }
    };
    let current = !enable_clipboard_monitor;
    // Update Config File
    set("clipboard_monitor", current);
    // Update State and Start Monitor
    let state = app.state::<ClipboardMonitorEnableWrapper>();
    state
        .0
        .lock()
        .unwrap()
        .replace_range(.., &current.to_string());
    if current {
        start_clipboard_monitor(app.clone());
    }
    // Update Tray Menu Status
    if let Some(item) = CLIPBOARD_MONITOR_ITEM.get() {
        let _ = item.set_checked(current);
    }
}
fn on_auto_copy_click(app: &AppHandle, mode: &str) {
    info!("Set copy mode to: {}", mode);
    set("translate_auto_copy", mode);
    let _ = app.emit("translate_auto_copy_changed", mode);
    update_tray(app.clone(), "".to_string(), mode.to_string());
}
fn on_ocr_recognize_click() {
    ocr_recognize();
}
fn on_ocr_translate_click() {
    ocr_translate();
}

fn on_config_click() {
    config_window();
}

fn on_check_update_click() {
    updater_window();
}
fn on_view_log_click(app: &AppHandle) {
    use tauri_plugin_opener::OpenerExt;
    let log_path = app.path().app_log_dir().unwrap();
    let _ = app.opener().open_path(log_path.to_str().unwrap(), None::<&str>);
}
fn on_restart_click(app: &AppHandle) {
    info!("============== Restart App ==============");
    app.restart();
}
fn on_quit_click(app: &AppHandle) {
    use tauri_plugin_global_shortcut::GlobalShortcutExt;
    let _ = app.global_shortcut().unregister_all();
    info!("============== Quit App ==============");
    app.exit(0);
}

fn tray_menu(app: &AppHandle) -> Menu<Wry> {
    tray_menu_en_with_app(app)
}

fn tray_menu_en() -> Menu<Wry> {
    let app = APP.get().unwrap();
    tray_menu_en_with_app(app)
}

fn tray_menu_en_with_app(app: &AppHandle) -> Menu<Wry> {
    let enable_clipboard_monitor = match get("clipboard_monitor") {
        Some(v) => v.as_bool().unwrap(),
        None => {
            set("clipboard_monitor", false);
            false
        }
    };
    let copy_mode = match get("translate_auto_copy") {
        Some(v) => v.as_str().unwrap().to_string(),
        None => {
            set("translate_auto_copy", "disable");
            "disable".to_string()
        }
    };
    let input_translate = MenuItem::with_id(app, "input_translate", "Input Translate", true, None::<&str>).unwrap();
    let clipboard_monitor = CheckMenuItem::with_id(app, "clipboard_monitor", "Clipboard Monitor", true, enable_clipboard_monitor, None::<&str>).unwrap();
    let copy_source = CheckMenuItem::with_id(app, "copy_source", "Source", true, copy_mode == "source", None::<&str>).unwrap();
    let copy_target = CheckMenuItem::with_id(app, "copy_target", "Target", true, copy_mode == "target", None::<&str>).unwrap();
    let copy_source_target = CheckMenuItem::with_id(app, "copy_source_target", "Source+Target", true, copy_mode == "source_target", None::<&str>).unwrap();
    let copy_disable = CheckMenuItem::with_id(app, "copy_disable", "Disable", true, copy_mode == "disable", None::<&str>).unwrap();
    let ocr_recognize = MenuItem::with_id(app, "ocr_recognize", "OCR Recognize", true, None::<&str>).unwrap();
    let ocr_translate = MenuItem::with_id(app, "ocr_translate", "OCR Translate", true, None::<&str>).unwrap();
    let config = MenuItem::with_id(app, "config", "Config", true, None::<&str>).unwrap();
    let check_update = MenuItem::with_id(app, "check_update", "Check Update", true, None::<&str>).unwrap();
    let view_log = MenuItem::with_id(app, "view_log", "View Log", true, None::<&str>).unwrap();
    let restart = MenuItem::with_id(app, "restart", "Restart", true, None::<&str>).unwrap();
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>).unwrap();
    let separator = PredefinedMenuItem::separator(app).unwrap();

    let _ = CLIPBOARD_MONITOR_ITEM.set(clipboard_monitor.clone());
    let _ = COPY_SOURCE_ITEM.set(copy_source.clone());
    let _ = COPY_TARGET_ITEM.set(copy_target.clone());
    let _ = COPY_SOURCE_TARGET_ITEM.set(copy_source_target.clone());
    let _ = COPY_DISABLE_ITEM.set(copy_disable.clone());

    let auto_copy_menu = Submenu::with_items(
        app,
        "Auto Copy",
        true,
        &[&copy_source, &copy_target, &copy_source_target, &separator, &copy_disable],
    ).unwrap();

    let menu = Menu::with_items(
        app,
        &[
            &input_translate,
            &clipboard_monitor,
            &auto_copy_menu,
            &separator,
            &ocr_recognize,
            &ocr_translate,
            &separator,
            &config,
            &check_update,
            &view_log,
            &separator,
            &restart,
            &quit,
        ],
    ).unwrap();
    menu
}

fn tray_menu_zh_cn() -> Menu<Wry> {
    let app = APP.get().unwrap();
    let enable_clipboard_monitor = match get("clipboard_monitor") {
        Some(v) => v.as_bool().unwrap(),
        None => {
            set("clipboard_monitor", false);
            false
        }
    };
    let copy_mode = match get("translate_auto_copy") {
        Some(v) => v.as_str().unwrap().to_string(),
        None => {
            set("translate_auto_copy", "disable");
            "disable".to_string()
        }
    };
    let input_translate = MenuItem::with_id(app, "input_translate", "输入翻译", true, None::<&str>).unwrap();
    let clipboard_monitor = CheckMenuItem::with_id(app, "clipboard_monitor", "监听剪切板", true, enable_clipboard_monitor, None::<&str>).unwrap();
    let copy_source = CheckMenuItem::with_id(app, "copy_source", "原文", true, copy_mode == "source", None::<&str>).unwrap();
    let copy_target = CheckMenuItem::with_id(app, "copy_target", "译文", true, copy_mode == "target", None::<&str>).unwrap();
    let copy_source_target = CheckMenuItem::with_id(app, "copy_source_target", "原文+译文", true, copy_mode == "source_target", None::<&str>).unwrap();
    let copy_disable = CheckMenuItem::with_id(app, "copy_disable", "关闭", true, true, None::<&str>).unwrap();
    let ocr_recognize = MenuItem::with_id(app, "ocr_recognize", "文字识别", true, None::<&str>).unwrap();
    let ocr_translate = MenuItem::with_id(app, "ocr_translate", "截图翻译", true, None::<&str>).unwrap();
    let config = MenuItem::with_id(app, "config", "偏好设置", true, None::<&str>).unwrap();
    let check_update = MenuItem::with_id(app, "check_update", "检查更新", true, None::<&str>).unwrap();
    let restart = MenuItem::with_id(app, "restart", "重启应用", true, None::<&str>).unwrap();
    let view_log = MenuItem::with_id(app, "view_log", "查看日志", true, None::<&str>).unwrap();
    let quit = MenuItem::with_id(app, "quit", "退出", true, None::<&str>).unwrap();
    let separator = PredefinedMenuItem::separator(app).unwrap();

    let _ = CLIPBOARD_MONITOR_ITEM.set(clipboard_monitor.clone());
    let _ = COPY_SOURCE_ITEM.set(copy_source.clone());
    let _ = COPY_TARGET_ITEM.set(copy_target.clone());
    let _ = COPY_SOURCE_TARGET_ITEM.set(copy_source_target.clone());
    let _ = COPY_DISABLE_ITEM.set(copy_disable.clone());

    let auto_copy_menu = Submenu::with_items(
        app,
        "自动复制",
        true,
        &[&copy_source, &copy_target, &copy_source_target, &separator, &copy_disable],
    ).unwrap();

    Menu::with_items(
        app,
        &[
            &input_translate,
            &clipboard_monitor,
            &auto_copy_menu,
            &separator,
            &ocr_recognize,
            &ocr_translate,
            &separator,
            &config,
            &check_update,
            &view_log,
            &separator,
            &restart,
            &quit,
        ],
    ).unwrap()
}

fn tray_menu_zh_tw() -> Menu<Wry> {
    let app = APP.get().unwrap();
    let enable_clipboard_monitor = match get("clipboard_monitor") {
        Some(v) => v.as_bool().unwrap(),
        None => {
            set("clipboard_monitor", false);
            false
        }
    };
    let copy_mode = match get("translate_auto_copy") {
        Some(v) => v.as_str().unwrap().to_string(),
        None => {
            set("translate_auto_copy", "disable");
            "disable".to_string()
        }
    };
    let input_translate = MenuItem::with_id(app, "input_translate", "輸入翻譯", true, None::<&str>).unwrap();
    let clipboard_monitor = CheckMenuItem::with_id(app, "clipboard_monitor", "偵聽剪貼簿", true, enable_clipboard_monitor, None::<&str>).unwrap();
    let copy_source = CheckMenuItem::with_id(app, "copy_source", "原文", true, copy_mode == "source", None::<&str>).unwrap();
    let copy_target = CheckMenuItem::with_id(app, "copy_target", "譯文", true, copy_mode == "target", None::<&str>).unwrap();
    let copy_source_target = CheckMenuItem::with_id(app, "copy_source_target", "原文+譯文", true, copy_mode == "source_target", None::<&str>).unwrap();
    let copy_disable = CheckMenuItem::with_id(app, "copy_disable", "關閉", true, true, None::<&str>).unwrap();
    let ocr_recognize = MenuItem::with_id(app, "ocr_recognize", "文字識別", true, None::<&str>).unwrap();
    let ocr_translate = MenuItem::with_id(app, "ocr_translate", "截圖翻譯", true, None::<&str>).unwrap();
    let config = MenuItem::with_id(app, "config", "偏好設定", true, None::<&str>).unwrap();
    let check_update = MenuItem::with_id(app, "check_update", "檢查更新", true, None::<&str>).unwrap();
    let restart = MenuItem::with_id(app, "restart", "重啓程式", true, None::<&str>).unwrap();
    let view_log = MenuItem::with_id(app, "view_log", "查看日誌", true, None::<&str>).unwrap();
    let quit = MenuItem::with_id(app, "quit", "退出", true, None::<&str>).unwrap();
    let separator = PredefinedMenuItem::separator(app).unwrap();

    let _ = CLIPBOARD_MONITOR_ITEM.set(clipboard_monitor.clone());
    let _ = COPY_SOURCE_ITEM.set(copy_source.clone());
    let _ = COPY_TARGET_ITEM.set(copy_target.clone());
    let _ = COPY_SOURCE_TARGET_ITEM.set(copy_source_target.clone());
    let _ = COPY_DISABLE_ITEM.set(copy_disable.clone());

    let auto_copy_menu = Submenu::with_items(
        app,
        "自動複製",
        true,
        &[&copy_source, &copy_target, &copy_source_target, &separator, &copy_disable],
    ).unwrap();

    Menu::with_items(
        app,
        &[
            &input_translate,
            &clipboard_monitor,
            &auto_copy_menu,
            &separator,
            &ocr_recognize,
            &ocr_translate,
            &separator,
            &config,
            &check_update,
            &view_log,
            &separator,
            &restart,
            &quit,
        ],
    ).unwrap()
}

fn tray_menu_ja() -> Menu<Wry> {
    let app = APP.get().unwrap();
    let enable_clipboard_monitor = match get("clipboard_monitor") {
        Some(v) => v.as_bool().unwrap(),
        None => {
            set("clipboard_monitor", false);
            false
        }
    };
    let copy_mode = match get("translate_auto_copy") {
        Some(v) => v.as_str().unwrap().to_string(),
        None => {
            set("translate_auto_copy", "disable");
            "disable".to_string()
        }
    };
    let input_translate = MenuItem::with_id(app, "input_translate", "翻訳を入力", true, None::<&str>).unwrap();
    let clipboard_monitor = CheckMenuItem::with_id(app, "clipboard_monitor", "クリップボードを監視する", true, enable_clipboard_monitor, None::<&str>).unwrap();
    let copy_source = CheckMenuItem::with_id(app, "copy_source", "原文", true, copy_mode == "source", None::<&str>).unwrap();
    let copy_target = CheckMenuItem::with_id(app, "copy_target", "訳文", true, copy_mode == "target", None::<&str>).unwrap();
    let copy_source_target = CheckMenuItem::with_id(app, "copy_source_target", "原文+訳文", true, copy_mode == "source_target", None::<&str>).unwrap();
    let copy_disable = CheckMenuItem::with_id(app, "copy_disable", "閉じる", true, true, None::<&str>).unwrap();
    let ocr_recognize = MenuItem::with_id(app, "ocr_recognize", "テキスト認識", true, None::<&str>).unwrap();
    let ocr_translate = MenuItem::with_id(app, "ocr_translate", "スクリーンショットの翻訳", true, None::<&str>).unwrap();
    let config = MenuItem::with_id(app, "config", "プリファレンス設定", true, None::<&str>).unwrap();
    let check_update = MenuItem::with_id(app, "check_update", "更新を確認する", true, None::<&str>).unwrap();
    let restart = MenuItem::with_id(app, "restart", "アプリの再起動", true, None::<&str>).unwrap();
    let view_log = MenuItem::with_id(app, "view_log", "ログを見る", true, None::<&str>).unwrap();
    let quit = MenuItem::with_id(app, "quit", "退出する", true, None::<&str>).unwrap();
    let separator = PredefinedMenuItem::separator(app).unwrap();

    let _ = CLIPBOARD_MONITOR_ITEM.set(clipboard_monitor.clone());
    let _ = COPY_SOURCE_ITEM.set(copy_source.clone());
    let _ = COPY_TARGET_ITEM.set(copy_target.clone());
    let _ = COPY_SOURCE_TARGET_ITEM.set(copy_source_target.clone());
    let _ = COPY_DISABLE_ITEM.set(copy_disable.clone());

    let auto_copy_menu = Submenu::with_items(
        app,
        "自動コピー",
        true,
        &[&copy_source, &copy_target, &copy_source_target, &separator, &copy_disable],
    ).unwrap();

    Menu::with_items(
        app,
        &[
            &input_translate,
            &clipboard_monitor,
            &auto_copy_menu,
            &separator,
            &ocr_recognize,
            &ocr_translate,
            &separator,
            &config,
            &check_update,
            &view_log,
            &separator,
            &restart,
            &quit,
        ],
    ).unwrap()
}

fn tray_menu_ko() -> Menu<Wry> {
    let app = APP.get().unwrap();
    let enable_clipboard_monitor = match get("clipboard_monitor") {
        Some(v) => v.as_bool().unwrap(),
        None => {
            set("clipboard_monitor", false);
            false
        }
    };
    let copy_mode = match get("translate_auto_copy") {
        Some(v) => v.as_str().unwrap().to_string(),
        None => {
            set("translate_auto_copy", "disable");
            "disable".to_string()
        }
    };
    let input_translate = MenuItem::with_id(app, "input_translate", "입력 번역", true, None::<&str>).unwrap();
    let clipboard_monitor = CheckMenuItem::with_id(app, "clipboard_monitor", "감청 전단판", true, enable_clipboard_monitor, None::<&str>).unwrap();
    let copy_source = CheckMenuItem::with_id(app, "copy_source", "원문", true, copy_mode == "source", None::<&str>).unwrap();
    let copy_target = CheckMenuItem::with_id(app, "copy_target", "번역문", true, copy_mode == "target", None::<&str>).unwrap();
    let copy_source_target = CheckMenuItem::with_id(app, "copy_source_target", "원문+번역문", true, copy_mode == "source_target", None::<&str>).unwrap();
    let copy_disable = CheckMenuItem::with_id(app, "copy_disable", "닫기", true, true, None::<&str>).unwrap();
    let ocr_recognize = MenuItem::with_id(app, "ocr_recognize", "문자인식", true, None::<&str>).unwrap();
    let ocr_translate = MenuItem::with_id(app, "ocr_translate", "스크린샷 번역", true, None::<&str>).unwrap();
    let config = MenuItem::with_id(app, "config", "기본 설정", true, None::<&str>).unwrap();
    let check_update = MenuItem::with_id(app, "check_update", "업데이트 확인", true, None::<&str>).unwrap();
    let restart = MenuItem::with_id(app, "restart", "응용 프로그램 다시 시작", true, None::<&str>).unwrap();
    let view_log = MenuItem::with_id(app, "view_log", "로그 보기", true, None::<&str>).unwrap();
    let quit = MenuItem::with_id(app, "quit", "퇴출", true, None::<&str>).unwrap();
    let separator = PredefinedMenuItem::separator(app).unwrap();

    let _ = CLIPBOARD_MONITOR_ITEM.set(clipboard_monitor.clone());
    let _ = COPY_SOURCE_ITEM.set(copy_source.clone());
    let _ = COPY_TARGET_ITEM.set(copy_target.clone());
    let _ = COPY_SOURCE_TARGET_ITEM.set(copy_source_target.clone());
    let _ = COPY_DISABLE_ITEM.set(copy_disable.clone());

    let auto_copy_menu = Submenu::with_items(
        app,
        "자동 복사",
        true,
        &[&copy_source, &copy_target, &copy_source_target, &separator, &copy_disable],
    ).unwrap();

    Menu::with_items(
        app,
        &[
            &input_translate,
            &clipboard_monitor,
            &auto_copy_menu,
            &separator,
            &ocr_recognize,
            &ocr_translate,
            &separator,
            &config,
            &check_update,
            &view_log,
            &separator,
            &restart,
            &quit,
        ],
    ).unwrap()
}

fn tray_menu_fr() -> Menu<Wry> {
    let app = APP.get().unwrap();
    let enable_clipboard_monitor = match get("clipboard_monitor") {
        Some(v) => v.as_bool().unwrap(),
        None => {
            set("clipboard_monitor", false);
            false
        }
    };
    let copy_mode = match get("translate_auto_copy") {
        Some(v) => v.as_str().unwrap().to_string(),
        None => {
            set("translate_auto_copy", "disable");
            "disable".to_string()
        }
    };
    let input_translate = MenuItem::with_id(app, "input_translate", "Traduction d'entrée", true, None::<&str>).unwrap();
    let clipboard_monitor = CheckMenuItem::with_id(app, "clipboard_monitor", "Surveiller le presse-papiers", true, enable_clipboard_monitor, None::<&str>).unwrap();
    let copy_source = CheckMenuItem::with_id(app, "copy_source", "Source", true, copy_mode == "source", None::<&str>).unwrap();
    let copy_target = CheckMenuItem::with_id(app, "copy_target", "Cible", true, copy_mode == "target", None::<&str>).unwrap();
    let copy_source_target = CheckMenuItem::with_id(app, "copy_source_target", "Source+Cible", true, copy_mode == "source_target", None::<&str>).unwrap();
    let copy_disable = CheckMenuItem::with_id(app, "copy_disable", "Désactiver", true, true, None::<&str>).unwrap();
    let ocr_recognize = MenuItem::with_id(app, "ocr_recognize", "Reconnaissance de texte", true, None::<&str>).unwrap();
    let ocr_translate = MenuItem::with_id(app, "ocr_translate", "Traduction d'image", true, None::<&str>).unwrap();
    let config = MenuItem::with_id(app, "config", "Paramètres", true, None::<&str>).unwrap();
    let check_update = MenuItem::with_id(app, "check_update", "Vérifier les mises à jour", true, None::<&str>).unwrap();
    let restart = MenuItem::with_id(app, "restart", "Redémarrer l'application", true, None::<&str>).unwrap();
    let view_log = MenuItem::with_id(app, "view_log", "Voir le journal", true, None::<&str>).unwrap();
    let quit = MenuItem::with_id(app, "quit", "Quitter", true, None::<&str>).unwrap();
    let separator = PredefinedMenuItem::separator(app).unwrap();

    let _ = CLIPBOARD_MONITOR_ITEM.set(clipboard_monitor.clone());
    let _ = COPY_SOURCE_ITEM.set(copy_source.clone());
    let _ = COPY_TARGET_ITEM.set(copy_target.clone());
    let _ = COPY_SOURCE_TARGET_ITEM.set(copy_source_target.clone());
    let _ = COPY_DISABLE_ITEM.set(copy_disable.clone());

    let auto_copy_menu = Submenu::with_items(
        app,
        "Copier automatiquement",
        true,
        &[&copy_source, &copy_target, &copy_source_target, &separator, &copy_disable],
    ).unwrap();

    Menu::with_items(
        app,
        &[
            &input_translate,
            &clipboard_monitor,
            &auto_copy_menu,
            &separator,
            &ocr_recognize,
            &ocr_translate,
            &separator,
            &config,
            &check_update,
            &view_log,
            &separator,
            &restart,
            &quit,
        ],
    ).unwrap()
}
fn tray_menu_de() -> Menu<Wry> {
    let app = APP.get().unwrap();
    let enable_clipboard_monitor = match get("clipboard_monitor") {
        Some(v) => v.as_bool().unwrap(),
        None => {
            set("clipboard_monitor", false);
            false
        }
    };
    let copy_mode = match get("translate_auto_copy") {
        Some(v) => v.as_str().unwrap().to_string(),
        None => {
            set("translate_auto_copy", "disable");
            "disable".to_string()
        }
    };
    let input_translate = MenuItem::with_id(app, "input_translate", "Eingabeübersetzung", true, None::<&str>).unwrap();
    let clipboard_monitor = CheckMenuItem::with_id(app, "clipboard_monitor", "Zwischenablage überwachen", true, enable_clipboard_monitor, None::<&str>).unwrap();
    let copy_source = CheckMenuItem::with_id(app, "copy_source", "Quelle", true, copy_mode == "source", None::<&str>).unwrap();
    let copy_target = CheckMenuItem::with_id(app, "copy_target", "Ziel", true, copy_mode == "target", None::<&str>).unwrap();
    let copy_source_target = CheckMenuItem::with_id(app, "copy_source_target", "Quelle+Ziel", true, copy_mode == "source_target", None::<&str>).unwrap();
    let copy_disable = CheckMenuItem::with_id(app, "copy_disable", "Deaktivieren", true, true, None::<&str>).unwrap();
    let ocr_recognize = MenuItem::with_id(app, "ocr_recognize", "Texterkennung", true, None::<&str>).unwrap();
    let ocr_translate = MenuItem::with_id(app, "ocr_translate", "Bildübersetzung", true, None::<&str>).unwrap();
    let config = MenuItem::with_id(app, "config", "Einstellungen", true, None::<&str>).unwrap();
    let check_update = MenuItem::with_id(app, "check_update", "Auf Updates prüfen", true, None::<&str>).unwrap();
    let restart = MenuItem::with_id(app, "restart", "Anwendung neu starten", true, None::<&str>).unwrap();
    let view_log = MenuItem::with_id(app, "view_log", "Protokoll anzeigen", true, None::<&str>).unwrap();
    let quit = MenuItem::with_id(app, "quit", "Beenden", true, None::<&str>).unwrap();
    let separator = PredefinedMenuItem::separator(app).unwrap();

    let _ = CLIPBOARD_MONITOR_ITEM.set(clipboard_monitor.clone());
    let _ = COPY_SOURCE_ITEM.set(copy_source.clone());
    let _ = COPY_TARGET_ITEM.set(copy_target.clone());
    let _ = COPY_SOURCE_TARGET_ITEM.set(copy_source_target.clone());
    let _ = COPY_DISABLE_ITEM.set(copy_disable.clone());

    let auto_copy_menu = Submenu::with_items(
        app,
        "Automatisch kopieren",
        true,
        &[&copy_source, &copy_target, &copy_source_target, &separator, &copy_disable],
    ).unwrap();

    Menu::with_items(
        app,
        &[
            &input_translate,
            &clipboard_monitor,
            &auto_copy_menu,
            &separator,
            &ocr_recognize,
            &ocr_translate,
            &separator,
            &config,
            &check_update,
            &view_log,
            &separator,
            &restart,
            &quit,
        ],
    ).unwrap()
}

fn tray_menu_ru() -> Menu<Wry> {
    let app = APP.get().unwrap();
    let enable_clipboard_monitor = match get("clipboard_monitor") {
        Some(v) => v.as_bool().unwrap(),
        None => {
            set("clipboard_monitor", false);
            false
        }
    };
    let copy_mode = match get("translate_auto_copy") {
        Some(v) => v.as_str().unwrap().to_string(),
        None => {
            set("translate_auto_copy", "disable");
            "disable".to_string()
        }
    };
    let input_translate = MenuItem::with_id(app, "input_translate", "Ввод перевода", true, None::<&str>).unwrap();
    let clipboard_monitor = CheckMenuItem::with_id(app, "clipboard_monitor", "Следить за буфером обмена", true, enable_clipboard_monitor, None::<&str>).unwrap();
    let copy_source = CheckMenuItem::with_id(app, "copy_source", "Источник", true, copy_mode == "source", None::<&str>).unwrap();
    let copy_target = CheckMenuItem::with_id(app, "copy_target", "Цель", true, copy_mode == "target", None::<&str>).unwrap();
    let copy_source_target = CheckMenuItem::with_id(app, "copy_source_target", "Источник+Цель", true, copy_mode == "source_target", None::<&str>).unwrap();
    let copy_disable = CheckMenuItem::with_id(app, "copy_disable", "Отключить", true, true, None::<&str>).unwrap();
    let ocr_recognize = MenuItem::with_id(app, "ocr_recognize", "Распознавание текста", true, None::<&str>).unwrap();
    let ocr_translate = MenuItem::with_id(app, "ocr_translate", "Перевод изображения", true, None::<&str>).unwrap();
    let config = MenuItem::with_id(app, "config", "Настройки", true, None::<&str>).unwrap();
    let check_update = MenuItem::with_id(app, "check_update", "Проверить обновления", true, None::<&str>).unwrap();
    let restart = MenuItem::with_id(app, "restart", "Перезапустить приложение", true, None::<&str>).unwrap();
    let view_log = MenuItem::with_id(app, "view_log", "Просмотр журнала", true, None::<&str>).unwrap();
    let quit = MenuItem::with_id(app, "quit", "Выход", true, None::<&str>).unwrap();
    let separator = PredefinedMenuItem::separator(app).unwrap();

    let _ = CLIPBOARD_MONITOR_ITEM.set(clipboard_monitor.clone());
    let _ = COPY_SOURCE_ITEM.set(copy_source.clone());
    let _ = COPY_TARGET_ITEM.set(copy_target.clone());
    let _ = COPY_SOURCE_TARGET_ITEM.set(copy_source_target.clone());
    let _ = COPY_DISABLE_ITEM.set(copy_disable.clone());

    let auto_copy_menu = Submenu::with_items(
        app,
        "Автоматическое копирование",
        true,
        &[&copy_source, &copy_target, &copy_source_target, &separator, &copy_disable],
    ).unwrap();

    Menu::with_items(
        app,
        &[
            &input_translate,
            &clipboard_monitor,
            &auto_copy_menu,
            &separator,
            &ocr_recognize,
            &ocr_translate,
            &separator,
            &config,
            &check_update,
            &view_log,
            &separator,
            &restart,
            &quit,
        ],
    ).unwrap()
}

fn tray_menu_fa() -> Menu<Wry> {
    let app = APP.get().unwrap();
    let enable_clipboard_monitor = match get("clipboard_monitor") {
        Some(v) => v.as_bool().unwrap(),
        None => {
            set("clipboard_monitor", false);
            false
        }
    };
    let copy_mode = match get("translate_auto_copy") {
        Some(v) => v.as_str().unwrap().to_string(),
        None => {
            set("translate_auto_copy", "disable");
            "disable".to_string()
        }
    };
    let input_translate = MenuItem::with_id(app, "input_translate", "متن", true, None::<&str>).unwrap();
    let clipboard_monitor = CheckMenuItem::with_id(app, "clipboard_monitor", "گوش دادن به تخته برش", true, enable_clipboard_monitor, None::<&str>).unwrap();
    let copy_source = CheckMenuItem::with_id(app, "copy_source", "منبع", true, copy_mode == "source", None::<&str>).unwrap();
    let copy_target = CheckMenuItem::with_id(app, "copy_target", "هدف", true, copy_mode == "target", None::<&str>).unwrap();
    let copy_source_target = CheckMenuItem::with_id(app, "copy_source_target", "منبع + هدف", true, copy_mode == "source_target", None::<&str>).unwrap();
    let copy_disable = CheckMenuItem::with_id(app, "copy_disable", "متن", true, true, None::<&str>).unwrap();
    let ocr_recognize = MenuItem::with_id(app, "ocr_recognize", "تشخیص متن", true, None::<&str>).unwrap();
    let ocr_translate = MenuItem::with_id(app, "ocr_translate", "ترجمه عکس", true, None::<&str>).unwrap();
    let config = MenuItem::with_id(app, "config", "تنظیمات ترجیح", true, None::<&str>).unwrap();
    let check_update = MenuItem::with_id(app, "check_update", "بررسی بروزرسانی", true, None::<&str>).unwrap();
    let restart = MenuItem::with_id(app, "restart", "راه‌اندازی مجدد برنامه", true, None::<&str>).unwrap();
    let view_log = MenuItem::with_id(app, "view_log", "مشاهده گزارشات", true, None::<&str>).unwrap();
    let quit = MenuItem::with_id(app, "quit", "خروج", true, None::<&str>).unwrap();
    let separator = PredefinedMenuItem::separator(app).unwrap();

    let _ = CLIPBOARD_MONITOR_ITEM.set(clipboard_monitor.clone());
    let _ = COPY_SOURCE_ITEM.set(copy_source.clone());
    let _ = COPY_TARGET_ITEM.set(copy_target.clone());
    let _ = COPY_SOURCE_TARGET_ITEM.set(copy_source_target.clone());
    let _ = COPY_DISABLE_ITEM.set(copy_disable.clone());

    let auto_copy_menu = Submenu::with_items(
        app,
        "کپی خودکار",
        true,
        &[&copy_source, &copy_target, &copy_source_target, &separator, &copy_disable],
    ).unwrap();

    Menu::with_items(
        app,
        &[
            &input_translate,
            &clipboard_monitor,
            &auto_copy_menu,
            &separator,
            &ocr_recognize,
            &ocr_translate,
            &separator,
            &config,
            &check_update,
            &view_log,
            &separator,
            &restart,
            &quit,
        ],
    ).unwrap()
}

fn tray_menu_pt_br() -> Menu<Wry> {
    let app = APP.get().unwrap();
    let enable_clipboard_monitor = match get("clipboard_monitor") {
        Some(v) => v.as_bool().unwrap(),
        None => {
            set("clipboard_monitor", false);
            false
        }
    };
    let copy_mode = match get("translate_auto_copy") {
        Some(v) => v.as_str().unwrap().to_string(),
        None => {
            set("translate_auto_copy", "disable");
            "disable".to_string()
        }
    };
    let input_translate = MenuItem::with_id(app, "input_translate", "Traduzir Entrada", true, None::<&str>).unwrap();
    let clipboard_monitor = CheckMenuItem::with_id(app, "clipboard_monitor", "Monitorando a área de transferência", true, enable_clipboard_monitor, None::<&str>).unwrap();
    let copy_source = CheckMenuItem::with_id(app, "copy_source", "Origem", true, copy_mode == "source", None::<&str>).unwrap();
    let copy_target = CheckMenuItem::with_id(app, "copy_target", "Destino", true, copy_mode == "target", None::<&str>).unwrap();
    let copy_source_target = CheckMenuItem::with_id(app, "copy_source_target", "Origem+Destino", true, copy_mode == "source_target", None::<&str>).unwrap();
    let copy_disable = CheckMenuItem::with_id(app, "copy_disable", "Desabilitar", true, true, None::<&str>).unwrap();
    let ocr_recognize = MenuItem::with_id(app, "ocr_recognize", "Reconhecimento de Texto", true, None::<&str>).unwrap();
    let ocr_translate = MenuItem::with_id(app, "ocr_translate", "Tradução de Imagem", true, None::<&str>).unwrap();
    let config = MenuItem::with_id(app, "config", "Configurações", true, None::<&str>).unwrap();
    let check_update = MenuItem::with_id(app, "check_update", "Checar por Atualização", true, None::<&str>).unwrap();
    let restart = MenuItem::with_id(app, "restart", "Reiniciar aplicativo", true, None::<&str>).unwrap();
    let view_log = MenuItem::with_id(app, "view_log", "Exibir Registro", true, None::<&str>).unwrap();
    let quit = MenuItem::with_id(app, "quit", "Sair", true, None::<&str>).unwrap();
    let separator = PredefinedMenuItem::separator(app).unwrap();

    let _ = CLIPBOARD_MONITOR_ITEM.set(clipboard_monitor.clone());
    let _ = COPY_SOURCE_ITEM.set(copy_source.clone());
    let _ = COPY_TARGET_ITEM.set(copy_target.clone());
    let _ = COPY_SOURCE_TARGET_ITEM.set(copy_source_target.clone());
    let _ = COPY_DISABLE_ITEM.set(copy_disable.clone());

    let auto_copy_menu = Submenu::with_items(
        app,
        "Copiar Automaticamente",
        true,
        &[&copy_source, &copy_target, &copy_source_target, &separator, &copy_disable],
    ).unwrap();

    Menu::with_items(
        app,
        &[
            &input_translate,
            &clipboard_monitor,
            &auto_copy_menu,
            &separator,
            &ocr_recognize,
            &ocr_translate,
            &separator,
            &config,
            &check_update,
            &view_log,
            &separator,
            &restart,
            &quit,
        ],
    ).unwrap()
}

fn tray_menu_uk() -> Menu<Wry> {
    let app = APP.get().unwrap();
    let enable_clipboard_monitor = match get("clipboard_monitor") {
        Some(v) => v.as_bool().unwrap(),
        None => {
            set("clipboard_monitor", false);
            false
        }
    };
    let copy_mode = match get("translate_auto_copy") {
        Some(v) => v.as_str().unwrap().to_string(),
        None => {
            set("translate_auto_copy", "disable");
            "disable".to_string()
        }
    };
    let input_translate = MenuItem::with_id(app, "input_translate", "Введення перекладу", true, None::<&str>).unwrap();
    let clipboard_monitor = CheckMenuItem::with_id(app, "clipboard_monitor", "Стежити за буфером обміну", true, enable_clipboard_monitor, None::<&str>).unwrap();
    let copy_source = CheckMenuItem::with_id(app, "copy_source", "Джерело", true, copy_mode == "source", None::<&str>).unwrap();
    let copy_target = CheckMenuItem::with_id(app, "copy_target", "Мета", true, copy_mode == "target", None::<&str>).unwrap();
    let copy_source_target = CheckMenuItem::with_id(app, "copy_source_target", "Джерело+Мета", true, copy_mode == "source_target", None::<&str>).unwrap();
    let copy_disable = CheckMenuItem::with_id(app, "copy_disable", "Відключивши", true, true, None::<&str>).unwrap();
    let ocr_recognize = MenuItem::with_id(app, "ocr_recognize", "Розпізнавання тексту", true, None::<&str>).unwrap();
    let ocr_translate = MenuItem::with_id(app, "ocr_translate", "Переклад зображення", true, None::<&str>).unwrap();
    let config = MenuItem::with_id(app, "config", "Настройка", true, None::<&str>).unwrap();
    let check_update = MenuItem::with_id(app, "check_update", "Перевірити оновлення", true, None::<&str>).unwrap();
    let restart = MenuItem::with_id(app, "restart", "Перезапустити додаток", true, None::<&str>).unwrap();
    let view_log = MenuItem::with_id(app, "view_log", "Перегляд журналу", true, None::<&str>).unwrap();
    let quit = MenuItem::with_id(app, "quit", "Вихід", true, None::<&str>).unwrap();
    let separator = PredefinedMenuItem::separator(app).unwrap();

    let _ = CLIPBOARD_MONITOR_ITEM.set(clipboard_monitor.clone());
    let _ = COPY_SOURCE_ITEM.set(copy_source.clone());
    let _ = COPY_TARGET_ITEM.set(copy_target.clone());
    let _ = COPY_SOURCE_TARGET_ITEM.set(copy_source_target.clone());
    let _ = COPY_DISABLE_ITEM.set(copy_disable.clone());

    let auto_copy_menu = Submenu::with_items(
        app,
        "Автоматичне копіювання",
        true,
        &[&copy_source, &copy_target, &copy_source_target, &separator, &copy_disable],
    ).unwrap();

    Menu::with_items(
        app,
        &[
            &input_translate,
            &clipboard_monitor,
            &auto_copy_menu,
            &separator,
            &ocr_recognize,
            &ocr_translate,
            &separator,
            &config,
            &check_update,
            &view_log,
            &separator,
            &restart,
            &quit,
        ],
    ).unwrap()
}
