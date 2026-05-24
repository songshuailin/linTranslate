# LinTranslate / 灵译

LinTranslate is a lightweight macOS desktop translator for quick selected-text and screenshot translation. It uses OpenAI-compatible API providers, so you can connect it to local models, self-hosted gateways, or cloud services that expose `/v1/chat/completions`.

灵译是一款轻量 macOS 桌面翻译工具，支持划词翻译和截图翻译。它通过 OpenAI-compatible API 接入模型服务，可以连接本地模型、自建网关，或兼容 `/v1/chat/completions` 的云端服务。

## Features / 功能

- Selected-text translation from a global shortcut.
- Screenshot translation from a global shortcut.
- OpenAI-compatible API configuration.
- Separate text and vision model selection.
- Local user configuration; API URL and API Key are not bundled into the app.
- GitHub Releases based version checking.

- 全局快捷键划词翻译。
- 全局快捷键截图翻译。
- 支持 OpenAI-compatible API 配置。
- 支持分别选择文本模型和视觉模型。
- 配置仅保存在用户本地，API URL 和 API Key 不会打包进应用。
- 支持基于 GitHub Releases 的版本检查。

## Shortcuts / 快捷键

- `Command + E`: translate selected text.
- `Command + R`: screenshot translation.

- `Command + E`：翻译选中文本。
- `Command + R`：截图翻译。

## API Setup / API 设置

Use an OpenAI-compatible base URL, for example:

填写 OpenAI-compatible API 地址，例如：

```text
http://127.0.0.1:8888/v1
```

If your provider requires authentication, enter the API Key in Settings. The key is stored only in the current user's local config directory.

如果你的服务需要鉴权，请在设置页填写 API Key。密钥只会保存在当前用户的本地配置目录中。

## Development / 开发

Requirements:

依赖环境：

- Node.js
- pnpm
- Rust and Cargo

Install dependencies:

安装依赖：

```bash
pnpm install
```

Run in development:

开发运行：

```bash
npm run dev
```

Build the app:

构建应用：

```bash
npm run build
```

## Release / 发布

Create and push a version tag to publish a GitHub Release:

推送版本 tag 即可触发 GitHub Release：

```bash
git tag v0.1.0
git push origin v0.1.0
```

## License / 许可证

MIT
