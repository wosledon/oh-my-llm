export interface Translation {
  app: {
    name: string;
    platform: string;
  };
  nav: {
    dashboard: string;
    usage: string;
    providers: string;
    models: string;
    settings: string;
    logs: string;
  };
  dashboard: {
    title: string;
    proxyStatus: string;
    running: string;
    stopped: string;
    port: string;
    start: string;
    stop: string;
    providers: string;
    total: string;
    models: string;
    recentRequests: string;
    noData: string;
    proxyAddress: string;
    copyAddress: string;
    copied: string;
    shadowModel: string;
    shadowModelSwitch: string;
    protocols: string;
    active: string;
    inactive: string;
    enabledCount: string;
    shadowModelSelectLabel: string;
  };
  providers: {
    title: string;
    addProvider: string;
    editProvider: string;
    name: string;
    type: string;
    baseUrl: string;
    apiKey: string;
    extraHeaders: string;
    actions: string;
    save: string;
    cancel: string;
    delete: string;
    edit: string;
    placeholderName: string;
    placeholderUrl: string;
    placeholderHeaders: string;
  };
  models: {
    title: string;
    addModel: string;
    editModel: string;
    provider: string;
    exposedName: string;
    upstreamName: string;
    inputPrice: string;
    outputPrice: string;
    enabled: string;
    actions: string;
    save: string;
    cancel: string;
    delete: string;
    edit: string;
    placeholderExposed: string;
    placeholderUpstream: string;
  };
  settings: {
    title: string;
    port: string;
    openaiEnabled: string;
    anthropicEnabled: string;
    autoStart: string;
    logRequests: string;
    logRetentionDays: string;
    maxRetries: string;
    timeoutSecs: string;
    shadowModelName: string;
    shadowModelDesc: string;
    language: string;
    zh: string;
    en: string;
  };
  logs: {
    title: string;
    time: string;
    model: string;
    provider: string;
    protocol: string;
    method: string;
    status: string;
    latency: string;
    tokens: string;
    cost: string;
    search: string;
    filter: string;
    all: string;
    success: string;
    error: string;
    streaming: string;
    normal: string;
    detail: string;
    request: string;
    response: string;
    messages: string;
  };
  usage: {
    title: string;
    dateRange: string;
    totalRequests: string;
    totalPromptTokens: string;
    totalCompletionTokens: string;
    totalCost: string;
    model: string;
    requestCount: string;
    promptTokens: string;
    completionTokens: string;
    cost: string;
    dailyBreakdown: string;
  };
  common: {
    loading: string;
    error: string;
    confirm: string;
    close: string;
  };
}

export const zh: Translation = {
  app: {
    name: 'oh-my-llm',
    platform: '平台',
  },
  nav: {
    dashboard: '概览',
    usage: '用量',
    providers: '供应商',
    models: '模型',
    settings: '设置',
    logs: '日志',
  },
  dashboard: {
    title: '概览',
    proxyStatus: '代理状态',
    running: '运行中',
    stopped: '已停止',
    port: '端口',
    start: '启动',
    stop: '停止',
    providers: '供应商',
    total: '总计',
    models: '模型',
    recentRequests: '最近请求',
    noData: '暂无数据',
    proxyAddress: '代理地址',
    copyAddress: '复制地址',
    copied: '已复制',
    shadowModel: '影子模型',
    shadowModelSwitch: '影子模型开关',
    protocols: '协议',
    active: '已启用',
    inactive: '未启用',
    enabledCount: '个已启用',
    shadowModelSelectLabel: '映射模型',
  },
  providers: {
    title: '供应商',
    addProvider: '添加供应商',
    editProvider: '编辑供应商',
    name: '名称',
    type: '类型',
    baseUrl: 'Base URL',
    apiKey: 'API Key',
    extraHeaders: '额外请求头 (JSON)',
    actions: '操作',
    save: '保存',
    cancel: '取消',
    delete: '删除',
    edit: '编辑',
    placeholderName: 'DeepSeek',
    placeholderUrl: 'https://api.example.com/v1',
    placeholderHeaders: '{"X-Custom": "value"}',
  },
  models: {
    title: '模型映射',
    addModel: '添加模型',
    editModel: '编辑模型',
    provider: '供应商',
    exposedName: '暴露名称',
    upstreamName: '上游名称',
    inputPrice: '输入价格 (USD / 1M tokens)',
    outputPrice: '输出价格 (USD / 1M tokens)',
    enabled: '启用',
    actions: '操作',
    save: '保存',
    cancel: '取消',
    delete: '删除',
    edit: '编辑',
    placeholderExposed: 'gpt-4',
    placeholderUpstream: 'gpt-4-0613',
  },
  settings: {
    title: '代理设置',
    port: '端口',
    openaiEnabled: '启用 OpenAI 协议',
    anthropicEnabled: '启用 Anthropic 协议',
    autoStart: '开机自启',
    logRequests: '记录请求日志',
    logRetentionDays: '日志保留天数',
    maxRetries: '最大重试次数',
    timeoutSecs: '超时时间 (秒)',
    shadowModelName: '影子模型名称',
    shadowModelDesc: '用作 /v1/models 返回的统一模型名（映射到 shadow_mapping_id 对应的模型）',
    language: '界面语言',
    zh: '中文',
    en: 'English',
  },
  logs: {
    title: '请求日志',
    time: '时间',
    model: '模型',
    provider: '供应商',
    protocol: '协议',
    method: '方式',
    status: '状态',
    latency: '延迟',
    tokens: 'Token',
    cost: '费用',
    search: '搜索',
    filter: '筛选',
    all: '全部',
    success: '成功',
    error: '失败',
    streaming: '流式',
    normal: '普通',
    detail: '详情',
    request: '请求',
    response: '响应',
    messages: '消息',
  },
  usage: {
    title: '用量统计',
    dateRange: '日期范围',
    totalRequests: '总请求数',
    totalPromptTokens: '输入 Token',
    totalCompletionTokens: '输出 Token',
    totalCost: '总费用',
    model: '模型',
    requestCount: '请求数',
    promptTokens: '输入 Token',
    completionTokens: '输出 Token',
    cost: '费用',
    dailyBreakdown: '每日明细',
  },
  common: {
    loading: '加载中...',
    error: '错误',
    confirm: '确认',
    close: '关闭',
  },
} as const;
