use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

use tauri::command;

static SCREENSHOT_IN_PROGRESS: AtomicBool = AtomicBool::new(false);
static SCREENSHOT_PID: OnceLock<Mutex<Option<u32>>> = OnceLock::new();

#[cfg(target_os = "macos")]
mod macos_impl {
    use std::os::raw::c_uchar;

    #[link(name = "ApplicationServices", kind = "framework")]
    extern "C" {
        fn CGPreflightScreenCaptureAccess() -> c_uchar;
    }

    pub fn screen_recording_granted() -> bool {
        unsafe { CGPreflightScreenCaptureAccess() != 0 }
    }
}

fn encode_base64(bytes: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut encoded = String::with_capacity(bytes.len().div_ceil(3) * 4);

    for chunk in bytes.chunks(3) {
        let b0 = chunk[0];
        let b1 = *chunk.get(1).unwrap_or(&0);
        let b2 = *chunk.get(2).unwrap_or(&0);

        encoded.push(TABLE[(b0 >> 2) as usize] as char);
        encoded.push(TABLE[(((b0 & 0b0000_0011) << 4) | (b1 >> 4)) as usize] as char);

        if chunk.len() > 1 {
            encoded.push(TABLE[(((b1 & 0b0000_1111) << 2) | (b2 >> 6)) as usize] as char);
        } else {
            encoded.push('=');
        }

        if chunk.len() > 2 {
            encoded.push(TABLE[(b2 & 0b0011_1111) as usize] as char);
        } else {
            encoded.push('=');
        }
    }

    encoded
}

fn screenshot_path() -> Result<PathBuf, String> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| format!("生成截图文件名失败: {}", e))?
        .as_nanos();

    Ok(std::env::temp_dir().join(format!("lintranslate-screenshot-{timestamp}.png")))
}

fn screenshot_pid() -> &'static Mutex<Option<u32>> {
    SCREENSHOT_PID.get_or_init(|| Mutex::new(None))
}

fn set_screenshot_pid(pid: Option<u32>) {
    if let Ok(mut current_pid) = screenshot_pid().lock() {
        *current_pid = pid;
    }
}

fn cancel_active_screenshot() {
    let pid = screenshot_pid().lock().ok().and_then(|pid| *pid);
    if let Some(pid) = pid {
        let _ = Command::new("kill")
            .arg("-TERM")
            .arg(pid.to_string())
            .output();
    }
}

pub fn capture_screenshot_to_temp() -> Result<String, String> {
    if SCREENSHOT_IN_PROGRESS.swap(true, Ordering::SeqCst) {
        cancel_active_screenshot();
        return Err("截图已取消".to_string());
    }

    #[cfg(target_os = "macos")]
    if !macos_impl::screen_recording_granted() {
        SCREENSHOT_IN_PROGRESS.store(false, Ordering::SeqCst);
        return Err("缺少屏幕录制权限，请在系统设置中授权".to_string());
    }

    let result = capture_screenshot_to_temp_inner();
    set_screenshot_pid(None);
    SCREENSHOT_IN_PROGRESS.store(false, Ordering::SeqCst);
    result
}

fn capture_screenshot_to_temp_inner() -> Result<String, String> {
    let path = screenshot_path()?;
    let child = Command::new("/usr/sbin/screencapture")
        .args(["-i", "-x"])
        .arg(&path)
        .spawn()
        .map_err(|e| format!("启动截图工具失败: {}", e))?;
    set_screenshot_pid(Some(child.id()));

    let output = child
        .wait_with_output()
        .map_err(|e| format!("等待截图工具失败: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(if stderr.is_empty() {
            "截图已取消或失败".to_string()
        } else {
            stderr
        });
    }

    let metadata = fs::metadata(&path).map_err(|_| "截图已取消或未生成图片".to_string())?;
    if metadata.len() == 0 {
        let _ = fs::remove_file(&path);
        return Err("截图为空，请重新框选区域".to_string());
    }

    Ok(path.to_string_lossy().to_string())
}

fn ensure_temp_screenshot_path(path: &Path) -> Result<(), String> {
    let canonical = path
        .canonicalize()
        .map_err(|e| format!("读取截图文件失败: {}", e))?;
    let temp_dir = std::env::temp_dir()
        .canonicalize()
        .map_err(|e| format!("读取临时目录失败: {}", e))?;

    if !canonical.starts_with(temp_dir) {
        return Err("截图文件路径无效".to_string());
    }

    let file_name = canonical
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default();
    if !file_name.starts_with("lintranslate-screenshot-") || !file_name.ends_with(".png") {
        return Err("截图文件路径无效".to_string());
    }

    Ok(())
}

#[command]
pub fn start_screenshot_selection() -> Result<String, String> {
    #[cfg(target_os = "macos")]
    return capture_screenshot_to_temp();

    #[cfg(not(target_os = "macos"))]
    Err("This command is only available on macOS".to_string())
}

#[command]
pub fn read_screenshot_image(path: String) -> Result<String, String> {
    let path = PathBuf::from(path);
    ensure_temp_screenshot_path(&path)?;

    let bytes = fs::read(&path).map_err(|e| format!("读取截图文件失败: {}", e))?;
    let _ = fs::remove_file(&path);

    Ok(encode_base64(&bytes))
}
