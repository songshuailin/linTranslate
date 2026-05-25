<script setup lang="ts">
import SettingsWindow from './windows/settings/SettingsWindow.vue'
import PopupWindow from './windows/popup/PopupWindow.vue'
import { usePopupStore } from './windows/popup/popup-store'
import { translateTextStream } from './services/translator/text-translator'
import { translateImageStream } from './services/translator/image-translator'
import { onMounted, ref } from 'vue'
import type { AppConfig } from './services/config/app-config'
import type { TranslationPopup, TranslationStatus } from './services/translator/translator-types'
import { loadConfig } from './services/config/config-storage'
import { closePopupWindow as closePopupWindowCommand, readScreenshotImage, startScreenshotSelection } from './services/tauri/commands'
import { emit, emitTo, listen } from '@tauri-apps/api/event'
import { getCurrentWebviewWindow, WebviewWindow } from '@tauri-apps/api/webviewWindow'

const popupStore = usePopupStore()
const translationControllers = new Map<string, AbortController>()
const currentWindow = getCurrentWebviewWindow()
const isPopupWindow = currentWindow.label === 'popup'
let config: AppConfig | null = null
let isClosingPopupWindow = false
const isPopupDragActive = ref(false)

function markPopupDragStart() {
  isPopupDragActive.value = true
}

function markPopupDragEnd() {
  window.setTimeout(() => {
    isPopupDragActive.value = false
  }, 120)
}

onMounted(async () => {
  if (isPopupWindow) return
  config = await loadConfig()
})

async function closeMainWindow() {
  await currentWindow.hide()
}

async function translateInCurrentWindow(selectedText: string) {
  const runtimeConfig = config ?? await loadConfig()
  config = runtimeConfig

  const popupId = popupStore.createPopup({
    mode: 'selection',
    sourceText: selectedText,
    targetLanguage: runtimeConfig.targetLanguage || '中文',
  })

  popupStore.setStatus(popupId, 'streaming')

  const abortController = new AbortController()
  translationControllers.set(popupId, abortController)

  try {
    const stream = translateTextStream(
      runtimeConfig.textModel,
      { text: selectedText, targetLanguage: runtimeConfig.targetLanguage || '中文' },
      { signal: abortController.signal },
    )

    for await (const delta of stream) {
      popupStore.appendDelta(popupId, delta)
    }

    popupStore.setStatus(popupId, 'done')
  } catch (e: unknown) {
    if (e instanceof DOMException && e.name === 'AbortError') return

    const errorMsg = e instanceof Error ? e.message : '未知错误'
    popupStore.setError(popupId, errorMsg)
  } finally {
    translationControllers.delete(popupId)
  }
}

async function translateScreenshotInCurrentWindow(imageBase64: string) {
  const runtimeConfig = config ?? await loadConfig()
  config = runtimeConfig

  const popupId = popupStore.createPopup({
    mode: 'screenshot',
    targetLanguage: runtimeConfig.targetLanguage || '中文',
  })

  popupStore.setStatus(popupId, 'streaming')

  const abortController = new AbortController()
  translationControllers.set(popupId, abortController)

  try {
    const stream = translateImageStream(
      runtimeConfig.visionModel,
      { imageBase64, targetLanguage: runtimeConfig.targetLanguage || '中文' },
      { signal: abortController.signal },
    )

    let hasDelta = false
    for await (const delta of stream) {
      hasDelta = true
      popupStore.appendDelta(popupId, delta)
    }
    if (!hasDelta) {
      popupStore.appendDelta(popupId, '未识别到可翻译文字')
    }

    popupStore.setStatus(popupId, 'done')
  } catch (e: unknown) {
    if (e instanceof DOMException && e.name === 'AbortError') return

    const errorMsg = e instanceof Error ? e.message : '未知错误'
    popupStore.setError(popupId, errorMsg)
  } finally {
    translationControllers.delete(popupId)
  }
}

function showErrorInCurrentWindow(errorMessage: string) {
  const popupId = popupStore.createPopup({
    mode: 'selection',
    sourceText: '',
    targetLanguage: config?.targetLanguage || '中文',
  })

  popupStore.setError(popupId, errorMessage)
}

async function startPopupFromUrl() {
  const params = new URL(window.location.href).searchParams
  const mode = params.get('mode')

  if (mode === 'selection') {
    const selectedText = params.get('text')?.trim()
    if (selectedText) {
      await translateInCurrentWindow(selectedText)
    } else {
      showErrorInCurrentWindow('未检测到选中文本')
    }
    return
  }

  if (mode === 'screenshot') {
    try {
      const imagePath = params.get('imagePath')
      const imageBase64 = imagePath
        ? await readScreenshotImage(imagePath)
        : await readScreenshotImage(await startScreenshotSelection())
      await translateScreenshotInCurrentWindow(imageBase64)
    } catch (e: unknown) {
      showErrorInCurrentWindow(e instanceof Error ? e.message : '截图翻译失败')
    }
    return
  }

  if (mode === 'error') {
    showErrorInCurrentWindow(params.get('message') || '操作失败')
  }
}

async function getPopupWindow(): Promise<WebviewWindow> {
  const existing = await WebviewWindow.getByLabel('popup')
  if (existing) {
    await existing.show()
    await existing.setFocus()
    return existing
  }

  const popup = new WebviewWindow('popup', {
    url: 'index.html',
    title: '灵译',
    width: 440,
    height: 360,
    center: true,
    resizable: false,
    decorations: false,
    transparent: true,
    alwaysOnTop: true,
    skipTaskbar: true,
    shadow: true,
    focus: true,
  })

  await new Promise<void>((resolve, reject) => {
    popup.once('tauri://created', () => resolve())
    popup.once('tauri://error', (event) => reject(event.payload))
  })

  await new Promise(resolve => window.setTimeout(resolve, 120))

  return popup
}

async function emitPopupState(popup: TranslationPopup) {
  await getPopupWindow()
  await emitTo('popup', 'popup-state', popup)
}

async function emitPopupDelta(id: string, delta: string) {
  await emitTo('popup', 'popup-delta', { id, delta })
}

async function emitPopupStatus(id: string, status: TranslationStatus) {
  await emitTo('popup', 'popup-status', { id, status })
}

async function emitPopupError(id: string, message: string) {
  await emitTo('popup', 'popup-error', { id, message })
}

async function handleSelectedText(selectedText: string) {
  if (!config) return

  const popupId = popupStore.createPopup({
    mode: 'selection',
    sourceText: selectedText,
    targetLanguage: config.targetLanguage || '中文',
  })
  const popup = popupStore.popups.find(p => p.id === popupId)

  popupStore.setStatus(popupId, 'streaming')
  if (popup) {
    popup.status = 'streaming'
    await emitPopupState(popup)
  }

  const abortController = new AbortController()
  translationControllers.set(popupId, abortController)

  try {
    const stream = translateTextStream(
      config.textModel,
      { text: selectedText, targetLanguage: config.targetLanguage || '中文' },
      { signal: abortController.signal },
    )

    for await (const delta of stream) {
      popupStore.appendDelta(popupId, delta)
      await emitPopupDelta(popupId, delta)
    }

    popupStore.setStatus(popupId, 'done')
    await emitPopupStatus(popupId, 'done')
  } catch (e: unknown) {
    if (e instanceof DOMException && e.name === 'AbortError') return

    popupStore.setStatus(popupId, 'error')
    const errorMsg = e instanceof Error ? e.message : '未知错误'
    popupStore.setError(popupId, errorMsg)
    await emitPopupError(popupId, errorMsg)
  } finally {
    translationControllers.delete(popupId)
  }
}

async function handleSelectionError(errorMessage: string) {
  const popupId = popupStore.createPopup({
    mode: 'selection',
    sourceText: '',
    targetLanguage: config?.targetLanguage || '中文',
  })
  const popup = popupStore.popups.find(p => p.id === popupId)

  popupStore.setStatus(popupId, 'error')
  popupStore.setError(popupId, errorMessage)
  if (popup) {
    popup.status = 'error'
    popup.errorMessage = errorMessage
    await emitPopupState(popup)
  }
}

onMounted(async () => {
  if (isPopupWindow) {
    await startPopupFromUrl()

    await listen<TranslationPopup>('popup-state', (event) => {
      popupStore.popups.splice(0, popupStore.popups.length, event.payload)
    })

    await listen<{ id: string; delta: string }>('popup-delta', (event) => {
      popupStore.appendDelta(event.payload.id, event.payload.delta)
    })

    await listen<{ id: string; status: TranslationStatus }>('popup-status', (event) => {
      popupStore.setStatus(event.payload.id, event.payload.status)
    })

    await listen<{ id: string; message: string }>('popup-error', (event) => {
      popupStore.setError(event.payload.id, event.payload.message)
    })

    await listen<string>('popup-close', (event) => {
      popupStore.closePopup(event.payload)
      if (popupStore.popups.length === 0) {
        destroyPopupHostWindow()
      }
    })

    await currentWindow.onFocusChanged((event) => {
      if (!event.payload) {
        window.setTimeout(() => {
          if (!isPopupDragActive.value) {
            closePopupWindow()
          }
        }, 160)
      }
    })

    window.addEventListener('keydown', (event) => {
      if (event.key === 'Escape') {
        closePopupWindow()
      }
    })

    await emit('popup-ready')

    return
  }

  await listen<string>('selected-text', (event) => {
    console.log('[selected-text]', event.payload)
    handleSelectedText(event.payload)
  })

  await listen<string>('selection-error', (event) => {
    console.log('[selection-error]', event.payload)
    handleSelectionError(event.payload)
  })

  await listen<string>('popup-close', (event) => {
    translationControllers.get(event.payload)?.abort()
    translationControllers.delete(event.payload)
    popupStore.closePopup(event.payload)
  })

})

function closePopup(popupId: string) {
  translationControllers.get(popupId)?.abort()
  translationControllers.delete(popupId)
  popupStore.closePopup(popupId)
  if (isPopupWindow) {
    emitTo('main', 'popup-close', popupId)
    if (popupStore.popups.length === 0) {
      destroyPopupHostWindow()
    }
  }
}

function closePopupWindow() {
  for (const popup of popupStore.popups) {
    translationControllers.get(popup.id)?.abort()
    translationControllers.delete(popup.id)
    emitTo('main', 'popup-close', popup.id)
  }
  popupStore.popups.splice(0, popupStore.popups.length)
  destroyPopupHostWindow()
}

async function destroyPopupHostWindow() {
  if (!isPopupWindow || isClosingPopupWindow) return

  isClosingPopupWindow = true
  try {
    await closePopupWindowCommand()
  } catch (e) {
    console.error('[popup] Rust close failed, falling back to frontend destroy', e)
    await currentWindow.destroy()
  }
}

</script>

<template>
  <div class="app-container" :class="{ 'popup-mode': isPopupWindow }">
    <SettingsWindow v-if="!isPopupWindow" @close="closeMainWindow" />
    <div v-if="isPopupWindow || popupStore.popups.length > 0" class="popup-list">
      <PopupWindow
        v-for="p in popupStore.popups"
        :key="p.id"
        :popup="p"
        @close="closePopup(p.id)"
        @drag-start="markPopupDragStart"
        @drag-end="markPopupDragEnd"
      />
    </div>
  </div>
</template>

<style scoped>
.app-container {
  width: 100vw;
  height: 100vh;
  overflow: hidden;
  background: #f4f5f7;
}

.popup-mode {
  display: flex;
  align-items: stretch;
  justify-content: stretch;
  padding: 0;
  background: transparent;
}

.popup-list {
  position: fixed;
  bottom: 80px;
  right: 24px;
  display: flex;
  flex-direction: column-reverse;
  gap: 12px;
}

.popup-mode .popup-list {
  position: static;
  width: 100%;
  height: 100%;
  flex-direction: column;
  gap: 0;
}
</style>

<style>
* {
  box-sizing: border-box;
}

html,
body,
#app {
  width: 100%;
  height: 100%;
  margin: 0;
  overflow: hidden;
  background: transparent;
  color: #1f2937;
  font-family: -apple-system, BlinkMacSystemFont, "SF Pro Text", "Segoe UI", sans-serif;
  letter-spacing: 0;
}

button,
input,
select,
textarea {
  font: inherit;
}
</style>
