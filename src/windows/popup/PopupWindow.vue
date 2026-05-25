<script setup lang="ts">
import { ref } from 'vue'
import { beginPopupDrag, dragPopupWindow, endPopupDrag } from '../../services/tauri/commands'
import type { TranslationPopup, TranslationStatus } from '../../services/translator/translator-types'

interface Props {
  popup: TranslationPopup
}

const props = defineProps<Props>()
const emit = defineEmits<{
  close: []
  copy: [text: string]
  dragStart: []
  dragEnd: []
}>()

const isCopied = ref(false)

async function copyText(): Promise<void> {
  try {
    await navigator.clipboard.writeText(props.popup.translatedText)
    isCopied.value = true
    emit('copy', props.popup.translatedText)
    setTimeout(() => { isCopied.value = false }, 1500)
  } catch { /* ignore */ }
}

async function startDrag(event: PointerEvent): Promise<void> {
  if (event.button !== 0) return

  const target = event.target as HTMLElement | null
  if (target?.closest('button')) return

  event.preventDefault()
  emit('dragStart')

  const dragTarget = event.currentTarget as HTMLElement | null
  dragTarget?.setPointerCapture?.(event.pointerId)

  let frame = 0
  let isActive = true
  let beginFinished = false
  let beginFailed = false
  let latestCursor = { x: event.screenX, y: event.screenY }
  let beginPromise: Promise<void> = Promise.resolve()

  const finishDrag = () => {
    if (!isActive) return
    isActive = false
    if (frame) {
      cancelAnimationFrame(frame)
      frame = 0
    }
    dragTarget?.releasePointerCapture?.(event.pointerId)
    window.removeEventListener('pointermove', moveManually)
    window.removeEventListener('pointerup', finishDrag)
    window.removeEventListener('pointercancel', finishDrag)

    const endDrag = () => endPopupDrag().catch((e) => console.error('[popup] end drag failed', e))
    if (beginFinished) {
      endDrag()
    } else {
      beginPromise.finally(endDrag)
    }

    emit('dragEnd')
  }

  const updatePosition = async () => {
    frame = 0
    await beginPromise.catch(() => {})
    if (!isActive || beginFailed) return

    await dragPopupWindow(latestCursor.x, latestCursor.y)
      .catch((e) => console.error('[popup] drag failed', e))
  }

  function moveManually(moveEvent: PointerEvent) {
    latestCursor = { x: moveEvent.screenX, y: moveEvent.screenY }
    if (!frame) {
      frame = requestAnimationFrame(updatePosition)
    }
  }

  window.addEventListener('pointermove', moveManually)
  window.addEventListener('pointerup', finishDrag)
  window.addEventListener('pointercancel', finishDrag)

  beginPromise = beginPopupDrag(event.screenX, event.screenY)
    .then(() => { beginFinished = true })
    .catch((e) => {
      beginFailed = true
      console.error('[popup] begin drag failed', e)
      finishDrag()
    })
}

const statusLabel: Record<TranslationStatus, string> = {
  idle: '等待中',
  streaming: '翻译中...',
  done: '完成',
  error: '错误',
}

const statusClass = (status: TranslationStatus): string => {
  switch (status) {
    case 'idle': return 'text-gray-400'
    case 'streaming': return 'status-streaming'
    case 'done': return 'text-green-600'
    case 'error': return 'text-red-500'
  }
}

function popupTitle(): string {
  return props.popup.mode === 'screenshot' ? '截图翻译' : '翻译'
}
</script>

<template>
  <div class="popup-container">
    <!-- Header -->
    <div class="header-bar" @pointerdown="startDrag">
      <div class="flex-items-center gap-2">
        <span class="text-sm font-medium text-gray-700">{{ popupTitle() }}</span>
        <span class="text-xs" :class="statusClass(popup.status)">{{ statusLabel[popup.status] }}</span>
      </div>
      <div class="flex gap-1">
        <button title="复制" class="action-btn" @pointerdown.stop @click="copyText">
          {{ isCopied ? '已复制' : '复制' }}
        </button>
        <button title="关闭" class="action-btn" @pointerdown.stop @click="emit('close')">✕</button>
      </div>
    </div>

    <!-- Content -->
    <div class="content-area">
      <div v-if="popup.status === 'error'" class="text-red-500 text-xs">
        {{ popup.errorMessage || '未知错误' }}
      </div>
      <template v-else-if="popup.translatedText">
        <!-- Source text (collapsible) -->
        <details v-if="popup.sourceText" class="source-details">
          <summary class="text-xs text-gray-400 cursor-pointer select-none">原文</summary>
          <pre class="text-xs text-gray-500 whitespace-pre-wrap mt-1">{{ popup.sourceText }}</pre>
        </details>
        <!-- Translated text -->
        <div class="whitespace-pre-wrap">{{ popup.translatedText }}</div>
      </template>
    </div>
  </div>
</template>

<style scoped>
.popup-container {
  width: 100%;
  max-height: 100%;
  border-radius: 14px;
  background-color: rgba(255, 255, 255, 0.98);
  box-shadow: 0 10px 25px rgba(0, 0, 0, 0.1);
  overflow: hidden;
  outline: none;
}

.header-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 16px;
  border-bottom: 1px solid #f3f4f6;
  cursor: move;
  user-select: none;
  touch-action: none;
  -webkit-user-select: none;
}

.flex-items-center {
  display: flex;
  align-items: center;
}

.gap-2 {
  gap: 8px;
}

.text-sm {
  font-size: 14px;
}

.font-medium {
  font-weight: 500;
}

.text-gray-700 {
  color: #374151;
}

.text-xs {
  font-size: 12px;
}

.text-gray-400 {
  color: #9ca3af;
}

.flex {
  display: flex;
  gap: 4px;
}

.action-btn {
  padding: 8px;
  font-size: 12px;
  border-radius: 6px;
  cursor: pointer;
  background: none;
  border: none;
  cursor: pointer;
}

.action-btn:hover {
  background-color: #f3f4f6;
}

.content-area {
  padding: 12px;
  overflow-y: auto;
  font-size: 14px;
  line-height: 1.625;
  max-height: calc(100vh - 76px);
  cursor: auto;
  user-select: text;
}

.text-red-500 {
  color: #ef4444;
}

.source-details {
  margin-bottom: 8px;
}

.text-green-600 {
  color: #16a34a;
}

.text-red-500 {
  color: #ef4444;
}

.status-streaming {
  color: #3b82f6;
  animation: pulse-anim 1.5s ease-in-out infinite;
}

@keyframes pulse-anim {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.4; }
}

.mt-1 {
  margin-top: 4px;
}

.whitespace-pre-wrap {
  white-space: pre-wrap;
}
</style>
