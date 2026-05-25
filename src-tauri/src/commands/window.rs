use std::sync::Mutex;

use tauri::{command, Manager, PhysicalPosition, State};

#[derive(Clone, Copy)]
struct PopupDragSession {
    start_cursor_x: f64,
    start_cursor_y: f64,
    start_window_x: i32,
    start_window_y: i32,
    scale_factor: f64,
}

#[derive(Default)]
pub struct PopupDragState {
    session: Mutex<Option<PopupDragSession>>,
}

#[command]
pub fn close_popup_window(app: tauri::AppHandle) -> Result<(), String> {
    if let Some(win) = app.get_webview_window("popup") {
        win.destroy()
            .map_err(|e| format!("关闭翻译气泡窗口失败: {}", e))?;
    }

    Ok(())
}

#[command]
pub fn begin_popup_drag(
    app: tauri::AppHandle,
    state: State<'_, PopupDragState>,
    cursor_x: f64,
    cursor_y: f64,
) -> Result<(), String> {
    let win = app
        .get_webview_window("popup")
        .ok_or_else(|| "未找到翻译气泡窗口".to_string())?;
    let position = win
        .outer_position()
        .map_err(|e| format!("读取翻译气泡位置失败: {}", e))?;
    let scale_factor = win.scale_factor().unwrap_or(1.0);

    let mut session = state
        .session
        .lock()
        .map_err(|_| "翻译气泡拖动状态锁定失败".to_string())?;
    *session = Some(PopupDragSession {
        start_cursor_x: cursor_x,
        start_cursor_y: cursor_y,
        start_window_x: position.x,
        start_window_y: position.y,
        scale_factor,
    });

    Ok(())
}

#[command]
pub fn drag_popup_window(
    app: tauri::AppHandle,
    state: State<'_, PopupDragState>,
    cursor_x: f64,
    cursor_y: f64,
) -> Result<(), String> {
    let session = {
        let session = state
            .session
            .lock()
            .map_err(|_| "翻译气泡拖动状态锁定失败".to_string())?;
        *session
    };
    let Some(session) = session else {
        return Ok(());
    };

    let win = app
        .get_webview_window("popup")
        .ok_or_else(|| "未找到翻译气泡窗口".to_string())?;
    let next_x =
        session.start_window_x as f64 + (cursor_x - session.start_cursor_x) * session.scale_factor;
    let next_y =
        session.start_window_y as f64 + (cursor_y - session.start_cursor_y) * session.scale_factor;

    win.set_position(PhysicalPosition::new(
        next_x.round() as i32,
        next_y.round() as i32,
    ))
    .map_err(|e| format!("移动翻译气泡窗口失败: {}", e))?;

    Ok(())
}

#[command]
pub fn end_popup_drag(state: State<'_, PopupDragState>) -> Result<(), String> {
    let mut session = state
        .session
        .lock()
        .map_err(|_| "翻译气泡拖动状态锁定失败".to_string())?;
    *session = None;

    Ok(())
}
