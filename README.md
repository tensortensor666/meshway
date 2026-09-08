# Meshway

Meshway 是一个本地运行的 AI API Gateway，将 OpenAI Chat Completions、OpenAI Responses 和 Anthropic Claude Messages 统一到一个可自托管的入口。

## 快速开始

普通用户无需安装 Rust 或 Node.js。Windows 下载 CI 生成的 `Meshway-Setup.exe` 安装；Linux 下载 `.deb` 后使用系统包管理器安装。安装包已包含管理后台，运行后访问 `http://127.0.0.1:8080/`。首次使用请将 Admin Token 设置为自定义值，不要继续使用默认的 `change-me`。

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

`.github/workflows/build.yml` 在 pull request、`main`/`master` push 和 `v*` tag 上运行。前端先由 CI 构建，再被打包进 Linux/Windows 安装包。GitHub Actions 上传：

- `meshway-linux-deb`：Debian/Ubuntu 安装包
- `meshway-windows-installer`：Windows 安装程序

Linux 安装后由 systemd 管理服务，配置文件位于 `/etc/meshway/meshway.env`，数据库位于 `/var/lib/meshway/data/meshway.db`。Windows 安装后程序和前端位于用户本地应用目录，数据库默认位于 `%LOCALAPPDATA%\Meshway\data\meshway.db`。

```bash
sudo dpkg -i meshway_0.1.0_amd64.deb
sudo systemctl status meshway
```

本地验证本提交时不运行 Rust 或前端编译，编译结果以 GitHub Actions 为准。

## License

MIT
