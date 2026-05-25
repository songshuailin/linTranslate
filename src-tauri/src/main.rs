#![allow(unexpected_cfgs)]

use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use std::time::{Duration, Instant};

use tauri::image::Image;
use tauri::menu::{IsMenuItem, MenuItemBuilder};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::ActivationPolicy;
use tauri::Manager;
use tauri::{RunEvent, WindowEvent};
use tauri::{WebviewUrl, WebviewWindowBuilder};
use tauri_plugin_global_shortcut::{Code, Modifiers, ShortcutState};

mod commands;

struct QuitGuard {
    explicit_quit: AtomicBool,
    last_quit_request: Mutex<Option<Instant>>,
}

fn show_settings(app: &tauri::AppHandle) {
    if let Some(win) = app.get_webview_window("main") {
        win.show().ok();
        win.set_focus().ok();
        return;
    }

    match WebviewWindowBuilder::new(app, "main", WebviewUrl::App("index.html".into()))
        .title("Local Bubble Translator")
        .inner_size(560.0, 480.0)
        .resizable(true)
        .skip_taskbar(true)
        .focused(true)
        .build()
    {
        Ok(win) => {
            win.show().ok();
            win.set_focus().ok();
        }
        Err(err) => {
            eprintln!("[settings] Failed to open settings window: {}", err);
        }
    }
}

fn percent_encode(input: &str) -> String {
    let mut encoded = String::new();
    for byte in input.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(byte as char);
            }
            _ => encoded.push_str(&format!("%{byte:02X}")),
        }
    }
    encoded
}

fn open_popup(app: &tauri::AppHandle, url: String) {
    if let Some(win) = app.get_webview_window("popup") {
        win.destroy().ok();
    }

    match WebviewWindowBuilder::new(app, "popup", WebviewUrl::App(url.into()))
        .title("灵译")
        .inner_size(440.0, 360.0)
        .center()
        .resizable(false)
        .decorations(false)
        .transparent(true)
        .always_on_top(true)
        .skip_taskbar(true)
        .shadow(true)
        .focused(true)
        .build()
    {
        Ok(win) => {
            win.show().ok();
            win.set_focus().ok();
        }
        Err(err) => {
            eprintln!("[popup] Failed to open popup: {}", err);
        }
    }
}

fn open_selection_popup(app: &tauri::AppHandle, text: &str) {
    open_popup(
        app,
        format!(
            "index.html?window=popup&mode=selection&text={}",
            percent_encode(text)
        ),
    );
}

fn open_screenshot_popup(app: &tauri::AppHandle, image_path: &str) {
    open_popup(
        app,
        format!(
            "index.html?window=popup&mode=screenshot&imagePath={}",
            percent_encode(image_path)
        ),
    );
}

fn open_error_popup(app: &tauri::AppHandle, message: &str) {
    open_popup(
        app,
        format!(
            "index.html?window=popup&mode=error&message={}",
            percent_encode(message)
        ),
    );
}

fn trigger_translate_selection(app: tauri::AppHandle) {
    if let Some(win) = app.get_webview_window("popup") {
        win.destroy().ok();
        return;
    }

    tauri::async_runtime::spawn(async move {
        match commands::clipboard::get_selected_text() {
            Ok(text) if !text.is_empty() => {
                println!("[translate_selection] Got selected text: {}", text);
                open_selection_popup(&app, &text);
            }
            Ok(_) => {
                println!("[translate_selection] No selected text found");
                open_error_popup(&app, "未检测到选中文本");
            }
            Err(err) => {
                eprintln!("[translate_selection] Error: {}", err);
                open_error_popup(&app, &format!("获取选中文本失败: {}", err));
            }
        }
    });
}

fn trigger_screenshot_translation(app: tauri::AppHandle) {
    if let Some(win) = app.get_webview_window("popup") {
        win.destroy().ok();
    }

    tauri::async_runtime::spawn(async move {
        match commands::screenshot::capture_screenshot_to_temp() {
            Ok(image_path) => {
                println!("[translate_screenshot] Captured screenshot: {}", image_path);
                open_screenshot_popup(&app, &image_path);
            }
            Err(err) => {
                println!("[translate_screenshot] Interrupted: {}", err);
                if is_screenshot_permission_error(&err) {
                    open_error_popup(&app, "截图失败：请在系统设置中授权屏幕录制权限");
                } else if !is_screenshot_cancelled(&err) {
                    open_error_popup(&app, &err);
                }
            }
        }
    });
}

fn is_screenshot_cancelled(err: &str) -> bool {
    err.contains("取消")
        || err.contains("未生成图片")
        || err.contains("could not create image from rect")
}

fn is_screenshot_permission_error(err: &str) -> bool {
    err.contains("屏幕录制权限")
}

fn main() {
    let quit_guard = Arc::new(QuitGuard {
        explicit_quit: AtomicBool::new(false),
        last_quit_request: Mutex::new(None),
    });

    let builder = tauri::Builder::<tauri::Wry>::default().plugin(
        tauri_plugin_global_shortcut::Builder::new()
            .with_shortcuts(["Command+E", "Command+R"])
            .expect("Failed to register default global shortcuts")
            .with_handler(|app, shortcut, event| {
                if event.state != ShortcutState::Pressed {
                    return;
                }

                if shortcut.matches(Modifiers::SUPER, Code::KeyE) {
                    trigger_translate_selection(app.clone());
                } else if shortcut.matches(Modifiers::SUPER, Code::KeyR) {
                    trigger_screenshot_translation(app.clone());
                }
            })
            .build(),
    );

    // Register Tauri commands.
    let builder = builder
        .manage(commands::window::PopupDragState::default())
        .invoke_handler(tauri::generate_handler![
            commands::clipboard::get_selected_text,
            commands::config::load_app_config,
            commands::config::save_app_config,
            commands::permissions::get_permission_status,
            commands::permissions::open_accessibility_settings,
            commands::permissions::open_screen_recording_settings,
            commands::screenshot::start_screenshot_selection,
            commands::screenshot::read_screenshot_image,
            commands::url::open_github_releases,
            commands::window::close_popup_window,
            commands::window::begin_popup_drag,
            commands::window::drag_popup_window,
            commands::window::end_popup_drag
        ]);

    let builder = {
        use tauri::menu::MenuEvent;
        let quit_guard = Arc::clone(&quit_guard);

        builder.on_menu_event(move |app, event: MenuEvent| {
            let id_str: &str = &event.id.0;
            match id_str {
                "open_settings" => show_settings(app),
                "translate_selection" => {
                    trigger_translate_selection(app.app_handle().clone());
                }
                "translate_screenshot" => {
                    trigger_screenshot_translation(app.app_handle().clone());
                }
                "quit" => {
                    quit_guard.explicit_quit.store(true, Ordering::SeqCst);
                    app.exit(0);
                }
                _ => {}
            }
        })
    }
    .on_window_event(|window, event| {
        if let WindowEvent::CloseRequested { api, .. } = event {
            match window.label() {
                "main" => {
                    api.prevent_close();
                    window.hide().ok();
                }
                "popup" => {
                    api.prevent_close();
                    window.destroy().ok();
                }
                _ => {}
            }
        }
    });

    let app = builder
        .setup(|app| {
            #[cfg(target_os = "macos")]
            {
                app.set_activation_policy(ActivationPolicy::Accessory);
                app.set_dock_visibility(false);
            }

            let settings_window = app.get_webview_window("main").expect(
                "Failed to get settings window - check tauri.conf.json has a 'main' window",
            );
            settings_window.hide().ok();

            let open_settings =
                MenuItemBuilder::with_id("open_settings", "打开设置页").build(app)?;
            let translate_selection =
                MenuItemBuilder::with_id("translate_selection", "翻译选中文本").build(app)?;
            let translate_screenshot =
                MenuItemBuilder::with_id("translate_screenshot", "截图翻译").build(app)?;
            let quit = MenuItemBuilder::with_id("quit", "退出").build(app)?;

            let separator = tauri::menu::PredefinedMenuItem::separator(app)?;
            let tray_menu = tauri::menu::Menu::with_items(
                app,
                &[
                    &open_settings as &dyn IsMenuItem<_>,
                    &translate_selection as &dyn IsMenuItem<_>,
                    &translate_screenshot as &dyn IsMenuItem<_>,
                    &separator as &dyn IsMenuItem<_>,
                    &quit as &dyn IsMenuItem<_>,
                ],
            )?;

            let tray_icon = Image::from_bytes(include_bytes!("../icons/icon.png"))
                .expect("Failed to load tray icon from src-tauri/icons/icon.png");

            TrayIconBuilder::with_id("status")
                .icon(tray_icon)
                .icon_as_template(false)
                .tooltip("Local Bubble Translator")
                .menu(&tray_menu)
                .show_menu_on_left_click(false)
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        show_settings(tray.app_handle());
                    }
                })
                .build(app)?;

            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("Failed to build app");

    let quit_guard = Arc::clone(&quit_guard);
    app.run(move |app_handle, event| {
        if let RunEvent::ExitRequested { api, .. } = event {
            if quit_guard.explicit_quit.load(Ordering::SeqCst) {
                return;
            }

            let now = Instant::now();
            let should_exit = {
                let mut last_quit_request = quit_guard
                    .last_quit_request
                    .lock()
                    .expect("quit guard mutex poisoned");
                let should_exit = last_quit_request
                    .map(|last| now.duration_since(last) <= Duration::from_secs(2))
                    .unwrap_or(false);
                *last_quit_request = Some(now);
                should_exit
            };

            if !should_exit {
                api.prevent_exit();
                open_error_popup(app_handle, "再次按 Command+Q 退出灵译");
            }
        }
    });
}
