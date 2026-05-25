import { invoke } from '@tauri-apps/api/core'
import type { AppConfig } from '../config/app-config'

export async function getSelectedText(): Promise<string> {
  return await invoke<string>('get_selected_text')
}

export async function loadAppConfig(): Promise<AppConfig | null> {
  return await invoke<AppConfig | null>('load_app_config')
}

export async function saveAppConfig(config: AppConfig): Promise<void> {
  await invoke('save_app_config', { config })
}

export async function startScreenshotSelection(): Promise<string> {
  return await invoke<string>('start_screenshot_selection')
}

export async function readScreenshotImage(path: string): Promise<string> {
  return await invoke<string>('read_screenshot_image', { path })
}

export async function closePopupWindow(): Promise<void> {
  await invoke('close_popup_window')
}

export async function beginPopupDrag(cursorX: number, cursorY: number): Promise<void> {
  await invoke('begin_popup_drag', { cursorX, cursorY })
}

export async function dragPopupWindow(cursorX: number, cursorY: number): Promise<void> {
  await invoke('drag_popup_window', { cursorX, cursorY })
}

export async function endPopupDrag(): Promise<void> {
  await invoke('end_popup_drag')
}

export async function getPermissionStatus(): Promise<Record<string, string>> {
  return await invoke<Record<string, string>>('get_permission_status')
}

export async function openAccessibilitySettings(): Promise<void> {
  await invoke('open_accessibility_settings')
}

export async function openScreenRecordingSettings(): Promise<void> {
  await invoke('open_screen_recording_settings')
}

export async function openGithubReleases(): Promise<void> {
  await invoke('open_github_releases')
}
