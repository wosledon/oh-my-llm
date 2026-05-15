# oh-my-llm 架构设计

## 1. 项目定位

统一大模型代理平台，在本地运行一个 HTTP 代理服务器，对上游 LLM Provider（OpenAI、Anthropic 及兼容厂商）进行统一管理。外部工具只需要配置一个固定地址，即可访问所有已购买的大模型，无需逐个配置 API Key。

- 同时支持 **OpenAI 协议** 和 **Anthropic 协议**
- 自动协议转换：OpenAI 客户端可调用 Anthropic 模型，反之亦然
- 基于 **Tauri v2 + React 19 + Rust** 的桌面应用
- UI 遵循 **Fluent Design System (Fluent UI v9)**

---

## 2. 整体架构

```
┌─────────────────────────────────────────────────────┐
│                   Tauri Desktop App                  │
│  ┌──────────────────┐   ┌─────────────────────────┐ │
│  │  React Frontend  │   │    Rust Backend          │ │
│  │  (Fluent UI v9)  │◄──┤    (Tauri Commands)      │ │
│  │                  │IPC│                          │ │
│  │  • Provider配置   │   │  ┌───────────────────┐  │ │
│  │  • 模型映射       │   │  │  HTTP Proxy Server │  │ │
│  │  • 代理设置       │   │  │  (Axum + Tokio)    │  │ │
│  │  • 请求日志       │   │  │  :11888            │  │ │
│  └──────────────────┘   │  └──────┬────────────┘  │ │
│                          │         │                │ │
│                          │  ┌──────▼────────────┐  │ │
│                          │  │  Protocol Router   │  │ │
│                          │  │  + Translator      │  │ │
│                          │  └──────┬────────────┘  │ │
│                          │         │                │ │
│                          │  ┌──────▼────────────┐  │ │
│                          │  │  Provider Clients  │  │ │
│                          │  │  • OpenAI          │  │ │
│                          │  │  • Anthropic       │  │ │
│                          │  │  • Compatible      │  │ │
│                          │  └──────┬────────────┘  │ │
│                          │         │                │ │
│                          │  ┌──────▼────────────┐  │ │
│                          │  │  SQLite Storage    │  │ │
│                          │  └───────────────────┘  │ │
│                          └─────────────────────────┘ │
└─────────────────────────────────────────────────────┘
          ▲                          │
          │    OpenAI/Anthropic      │
          │    Protocol Requests     │
          │                          ▼
    ┌──────────┐          ┌──────────────────┐
    │  Client   │          │  Upstream LLM     │
    │  Tools    │          │  Providers        │
    │  (Cursor, │          │  • OpenAI         │
    │   Copilot,│          │  • Anthropic      │
    │   etc.)   │          │  • Azure OpenAI   │
    └──────────┘          │  • 阿里百炼       │
                          │  • DeepSeek       │
                          │  • Ollama         │
                          └──────────────────┘
```

**核心流程**：客户端工具配置 `http://localhost:11888/v1` 作为 API Base URL → 代理根据 `model` 字段路由到对应 Provider → 协议自动转换 → 返回统一格式响应。

---

## 3. 后端 Rust 模块设计

### 3.1 目录结构

```
src-tauri/src/
├── main.rs                  # 入口 + 启动代理
├── lib.rs                   # Tauri Builder + 注册命令
├── proxy/
│   ├── mod.rs               # 代理模块入口
│   ├── server.rs            # Axum HTTP Server (启动/停止)
│   ├── router.rs            # 请求路由 (模型→Provider映射)
│   ├── handlers/
│   │   ├── mod.rs
│   │   ├── openai.rs        # /v1/chat/completions 等
│   │   ├── anthropic.rs     # /v1/messages
│   │   └── models.rs        # /v1/models 模型列表
│   └── middleware/
│       ├── mod.rs
│       └── logging.rs       # 请求日志中间件
├── providers/
│   ├── mod.rs               # Provider trait 定义
│   ├── openai.rs            # OpenAI 原生客户端
│   ├── anthropic.rs         # Anthropic 原生客户端
│   └── compatible.rs        # OpenAI 兼容客户端 (通用)
├── logging/
│   ├── mod.rs               # 日志模块入口
│   ├── recorder.rs           # 请求/响应完整记录 (插入 SQLite)
│   ├── fts.rs                # FTS5 全文索引维护
│   └── retention.rs          # 过期日志清理
├── stats/
│   ├── mod.rs               # 统计模块入口
│   ├── aggregator.rs         # Token/费用 日聚合
│   └── calculator.rs         # 费用计算 (input_price × tokens + output_price × tokens)
├── protocol/
│   ├── mod.rs
│   ├── openai_types.rs      # OpenAI 请求/响应类型
│   ├── anthropic_types.rs   # Anthropic 请求/响应类型
│   └── translator.rs        # 协议互转 (OpenAI↔Anthropic)
├── storage/
│   ├── mod.rs
│   ├── db.rs                # SQLite 初始化 + 迁移
│   ├── provider_repo.rs     # Provider CRUD
│   ├── model_repo.rs        # Model CRUD
│   └── config_repo.rs       # Proxy 配置 CRUD
├── commands/
│   ├── mod.rs
│   ├── provider_cmd.rs      # Tauri Commands: Provider 管理
│   ├── model_cmd.rs         # Tauri Commands: Model 管理
│   ├── proxy_cmd.rs         # Tauri Commands: 代理控制
│   ├── log_cmd.rs           # Tauri Commands: 日志查询
│   └── stats_cmd.rs         # Tauri Commands: Token/费用统计
└── crypto.rs                # API Key 加密存储
```

### 3.2 Cargo 依赖

```toml
[dependencies]
tauri = { version = "2", features = [] }
tauri-plugin-opener = "2"
tauri-plugin-autostart = "2"  # 系统开机自启
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tokio = { version = "1", features = ["full"] }
axum = { version = "0.7", features = ["json"] }
axum-extra = "0.9"
tower = "0.4"
tower-http = { version = "0.5", features = ["cors"] }
reqwest = { version = "0.12", features = ["json", "stream"] }
rusqlite = { version = "0.31", features = ["bundled"] }
uuid = { version = "1", features = ["v4"] }
chrono = { version = "0.4", features = ["serde"] }
ring = "0.17"              # AEAD 加密 API Key
base64 = "0.22"
thiserror = "2"
tracing = "0.1"
tracing-subscriber = "0.3"
tauri-plugin-autostart = "2"  # 开机自启
```

---

## 4. 数据库设计 (SQLite)

```sql
-- Provider 配置
CREATE TABLE providers (
    id          TEXT PRIMARY KEY,            -- UUID
    name        TEXT NOT NULL,               -- 显示名称, 如 "DeepSeek"
    prov_type   TEXT NOT NULL,               -- 'openai' | 'anthropic' | 'openai_compatible'
    base_url    TEXT NOT NULL,               -- https://api.deepseek.com/v1
    api_key     BLOB NOT NULL,               -- AEAD 加密存储
    extra_headers TEXT,                      -- JSON: {"X-Custom":"val"}
    created_at  INTEGER NOT NULL,
    updated_at  INTEGER NOT NULL
);

-- 模型映射 (暴露给客户端的模型名 → 上游实际模型名 + 定价)
CREATE TABLE model_mappings (
    id              TEXT PRIMARY KEY,
    provider_id     TEXT NOT NULL REFERENCES providers(id) ON DELETE CASCADE,
    exposed_name    TEXT NOT NULL,           -- 客户端看到的模型名, 如 "gpt-4"
    upstream_name   TEXT NOT NULL,           -- 上游实际名, 如 "gpt-4-0613"
    enabled         INTEGER NOT NULL DEFAULT 1,
    input_price     REAL NOT NULL DEFAULT 0, -- 输入价格 (USD / 1M tokens)
    output_price    REAL NOT NULL DEFAULT 0, -- 输出价格 (USD / 1M tokens)
    UNIQUE(exposed_name, provider_id)
);

-- 代理配置 (单行配置表)
CREATE TABLE proxy_config (
    id                  INTEGER PRIMARY KEY DEFAULT 1,
    port                INTEGER NOT NULL DEFAULT 11888,
    openai_enabled      INTEGER NOT NULL DEFAULT 1,
    anthropic_enabled   INTEGER NOT NULL DEFAULT 1,
    default_model       TEXT,
    auto_start          INTEGER NOT NULL DEFAULT 0,  -- 开机自启
    log_requests        INTEGER NOT NULL DEFAULT 1,
    log_retention_days  INTEGER NOT NULL DEFAULT 30,-- 日志保留天数
    budget_enabled      INTEGER NOT NULL DEFAULT 0, -- 预算限制开关
    budget_monthly      REAL NOT NULL DEFAULT 0,    -- 月度预算 (USD)
    budget_warning      REAL NOT NULL DEFAULT 0.8,  -- 预警比例
    max_retries         INTEGER NOT NULL DEFAULT 3,
    timeout_secs        INTEGER NOT NULL DEFAULT 120
);

-- 请求日志 (主表)
CREATE TABLE request_logs (
    id              TEXT PRIMARY KEY,           -- UUID
    timestamp       INTEGER NOT NULL,           -- Unix 毫秒
    protocol        TEXT NOT NULL,              -- 'openai' | 'anthropic'
    model           TEXT NOT NULL,              -- 触发路由的模型名
    provider_id     TEXT,                       -- 上游 Provider ID
    upstream_model  TEXT,                       -- 实际请求的上游模型名
    stream          INTEGER NOT NULL DEFAULT 0, -- 是否流式请求
    latency_ms      INTEGER,                    -- 端到端延迟(ms)
    status_code     INTEGER,                    -- HTTP 状态码
    prompt_tokens   INTEGER,                    -- 输入 Token 数
    completion_tokens INTEGER,                  -- 输出 Token 数
    cost            REAL,                       -- 预估费用(USD)
    error_type      TEXT,                       -- 错误类型: 'auth'|'rate_limit'|'timeout'|'parse'|'upstream'|'other'
    error_message   TEXT,                       -- 错误详情
    created_at      INTEGER NOT NULL
);
CREATE INDEX idx_req_logs_ts ON request_logs(timestamp DESC);
CREATE INDEX idx_req_logs_model ON request_logs(model);

-- 请求内容 (完整请求体 + 消息历史)
CREATE TABLE request_contents (
    id              TEXT PRIMARY KEY,
    log_id          TEXT NOT NULL UNIQUE REFERENCES request_logs(id) ON DELETE CASCADE,
    request_body    TEXT NOT NULL,              -- JSON: 完整请求体 (含 messages、参数等)
    extracted_text  TEXT,                       -- 提取的所有对话文本，方便全文检索
    created_at      INTEGER NOT NULL
);

-- 响应内容 (完整响应体)
CREATE TABLE response_contents (
    id              TEXT PRIMARY KEY,
    log_id          TEXT NOT NULL UNIQUE REFERENCES request_logs(id) ON DELETE CASCADE,
    response_body   TEXT NOT NULL,              -- JSON: 完整响应体
    extracted_text  TEXT,                       -- 提取的响应文本，方便全文检索
    is_truncated    INTEGER NOT NULL DEFAULT 0, -- 是否被截断 (stream 模式下可能很长)
    created_at      INTEGER NOT NULL
);

-- 全文搜索索引 (FTS5)
CREATE VIRTUAL TABLE logs_fts USING fts5(
    log_id UNINDEXED,
    model,
    request_text,
    response_text,
    content='request_contents',
    content_rowid='rowid'
);

-- 用量聚合统计 (按日 + 模型)
CREATE TABLE daily_usage (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    date            TEXT NOT NULL,              -- '2026-05-16'
    model           TEXT NOT NULL,              -- 客户端模型名
    provider_id     TEXT NOT NULL,
    request_count   INTEGER NOT NULL DEFAULT 0, -- 请求次数
    prompt_tokens   INTEGER NOT NULL DEFAULT 0, -- 输入 Token 总量
    completion_tokens INTEGER NOT NULL DEFAULT 0,-- 输出 Token 总量
    cost            REAL NOT NULL DEFAULT 0,    -- 费用 (USD)
    UNIQUE(date, model, provider_id)
);
CREATE INDEX idx_daily_usage_date ON daily_usage(date DESC);
CREATE INDEX idx_daily_usage_model ON daily_usage(model);

-- 月度预算配置
CREATE TABLE budget_config (
    id                  INTEGER PRIMARY KEY DEFAULT 1,
    monthly_budget      REAL NOT NULL DEFAULT 0,   -- 月度预算上限 (USD), 0=不限制
    warning_threshold   REAL NOT NULL DEFAULT 0.8, -- 预警比例, 默认 80%
    enabled             INTEGER NOT NULL DEFAULT 0,-- 预算限制是否启用
    last_reset_date     TEXT                       -- 上次重置月
);
```
```

---

## 5. 前端设计 (Fluent UI React v9)

### 5.1 组件树

```
<FluentProvider theme={...}>
  <Shell>                              ← Mica 背景, 左侧导航
    <Sidebar>
      <NavCategory>
        <NavItem icon={Home} to="/">Dashboard</NavItem>
        <NavItem icon={DataUsage} to="/usage">Usage</NavItem>
        <NavItem icon={Server} to="/providers">Providers</NavItem>
        <NavItem icon={Cube} to="/models">Models</NavItem>
        <NavItem icon={Settings} to="/settings">Proxy Settings</NavItem>
        <NavItem icon={History} to="/logs">Request Logs</NavItem>
      </NavCategory>
      <ProxyStatusBadge />             ← 代理运行状态指示灯
    </Sidebar>

    <Content>                           ← react-router 渲染
      <Routes>
        <Route path="/" element={<Dashboard />} />
        <Route path="/usage" element={<UsagePage />} />
        <Route path="/providers" element={<ProvidersPage />} />
        <Route path="/models" element={<ModelsPage />} />
        <Route path="/settings" element={<SettingsPage />} />
        <Route path="/logs" element={<LogsPage />} />
      </Routes>
    </Content>
  </Shell>

  <Toaster />                          ← 通知提示
</FluentProvider>
```

### 5.2 各页面设计

| 页面          | 核心组件                                                                    | 功能                                                                    |
| ------------- | --------------------------------------------------------------------------- | ----------------------------------------------------------------------- |
| **Dashboard** | `ProxyToggle` + `TokenGauge` + `CostRing` + `BudgetBar` + `RecentLogsTable` | 概览看板：启停开关、今日 Token/费用、月度预算进度、热门模型 Top N       |
| **Usage**     | `LineChart`(日/周/月) + `ModelBreakdown`(饼图/柱状) + `BudgetTracker`       | Token&费用分析：趋势曲线、按模型/Provider 拆分、月度预算追踪            |
| **Providers** | `DataGrid`, `Dialog`(新增/编辑表单)                                         | 添加、编辑、删除 Provider, 测试连接                                     |
| **Models**    | `DataGrid`, `Dialog`                                                        | 模型映射管理, 设置暴露别名、上游实际模型名、输入/输出价格               |
| **Settings**  | `Input`(端口), `Switch`(协议开关/开机自启/预算开关), `Dropdown`(默认模型)   | 代理参数配置：端口、协议开关、开机自启、预算上限、默认模型              |
| **Logs**      | `DataGrid` + `FilterBar` + `LogDetailPanel`(侧边/底部展开抽屉)              | 按时间/模型/状态/关键词检索日志，点击行展开完整请求体、响应体与对话内容 |

### 5.2.1 Dashboard 看板设计

```
┌─ Dashboard ───────────────────────────────────────────────────┐
│                                                                │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌───────────────┐  │
│  │ 代理状态  │  │ 今日 Token│  │ 今日费用  │  │  月度预算      │  │
│  │          │  │          │  │          │  │               │  │
│  │  ● 运行中 │  │  1.2M    │  │  $2.35   │  │  █████░░  65% │  │
│  │  :11888  │  │  ↑ 234K  │  │  ↑ $0.42 │  │  $6.50/$10   │  │
│  └──────────┘  └──────────┘  └──────────┘  └───────────────┘  │
│                                                                │
│  ┌─ 今日用量趋势 (折线图) ──────────────────────────────────┐  │
│  │  Token                                                  │  │
│  │  ^                                                      │  │
│  │  |     ╱╲                                               │  │
│  │  |    ╱  ╲      ╱╲                                      │  │
│  │  |   ╱    ╲    ╱  ╲    ╱╲                               │  │
│  │  |  ╱      ╲──╱    ╲──╱  ╲────                          │  │
│  │  └──────────────────────────────────→ 小时              │  │
│  └─────────────────────────────────────────────────────────┘  │
│                                                                │
│  ┌─ 热门模型 Top 5 ───────┐  ┌─ 最近请求 ──────────────────┐  │
│  │  1. gpt-4    8.5K tok  │  │  16:32  gpt-4      ✅ 1.2s  │  │
│  │  2. claude-3  4.2K tok │  │  16:28  deepseek   ✅ 0.8s  │  │
│  │  3. deepseek  3.1K tok │  │  16:15  gpt-4      ❌ err  │  │
│  │  4. qwen-plus 1.8K tok │  │  16:01  claude-3   ✅ 2.1s  │  │
│  │  5. gemini    0.9K tok │  │  ...                       │  │
│  └────────────────────────┘  └─────────────────────────────┘  │
└──────────────────────────────────────────────────────────────┘
```

### 5.2.2 Usage 看板设计

```
┌─ Usage ──────────────────────────────────────────────────────┐
│  时间范围: [今天 ▼]  [7天] [30天] [本月] [自定义...]          │
│                                                               │
│  ┌─ Token 用量趋势 (面积图) ──────────────────────────────┐  │
│  │  Tokens                                                 │  │
│  │  ^   ░░░░░░░░░░░░░░░                                   │  │
│  │  |  ░░░░░░░░░░░░░░░░░                                  │  │
│  │  | ░░░░░░░░░░░░░░░░░░░░                                │  │
│  │  |░░░░░░░░░░░░░░░░░░░░░░░░                             │  │
│  │  └──────────────────────────→ 日期                     │  │
│  │  ── 输入Token  ── 输出Token                             │  │
│  └─────────────────────────────────────────────────────────┘ │
│                                                               │
│  ┌─ 费用趋势 ────────────┐ ┌─ 按模型拆分 (饼图) ──────────┐ │
│  │  $                    │ │        ┌─────┐               │ │
│  │  ^   ╱╲               │ │   gpt-4│█████│ 62%  $15.50   │ │
│  │  |  ╱  ╲    ╱╲        │ │  claude│███  │ 24%  $ 6.00   │ │
│  │  | ╱    ╲──╱  ╲──     │ │ deepsk│█    │ 10%  $ 2.50   │ │
│  │  |╱               ╲──  │ │  other│     │  4%  $ 1.00   │ │
│  │  └──────────────────→  │ └─────────────────────────────┘ │
│  └────────────────────────┘                                 │
│                                                               │
│  ┌─ 按 Provider 拆分 (柱状图) ────────────────────────────┐  │
│  │  Tokens                                                 │  │
│  │  ^                                                      │  │
│  │  |  ████                                                │  │
│  │  |  ████  ███                                           │  │
│  │  |  ████  ███  ██      █                                │  │
│  │  |  ████  ███  ██  █   █                                │  │
│  │  └──OpenAI Anthropic DeepSeek Qwen ──→                  │  │
│  │  ██ 输入Token  ░░ 输出Token                              │  │
│  └─────────────────────────────────────────────────────────┘ │
│                                                               │
│  ┌─ 月度预算追踪 ──────────────────────────────────────────┐ │
│  │  本月预算: $10.00                                       │ │
│  │  ████████████████░░░░░░░░░░░░░░░░  $6.50 (65%)          │ │
│  │              ⚠ 80% 预警线                                │ │
│  │  预估月底: $8.67  │  剩余: $3.50  │  日均: $0.41        │ │
│  └─────────────────────────────────────────────────────────┘ │
│                                                               │
│  ┌─ 明细数据表 ──────────────────────────────────────────┐  │
│  │  日期      模型       请求数    输入Token   输出Token  费用 │
│  │  05-16    gpt-4      23       156,000    42,000    $1.56 │
│  │  05-16    claude-3   12        89,000    21,000    $0.89 │
│  │  05-15    gpt-4       8        45,000    12,000    $0.47 │
│  │  ...                                                    │
│  └─────────────────────────────────────────────────────────┘ │
└──────────────────────────────────────────────────────────────┘
```

### 5.2.1 日志追溯详细设计

**列表视图字段**:

| 列       | 说明                       |
| -------- | -------------------------- |
| 时间     | 请求发起时间，精确到毫秒   |
| 模型     | 客户端请求的模型名         |
| Provider | 路由到的上游 Provider 名称 |
| 协议     | OpenAI / Anthropic         |
| 方式     | Stream / Normal            |
| 状态     | ✅成功 / ❌失败，颜色标记    |
| 延迟     | 端到端耗时                 |
| Token    | `输入 → 输出`              |
| 费用     | 预估 USD                   |

**详情面板 (抽屉展开)**:

```
┌─ Log Detail ──────────────────────────────────┐
│ 概览:    时间 · 模型 · Provider · 延迟 · 状态  │
│‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾│
│                                               │
│ [Request]  [Response]  [Messages]             │  ← Tab 切换
│                                               │
│ ┌─ Request Body ──────────────────────────┐   │
│ │ {                                        │   │
│ │   "model": "gpt-4",                      │   │
│ │   "messages": [...],                     │   │
│ │   "temperature": 0.7,                    │   │
│ │   ...                                    │   │
│ │ }                                        │   │
│ └──────────────────────────────────────────┘   │
│                                               │
│ ┌─ Response Body ─────────────────────────┐   │
│ │ {                                        │   │
│ │   "choices": [{                          │   │
│ │     "message": {                         │   │
│ │       "role": "assistant",               │   │
│ │       "content": "..."                   │   │
│ │     }                                    │   │
│ │   }],                                    │   │
│ │   "usage": {...}                         │   │
│ │ }                                        │   │
│ └──────────────────────────────────────────┘   │
│                                               │
│ ┌─ Messages (Conversation View) ──────────┐   │
│ │ ┌──────────────────────────────────┐     │   │
│ │ │ 🧑 User                          │     │   │
│ │ │ 请帮我解释量子计算的基本原理     │     │   │
│ │ └──────────────────────────────────┘     │   │
│ │ ┌──────────────────────────────────┐     │   │
│ │ │ 🤖 Assistant                     │     │   │
│ │ │ 量子计算是一种利用量子力学...    │     │   │
│ │ └──────────────────────────────────┘     │   │
│ └──────────────────────────────────────────┘   │
└───────────────────────────────────────────────┘
```

**检索能力**:
- 时间范围筛选 (今天/最近7天/最近30天/自定义)
- 模型名筛选 (下拉多选)
- Provider 筛选
- 状态筛选 (成功/失败)
- **全文搜索**: 对 `request_contents.extracted_text` 和 `response_contents.extracted_text` 做 SQLite FTS5 全文检索，支持在对话内容中搜索关键词
- 列表支持分页，默认按时间倒序

**存储策略**:
- 日志保留天数：可配置，默认 30 天
- 定时清理：代理启动时 + 每日凌晨执行过期日志清理
- 导出：支持单条或批量导出为 JSON

### 5.3 Fluent Design 要素

- **Mica 背景**: 通过 Tauri 窗口配置 + CSS `backdrop-filter`
- **Fluent UI v9 Tokens**: `borderRadiusLarge`, `colorNeutralBackground1`, `spacingHorizontalL`
- **亚克力卡片**: `box-shadow` + 半透明背景
- **Segoe UI 字体**: 默认 Fluent 字体栈
- **暗色/亮色主题**: 跟随系统, 通过 `FluentProvider` 切换
- **CommandBar**: 页面顶部操作栏
- **微动画**: 卡片悬浮效果、导航高亮过渡

### 5.4 前端依赖

```json
{
  "@fluentui/react-components": "^9.56.0",
  "@fluentui/react-icons": "^2.0.260",
  "@fluentui/react-nav-preview": "^9.0.0",
  "react-router-dom": "^7.0.0",
  "@tauri-apps/api": "^2",
  "@tauri-apps/plugin-opener": "^2",
  "zustand": "^5.0.0"
}
```

---

## 6. API 端点设计 (代理服务器)

| 方法   | 路径                        | 协议      | 说明                    |
| ------ | --------------------------- | --------- | ----------------------- |
| `POST` | `/v1/chat/completions`      | OpenAI    | 聊天补全 (含 streaming) |
| `POST` | `/v1/completions`           | OpenAI    | 文本补全                |
| `POST` | `/v1/embeddings`            | OpenAI    | 向量嵌入                |
| `GET`  | `/v1/models`                | OpenAI    | 列出可用模型            |
| `POST` | `/v1/messages`              | Anthropic | Messages API            |
| `POST` | `/v1/messages/count_tokens` | Anthropic | Token 计数              |
| `GET`  | `/health`                   | -         | 健康检查                |
| `GET`  | `/metrics`                  | -         | 代理指标                |

> 客户端工具只需配置 `OPENAI_BASE_URL=http://localhost:11888/v1` 或 `ANTHROPIC_BASE_URL=http://localhost:11888/v1` 即可使用。

---

## 7. 协议转换逻辑

```
客户端请求 (OpenAI)             客户端请求 (Anthropic)
        │                               │
        ▼                               ▼
  解析 model 字段                 解析 model 字段
        │                               │
        ▼                               ▼
  查 model_mappings              查 model_mappings
  exposed_name → (provider,       exposed_name → (provider,
                  upstream_name)                  upstream_name)
        │                               │
        ▼                               ▼
  OpenAI Provider? ─Yes→ 直接转发    Anthropic Provider? ─Yes→ 直接转发
        │No                             │No
        ▼                               ▼
  Translator:                      Translator:
  OpenAI → Anthropic               Anthropic → OpenAI
        │                               │
        ▼                               ▼
  发送到 Provider                  发送到 Provider
        │                               │
        ▼                               ▼
  响应 Translator (反向)           响应 Translator (反向)
        │                               │
        ▼                               ▼
  返回 OpenAI 格式响应             返回 Anthropic 格式响应
```

**关键转换规则**:

- OpenAI `messages[{role, content}]` ↔ Anthropic `messages[{role, content}]` + `system` 字段
- OpenAI `stream: true` → SSE `data: {...}\n\n` ↔ Anthropic SSE `event: content_block_delta\ndata: {...}\n\n`
- OpenAI `temperature` → Anthropic `temperature` (直接映射)
- Anthropic `max_tokens` ↔ OpenAI `max_completion_tokens`

---

## 8. 代理生命周期管理

### 8.1 运行时架构

代理服务器作为独立的 Tokio `task` 在后台运行，与 Tauri 主进程分离：

```
┌──────────────────────────────────────────┐
│              Tauri App Process            │
│                                          │
│  ┌─────────────┐   ┌──────────────────┐  │
│  │  UI Thread   │   │  Proxy Tokio Task │  │
│  │  (WebView)   │   │  (Axum Server)    │  │
│  │              │   │                   │  │
│  │  start/stop──┼──►│  oneshot channel  │  │
│  │  toggle      │   │  graceful shutdown│  │
│  └─────────────┘   └──────────────────┘  │
│                                          │
│  Shared State: Arc<ProxyState>           │
│  • running: AtomicBool                   │
│  • shutdown_tx: Option<Sender<()>>       │
│  • port: AtomicU16                       │
└──────────────────────────────────────────┘
```

### 8.2 随时启停

- **启动**: UI 点击开关 → `start_proxy` command → 后台 spawn Axum Server → 绑定 `127.0.0.1:{port}` → 更新状态为 running
- **停止**: UI 点击开关 → `stop_proxy` command → 发送 shutdown signal → Axum graceful shutdown → 更新状态为 stopped
- 启停操作 **不依赖 UI 是否打开**，代理作为后台服务独立运行
- UI 仅在打开时通过 `get_proxy_status` 轮询或事件订阅获取实时状态

### 8.3 开机自启

- 使用 `tauri-plugin-autostart` 将应用注册为系统登录项
- 应用启动时读取 `proxy_config.auto_start` 字段：
  - `auto_start = 1` → 自动调用 `start_proxy`，应用以托盘模式启动（不显示窗口）
  - `auto_start = 0` → 正常启动，代理处于停止状态
- 用户可在 Settings 页面开关此功能

### 8.4 端口动态配置

- 端口通过 `proxy_config.port` 字段持久化，默认 11888
- 修改端口流程：
  1. 用户在 Settings 页修改端口号
  2. 如果代理正在运行 → 自动先 `stop_proxy` → 更新配置 → 自动 `start_proxy`（新端口）
  3. 如果代理已停止 → 仅更新配置，下次启动时生效
- 端口合法性校验：1024~65535，检测是否被占用

---

## 9. 安全设计

- **API Key 存储**: 使用 `ring::aead` (AES-256-GCM) 加密, 密钥派生自机器唯一标识
- **代理绑定**: 仅监听 `127.0.0.1`, 拒绝外部连接
- **日志脱敏**: API Key 不在日志中出现
- **输入校验**: 所有 Tauri Command 参数经过 serde 反序列化校验

---

## 10. Tauri Commands (前后端通信接口)

| Command               | 参数                | 返回                | 说明                               |
| --------------------- | ------------------- | ------------------- | ---------------------------------- |
| `list_providers`      | -                   | `Vec<Provider>`     | 列出所有 Provider                  |
| `add_provider`        | `ProviderInput`     | `Provider`          | 添加 Provider                      |
| `update_provider`     | `id, ProviderInput` | `Provider`          | 更新 Provider                      |
| `delete_provider`     | `id`                | `()`                | 删除 Provider                      |
| `test_provider`       | `id`                | `TestResult`        | 测试连接                           |
| `list_models`         | `provider_id?`      | `Vec<ModelMapping>` | 列出模型映射                       |
| `add_model`           | `ModelInput`        | `ModelMapping`      | 添加模型映射                       |
| `update_model`        | `id, ModelInput`    | `ModelMapping`      | 更新模型映射                       |
| `delete_model`        | `id`                | `()`                | 删除模型映射                       |
| `get_proxy_config`    | -                   | `ProxyConfig`       | 获取代理配置                       |
| `update_proxy_config` | `ProxyConfig`       | `ProxyConfig`       | 更新代理配置                       |
| `start_proxy`         | -                   | `()`                | 启动代理服务器                     |
| `stop_proxy`          | -                   | `()`                | 停止代理服务器                     |
| `restart_proxy`       | -                   | `()`                | 重启代理（端口变更时）             |
| `get_proxy_status`    | -                   | `ProxyStatus`       | 获取代理运行状态                   |
| `get_auto_start`      | -                   | `bool`              | 查询开机自启状态                   |
| `set_auto_start`      | `bool`              | `()`                | 设置开机自启                       |
| `query_logs`          | `LogFilter`         | `LogPage`           | 分页查询请求日志                   |
| `get_log_detail`      | `log_id`            | `LogDetail`         | 获取日志完整详情 (请求/响应/消息)  |
| `search_logs`         | `query: str`        | `Vec<LogSummary>`   | FTS5 全文搜索对话内容              |
| `export_logs`         | `LogFilter`         | `String` (JSON)     | 导出日志为 JSON                    |
| `get_log_stats`       | `range?`            | `LogStats`          | 日志统计 (按天/模型/Provider 聚合) |
| `get_daily_usage`     | `date_range`        | `Vec<DailyUsage>`   | 获取每日 Token/费用聚合            |
| `get_usage_summary`   | `date_range`        | `UsageSummary`      | Token 总览 (总量+趋势+按模型拆分)  |
| `get_model_breakdown` | `date_range`        | `Vec<ModelUsage>`   | 按模型拆分用量                     |
| `get_budget_status`   | -                   | `BudgetStatus`      | 获取月度预算状态 (已用/剩余/预估)  |

---

## 11. 费用计算

### 11.1 计费模型

每个模型映射 (`model_mappings`) 维护专属价格：

```
费用 = (prompt_tokens / 1_000_000) × input_price + (completion_tokens / 1_000_000) × output_price
```

- 价格以 **USD / 1M tokens** 为单位
- 在 Models 页面编辑模型时可直接填写价格
- Token 数量从上游 Provider 响应中的 `usage` 字段提取

### 11.2 聚合流程

```
请求完成
    │
    ▼
1. 解析 usage.prompt_tokens / completion_tokens
    │
    ▼
2. 查 model_mappings.input_price / output_price
    │
    ▼
3. 计算 cost → 写入 request_logs.cost
    │
    ▼
4. Upsert daily_usage (date + model + provider_id)
    │  累计 request_count / prompt_tokens / completion_tokens / cost
    │
    ▼
5. 触发 budget 检查 → 超过预警阈值则推送前端通知
```

### 11.3 预算管理

- 月度预算：在 Settings 设置上限，0 表示不限制
- 预警线：默认 80%，可自定义
- 触发预警时：Dashboard 推送通知 + Usage 页面高亮告警
- 月初自动重置 (每月 1 日 00:00)

---

## 12. 开发阶段规划

| 阶段        | 内容         | 产出                                             |
| ----------- | ------------ | ------------------------------------------------ |
| **Phase 1** | 基础框架搭建 | 数据库初始化、Provider CRUD (Rust + Commands)    |
| **Phase 2** | 代理核心     | Axum Server、请求路由、OpenAI 协议转发           |
| **Phase 3** | 协议扩展     | Anthropic 协议支持、协议互转                     |
| **Phase 4** | 前端 UI      | Fluent UI 页面实现、前后端联调                   |
| **Phase 5** | 数据看板     | Usage 聚合、费用计算、预算管理、Dashboard 可视化 |
| **Phase 6** | 完善         | Streaming、日志追溯、安全加密、打包发布          |
