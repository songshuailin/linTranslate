import type { OpenAICompatibleModelConfig } from '../config/app-config'
import type { TranslateImageInput, TranslationStreamOptions } from './translator-types'
import { buildApiErrorMessage } from './api-errors'

export function buildImageTranslateSystemPrompt(targetLanguage: string): string {
  return `你是一个截图文字翻译引擎。

任务：识别图片中的可见文字，并把需要阅读理解的自然语言内容翻译成${targetLanguage}。

规则：
- 只输出翻译结果。不要描述图片、解释画面或前缀。
- 不要只转写原文；只要图片里有可翻译文字，就必须输出${targetLanguage}译文。
- 如果图片中没有可读文字，只输出：未识别到可翻译文字
- 尽量按阅读顺序输出，保留换行和段落结构。
- 对明显的 UI 按钮、菜单、错误提示，直接翻译其文字含义。
- 遇到代码、命令、路径、变量名、函数名、API 名称时，保留这些专有内容。
- 遇到代码注释、文档字符串、日志、错误信息、提交信息、README、界面文案等自然语言时，翻译成${targetLanguage}。
- 截图中如果夹杂代码和英文注释，保留代码结构并翻译注释/说明文字；不要因为有代码就整体输出原文。`.trim()
}

export async function* translateImageStream(
  config: OpenAICompatibleModelConfig,
  input: TranslateImageInput,
  options: TranslationStreamOptions = {},
): AsyncGenerator<string> {
  if (!config.baseUrl.trim() || !config.model.trim()) {
    throw new Error('请先在设置页配置截图翻译模型')
  }

  const url = `${config.baseUrl.replace(/\/$/, '')}/chat/completions`

  const body = {
    model: config.model,
    stream: true,
    temperature: config.temperature ?? 0.2,
    messages: [
      { role: 'system', content: buildImageTranslateSystemPrompt(input.targetLanguage) },
      {
        role: 'user',
        content: [
          {
            type: 'text',
            text: `请识别图片中的可见文字，并翻译成${input.targetLanguage}。如果包含代码或英文注释，请保留代码、变量名和路径，翻译注释、说明、错误信息和其他自然语言。只输出翻译结果，不要原文转写。`,
          },
          { type: 'image_url', image_url: { url: input.imageBase64.startsWith('data:') ? input.imageBase64 : `data:image/png;base64,${input.imageBase64}` } },
        ],
      },
    ],
  }
  const headers: Record<string, string> = { 'Content-Type': 'application/json' }
  const apiKey = config.apiKey.trim()
  if (apiKey) headers.Authorization = `Bearer ${apiKey}`

  const response = await fetch(url, {
    method: 'POST',
    headers,
    body: JSON.stringify(body),
    signal: options.signal,
  })

  if (!response.ok) throw new Error(await buildApiErrorMessage('截图翻译接口错误', response))
  if (!response.body) throw new Error('Vision translation API response body is empty')

  const reader = response.body.getReader()
  const decoder = new TextDecoder('utf-8')
  let buffer = ''

  while (true) {
    const { done, value } = await reader.read()
    if (done) break
    buffer += decoder.decode(value, { stream: true })
    const lines = buffer.split('\n')
    buffer = lines.pop() || ''
    for (const line of lines) {
      const trimmed = line.trim()
      if (!trimmed || !trimmed.startsWith('data:')) continue
      const data = trimmed.slice(5).trim()
      if (data === '[DONE]') return
      try {
        const json = JSON.parse(data)
        const delta = json.choices?.[0]?.delta?.content
        if (delta) yield delta
      } catch { /* ignore incomplete SSE fragments */ }
    }
  }
}
