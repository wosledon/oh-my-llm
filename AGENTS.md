# AGENTS.md

oh-my-llm 是一个 Tauri v2 桌面应用，作为本地 LLM 代理服务器，统一管理多个大模型 Provider，对外暴露 OpenAI / Anthropic 兼容 API。

## 技术栈

| 层          | 技术                | 版本      |
| ----------- | ------------------- | --------- |
| 桌面框架    | Tauri               | v2        |
| 前端        | React + TypeScript  | 19 + 5.8  |
| UI 组件库   | Fluent UI React v9  | ^9.56     |
| 状态管理    | Zustand             | ^5        |
| 路由        | React Router DOM    | ^7        |
| 构建        | Vite                | ^7        |
| 后端        | Rust (edition 2021) | -         |
| HTTP 服务   | Axum + Tokio        | 0.7 + 1.x |
| HTTP 客户端 | Reqwest             | 0.12      |
| 数据库      | SQLite (rusqlite)   | 0.31      |

完整架构设计见 [docs/architecture.md](docs/architecture.md)。

## 构建命令

```bash
# 安装前端依赖
npm install

# 开发模式 (前端 + Tauri 同时启动)
npm run tauri dev

# 仅前端开发
npm run dev

# 前端构建 (类型检查 + Vite 打包)
npm run build

# 生产构建
npm run tauri build
```

Tauri dev 模式下：Vite 运行在 `localhost:1420`，Rust 后端在 `src-tauri/` 中编译。

## 项目结构约定

```
src/                     # React 前端
  components/            # 可复用 UI 组件
  pages/                 # 路由页面组件
  stores/                # Zustand stores
  hooks/                 # 自定义 Hooks
  types/                 # TypeScript 类型定义

src-tauri/src/
  proxy/                 # Axum 代理服务器
    handlers/            # /v1/chat/completions, /v1/messages 等
    middleware/           # 日志中间件
  providers/             # 上游 LLM Provider 客户端
  protocol/              # OpenAI + Anthropic 类型定义 + 协议互转
  storage/               # SQLite 数据库操作 (rusqlite)
  commands/              # Tauri IPC Commands (前后端通信)
  logging/               # 请求/响应日志记录
  stats/                 # Token/费用统计聚合
  crypto.rs              # API Key AEAD 加密
```

## 关键约定

### 前后端通信
- 前端通过 `invoke("command_name", { args })` 调用 Rust 命令
- 所有 Command 函数签名必须是 `#[tauri::command] fn foo(state: State<AppState>, ...) -> Result<T, String>`
- 返回值必须实现 `serde::Serialize`，参数实现 `serde::Deserialize`
- 新增命令后需在 `lib.rs` 的 `generate_handler![]` 中注册

### 数据库
- 使用 SQLite，文件路径通过 `tauri::api::path::app_data_dir()` 获取
- 表结构定义见 [docs/architecture.md §4](docs/architecture.md)
- 首次运行时自动执行 DDL 迁移
- API Key 使用 `ring::aead` AES-256-GCM 加密存储

### 前端约定
- 路由使用 React Router v7，页面组件放 `pages/`
- 状态管理使用 Zustand，按领域拆分 store（proxy、providers、models、logs、usage）
- 所有 UI 使用 Fluent UI v9 组件，禁止自行手写样式替代
- 主题跟随系统，通过 `<FluentProvider theme={...}>` 包裹

### Rust 模块边界
- `commands/` 只处理 IPC 参数转换，调用 `storage/` 和 `proxy/` 模块，不直接操作数据库
- `proxy/handlers/` 处理 HTTP 请求解析，委托 `providers/` 发起上游请求
- `protocol/translator.rs` 负责 OpenAI ↔ Anthropic 协议互转
- `logging/recorder.rs` 在请求完成后异步写入 SQLite

### 代理服务器
- 启动时绑定 `127.0.0.1:{port}`，默认 11888，不允许外部连接
- 请求根据 `model` 字段查 `model_mappings` 表找到上游 Provider
- Provider 类型 (`openai` | `anthropic` | `openai_compatible`) 决定直接转发还是协议转换
- 端口变更时自动重启代理

## 当前状态

项目处于骨架阶段：前端仅有模板代码，后端仅有一个演示 `greet` 命令。下一步应按 [docs/architecture.md §12](docs/architecture.md) 从 Phase 1 开始实现。

## 注意事项

- **不要**在 Rust 命令中打印 API Key（日志脱敏）
- **不要**允许多个 Tauri Command 同时修改代理状态 —— 使用 `Arc<RwLock<ProxyState>>`
- Token 计数从上游响应 `usage` 字段提取，不要自行实现 Token 计算
- Fluent UI v9 使用 CSS-in-JS (Griffel)，不要混用普通 CSS 文件覆盖组件样式
- 前端构建前必须通过 `tsc` 类型检查（`npm run build` 已包含）
