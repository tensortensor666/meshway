# Meshway

Meshway 是一个本地运行的 AI API Gateway，将 OpenAI Chat Completions、OpenAI Responses 和 Anthropic Claude Messages 统一到一个可自托管的入口。

## 快速开始

需要 Rust stable、Node.js 22+。下载 CI 的 Windows/Linux 发布压缩包，解压后直接运行其中的 `meshway`/`meshway.exe`。压缩包中的 `frontend/dist` 会由 Rust 后端直接托管，运行后访问 `http://127.0.0.1:8080/`。复制 `.env.example` 为 `.env`，至少修改 `MESHWAY_ADMIN_TOKEN`。

前端开发环境运行：

```bash
cd frontend
npm install
npm run dev
```

后台使用 `MESHWAY_ADMIN_TOKEN` 访问 `/api/admin/*`。先添加 Provider，再创建一个 `mw_` 开头的访问 Key。访问 Key 只在创建成功时明文返回一次，数据库仅保存 SHA-256 哈希。

## API

调用兼容接口时使用：

```text
Authorization: Bearer mw_xxxxx
```

支持的入口：

```text
POST /v1/chat/completions
POST /v1/responses
POST /v1/messages
```

Provider 的 `base_url` 应填写 API 根地址，例如 `https://api.openai.com/v1` 或 `https://api.anthropic.com/v1`。Meshway 会根据入口自动追加对应路径，并透传普通及流式响应。

## CI 构建

`.github/workflows/build.yml` 在 pull request、`main`/`master` push 和 `v*` tag 上运行。前端先由 CI 构建，再被打包进 Linux/Windows 发布压缩包。GitHub Actions 上传：

- `meshway-linux-x64`
- `meshway-windows-x64`
- `meshway-admin`：前端临时构建产物

Windows/Linux 发布压缩包的目录结构：

```text
meshway-windows-x64/
├── meshway.exe
└── frontend/dist/
```

本地验证本提交时不运行 Rust 或前端编译，编译结果以 GitHub Actions 为准。

## License

MIT
