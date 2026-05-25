# TODO.md - Local Bubble Translator 执行清单

最后更新：2026-05-25

本文件是后续开发的约束清单。弱模型继续工作时，必须先读 `docs/DEVELOPMENT.md` 和本文件，再改代码。

---

## 0. 产品目标修正

核心目标不是“菜单栏应用菜单常驻”，而是：

- macOS 屏幕右上角状态栏常驻一个小图标，也就是 Tauri tray/status item。
- 启动后主设置窗口默认隐藏。
- 左键点击状态栏图标：打开设置窗口。
- 右键点击状态栏图标：弹出菜单，至少包含“打开设置页”和“退出”。
- 后续再把“翻译选中文本”“截图翻译”接到同一个状态栏菜单和全局快捷键上。

不要把“状态栏图标”误解成 macOS 顶部应用菜单里的 File/Edit 菜单。

---

## 1. 当前事实

### 已验证可用

- [x] Tauri v2 + Vue 3 + TypeScript + UnoCSS 项目可构建。
- [x] `npm run dev` 会通过 `beforeDevCommand` 自动启动 Vite。
- [x] `npm run build:frontend` 通过。
- [x] `npx vue-tsc --noEmit` 通过。
- [x] Rust/Tauri 已有主窗口 `main`，作为设置页窗口使用。
- [x] 启动时设置窗口默认隐藏。
- [x] 已创建状态栏 tray/status 图标。
- [x] 状态栏图标菜单已包含：打开设置页、翻译选中文本、截图翻译、退出。
- [x] 左键状态栏图标会打开设置页；右键状态栏图标显示菜单。
- [x] 状态栏菜单“翻译选中文本”已触发第一版 Command+C 选中文本流程。
- [x] `translateTextStream()` 已实现 OpenAI-compatible SSE 文本翻译。
- [x] `translateImageStream()` 已实现 OpenAI-compatible SSE 图片翻译请求。
- [x] 设置页可以配置 targetLanguage / textModel / visionModel，并可测试文本翻译。
- [x] 设置页支持输入 OpenAI-compatible API 地址和 API Key 后拉取模型列表。

### 仍是占位或未接通

- [x] Rust 端已实现并注册 `get_selected_text` 第一版 Command+C 策略。
- [x] Rust 端已实现并注册 `start_screenshot_selection` 第一版 macOS `screencapture -i` 策略。
- [x] Rust 端已实现并注册第一版权限检测 commands。
- [x] 状态栏菜单里的“截图翻译”已触发第一版截图翻译流程。
- [x] 全局快捷键 `Command+E` 已触发翻译选中文本。
- [x] 全局快捷键 `Command+R` 已触发第一版截图翻译流程。
- [x] 气泡窗口已作为独立 Tauri WebviewWindow 创建。
- [x] `PopupWindow.vue` 和 `popup-store.ts` 已接入基础窗口生命周期和流式事件。
- [x] `Command+E` 已支持打开/关闭翻译气泡。
- [x] `Command+Q` 已改为二次确认退出，避免误操作。
- [x] 配置已迁移到 app data 目录的 JSON 文件，不保存翻译数据。

---

## 2. 开发硬约束

- 第一版只做 macOS。
- 不引入 Electron。
- 不做 OCR。截图翻译必须直接把图片发给支持视觉能力的 OpenAI-compatible 模型。
- 不做历史记录、收藏、发音、词典、登录、云同步、自动更新、插件系统。
- 不实现自动划词弹小图标，也不做鼠标松开自动检测选区。
- 不大改 UI 风格。当前阶段优先打通核心流程。
- 每一步完成后至少跑：
  - `npm run build:frontend`
  - `npx vue-tsc --noEmit`
  - Rust 相关改动必须跑 `cargo check`，工作目录为 `src-tauri`
- 如果 `npm run dev` 在沙箱内因为端口监听报 `EPERM`，需要在沙箱外授权运行验证。

---

## 3. 阶段 1 - 基础启动与状态栏入口

状态：已完成，后续只允许修 bug，不要重构。

- [x] `package.json` 增加 `dev:frontend` / `build:frontend`。
- [x] `tauri.conf.json` 增加 `beforeDevCommand` / `beforeBuildCommand`。
- [x] `main` 设置窗口启动隐藏。
- [x] 创建 macOS 状态栏图标。
- [x] 状态栏图标左键打开设置页。
- [x] 状态栏图标右键显示菜单。
- [x] 状态栏菜单提供“打开设置页”和“退出”。

验收标准：

- [x] `npm run dev` 能启动 Vite 和 Tauri。
- [x] 启动后屏幕右上角能看到状态栏图标。
- [x] 点击状态栏入口能打开设置窗口。

---

## 4. 阶段 2 - 配置系统

状态：基础完成。

- [x] `AppConfig` / `OpenAICompatibleModelConfig` 类型。
- [x] `defaultConfig`。
- [x] 设置页可编辑目标语言、文本模型、视觉模型。
- [x] 已从 `localStorage` 迁移为 Tauri/Rust command 保存到：
  `~/Library/Application Support/LocalBubbleTranslator/config.json`
- [x] 保存前做最小校验：baseUrl 非空、model 非空、temperature 在 0 到 2。
- [x] 旧版 `localStorage` 配置会一次性迁移到 JSON 文件并删除旧配置。

验收标准：

- [x] 修改配置、关闭窗口、重新打开后配置仍保留。
- [x] 损坏配置文件不能导致白屏，应回退到默认配置。

---

## 5. 阶段 3 - 文本模型流式翻译

状态：基础完成，后续需要增强健壮性。

- [x] `translateTextStream()`。
- [x] SSE `data:` 解析。
- [x] 设置页“测试翻译”按钮。
- [x] 支持非 SSE 错误响应读取 body，错误消息更明确。
- [x] 支持用户取消当前流式请求，避免窗口关闭后还在追加内容。

验收标准：

- 本地 OpenAI-compatible 服务启动时，输入文本能流式返回翻译。
- 服务未启动时，UI 明确提示连接失败。

---

## 6. 阶段 4 - 选中文本翻译

状态：基础完成，后续继续补权限提示和边界优化。

实现顺序：

1. Rust 实现 `get_selected_text` command。
2. 第一版用 Command+C 策略：
   保存剪贴板 -> 模拟 Command+C -> 等待 80-150ms -> 读取文本 -> 恢复剪贴板。
3. 注册 Tauri command，并让 `src/services/tauri/commands.ts` 能真实调用。
4. 状态栏菜单“翻译选中文本”触发该 command。
5. 前端拿到文本后调用 `translateTextStream()`。
6. 将流式结果显示到气泡窗口。

当前状态：

- [x] `get_selected_text` command 已实现并注册。
- [x] 已采用 Command+C 策略，并尽量恢复原剪贴板。
- [x] 状态栏菜单“翻译选中文本”已触发 command。
- [x] Rust 获取选中文本后直接创建独立 popup 窗口。
- [x] popup 窗口读取 URL 参数后自行调用 `translateTextStream()`。
- [x] `Command+E` 再次触发时关闭当前 popup。

必须处理：

- 未选中文字：提示“未检测到选中文本”。
- 没有辅助功能权限：提示“缺少辅助功能权限，请在系统设置中授权”。
- 剪贴板原内容必须尽量恢复。

验收标准：

- 在任意 App 选中文本后，从状态栏菜单触发翻译，能看到流式结果。
- 原剪贴板内容不会被永久覆盖。

---

## 7. 阶段 5 - 气泡窗口

状态：文本翻译气泡基础完成，截图翻译后续复用。

- [x] `PopupWindow.vue` 基础 UI。
- [x] `popup-store.ts` 基础状态管理。
- [x] 创建独立 Tauri `popup` WebviewWindow。
- [x] popup 无边框、置顶、圆角卡片视觉。
- [x] 支持 appendDelta 流式追加。
- [x] 支持复制、关闭。
- [x] 支持拖动气泡窗口。
- [x] 点击外部/失焦关闭气泡窗口。
- [x] Esc 关闭当前气泡窗口。
- [x] macOS 下启用透明 popup 窗口，避免气泡外出现白色背景。

验收标准：

- 翻译开始时立即出现气泡。
- 流式内容逐字/逐段追加，不等全部完成才显示。
- 错误时保留已收到内容，并显示“翻译中断”或具体错误。

---

## 8. 阶段 6 - 全局快捷键

状态：未开始。

- [x] 注册 `Command+E`：翻译选中文本。
- [x] 注册 `Command+R`：截图翻译。
- [ ] 快捷键失败或冲突时，在设置页显示状态。

验收标准：

- App 在后台时，快捷键仍能触发。
- 不使用 `Command+S` 等高冲突快捷键。

---

## 9. 阶段 7 - 截图翻译

状态：第一版基础流程已接通，当前使用 macOS 系统截图框选能力，后续再考虑自定义截图遮罩窗口。

- [x] `translateImageStream()`。
- [ ] 创建截图遮罩窗口。
- [x] 支持用户拖拽框选区域（第一版通过 macOS `screencapture -i`）。
- [x] Rust 截取所选屏幕区域并返回 base64 PNG。
- [x] 调用视觉模型流式翻译。
- [x] 结果显示到气泡窗口。
- [x] 截图取消时不弹错误气泡。
- [x] 截图模型配置为空时提示用户配置视觉模型。
- [x] 视觉模型空响应时回退显示“未识别到可翻译文字”。
- [x] 未授权屏幕录制导致截图失败时，提示用户授权屏幕录制权限。
- [x] 截图图片不进入 popup 状态，不做本地持久化。

验收标准：

- 截图中有文字时，模型输出目标语言翻译。
- 图片中无可读文字时，输出“未识别到可翻译文字”。
- 当前模型不支持图片时，提示用户配置视觉模型。
- 翻译文字、截图图片、翻译结果均不做本地持久化。

---

## 10. 阶段 8 - 权限检测与引导

状态：基础完成，后续继续优化缺权限时的上下文提示。

- [x] 检测辅助功能权限。
- [x] 检测屏幕录制权限。
- [x] 设置页显示权限状态。
- [x] 提供打开系统设置按钮：
  - 辅助功能
  - 屏幕录制

验收标准：

- 缺权限时不静默失败。
- [x] 用户能从设置页直接跳到对应系统设置页面。

---

## 11. 阶段 9 - 打包

状态：基础打包完成，GitHub Release 当前只发布 macOS `.dmg`；Windows 暂缓发布，等有测试设备后再恢复验证。

- [x] 确认 app 名称、bundle identifier、图标。
- [x] 生成 `.app`。
- [x] 生成 `.dmg`。
- [x] 增加 `npm run build:mac`。
- [x] 增加 `npm run build:windows`，目标为 `x86_64-pc-windows-msvc` 的 NSIS/MSI 安装包。
- [x] GitHub Actions release workflow 当前仅使用 `macos-latest` 构建 `.dmg`。
- [ ] 在干净 macOS 用户环境验证首次启动、权限提示、状态栏图标、设置页。
- [ ] 在 Windows/MSVC 环境验证 `npm run build:windows`，并确认状态栏、快捷键、截图策略是否需要 Windows 原生适配。

Windows 构建说明：

- 在没有 Windows 测试设备前，不发布 Windows release。
- 当前只修复和优化 macOS 版本；Windows 脚本仅作为未来准备，不接入 release 矩阵。
- 推荐在 Windows 机器或 Windows CI 中构建，不把 macOS 交叉编译 Windows 安装包作为默认路径。
- 需要 Node.js、pnpm、Rust MSVC 工具链、Visual Studio Build Tools、WebView2 运行时/再发行组件支持。
- 当前应用多处功能仍以 macOS 第一版为主；Windows 安装包构建通过后，还需要逐项验证选中文本、截图、权限提示和托盘行为。

---

## 12. 已解决的问题记录

1. `npm run dev` 一直等待 `http://localhost:1420`
   - 原因：Tauri 配置缺少 `beforeDevCommand`，Vite 没有被启动。
   - 修复：`tauri.conf.json` 增加 `beforeDevCommand: npm run dev:frontend`。

2. `PopupWindow.vue` 类型检查失败
   - 原因：使用 `ref` 但未导入。
   - 修复：从 Vue 导入 `ref`。

3. 关闭翻译气泡后透明窗口仍挡住点击
   - 原因：popup 宿主窗口关闭依赖前端 `destroy()`，失败时 WebviewWindow 会残留。
   - 修复：新增 Rust `close_popup_window` command，由原生侧销毁 popup 宿主窗口。

4. 设置窗口关闭后无法再次打开
   - 原因：系统关闭事件可能销毁 `main` 设置窗口，状态栏再次点击时找不到窗口。
   - 修复：Rust 拦截 `main` 的关闭事件改为隐藏；如果窗口不存在则重建。

5. 设置页“取消/保存配置”不关闭窗口
   - 原因：子组件直接操作当前窗口句柄，某些生命周期下关闭行为不稳定。
   - 修复：设置页只发出 `close` 事件，由父组件统一隐藏 `main` 窗口。

3. `popup-store.ts` 引用未安装的 Pinia
   - 原因：`package.json` 没有 `pinia`，文档也只说 Pinia 可选。
   - 修复：改为 Vue `reactive` 小 store。

4. 翻译模块类型导入错误
   - 原因：`OpenAICompatibleModelConfig` 定义在 `app-config.ts`，不是 `translator-types.ts`。
   - 修复：修正 `text-translator.ts` / `image-translator.ts` 的导入。

5. 状态栏入口目标被误写成"菜单栏"
   - 原因：TODO 早期表述不精确。
   - 修复：明确为 macOS 右上角 tray/status item，主窗口默认隐藏。

6. **测试模型连接报错 "Load failed"**
   - 原因：模型服务未运行或 API 地址不可达。
   - 修复：改进错误提示，显示具体无法连接的地址和服务状态。

7. 翻译气泡无法稳定拖动
   - 原因：透明无边框 popup 依赖 `startDragging()` 时，在 macOS 上可能无法可靠触发拖动；前端 Tauri window API 兜底也容易被原生拖动调用和失焦关闭逻辑打断。
   - 修复：撤掉前端混合拖动，改为标题栏 pointer 事件调用 Rust command，由原生侧根据鼠标坐标移动 popup 窗口；拖动期间暂缓失焦关闭。

8. 截图翻译在代码/英文注释混合场景偶尔只展示原文
   - 原因：视觉翻译 prompt 没有明确区分代码结构和自然语言内容，模型可能把任务当成截图转写。
   - 修复：强化截图翻译 system/user prompt，要求保留代码、变量名、路径，翻译注释、说明、错误信息和其他自然语言，并禁止只转写原文。

9. 翻译气泡出现额外描边
   - 原因：气泡卡片保留了 1px 边框，在透明无边框宿主窗口中会显得像异常描边。
   - 修复：移除气泡卡片边框，只保留阴影和圆角背景。

---

## 13. 已知问题 - Clipboard API Crash (已解决)

**症状：** `fatal runtime error: Rust cannot catch foreign exceptions, aborting`
当点击状态栏菜单"翻译选中文本"时，Rust 进程直接崩溃。

**原因：** macOS Accessibility API (`AXUIElementCopyAttributeValue`、`msg_send!`) 在调用失败时可能抛出异常，Rust 无法捕获。

**修复方案：** `src-tauri/src/commands/clipboard.rs`
- 用 `std::panic::catch_unwind` 包裹所有 Objective-C / Accessibility API 调用
- 优先尝试 AX API（直接获取选中文本），失败后 fallback 到 Command+C 策略
- Command+C 策略：保存剪贴板 -> AppleScript 模拟 Cmd+C -> 等待 -> pbpaste 读取 -> 恢复剪贴板

---

## 14. 历史建议任务记录

阶段 4 起步时的建议任务：

- 先只实现 Command+C 获取选中文本。
- 不要先做 Accessibility API 深度读取。
- 不要先做截图。
- 不要先重构 UI。

完成后再把状态栏菜单“翻译选中文本”和 `Command+E` 接上同一个流程。
