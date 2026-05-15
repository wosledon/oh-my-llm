export interface Provider {
  id: string;
  name: string;
  prov_type: string;
  base_url: string;
  api_key: string;
  extra_headers?: string;
  created_at: number;
  updated_at: number;
}

export interface ProviderInput {
  name: string;
  prov_type: string;
  base_url: string;
  api_key: string;
  extra_headers?: string;
}

export interface ModelMapping {
  id: string;
  provider_id: string;
  exposed_name: string;
  upstream_name: string;
  enabled: boolean;
  input_price: number;
  output_price: number;
}

export interface ModelInput {
  provider_id: string;
  exposed_name: string;
  upstream_name: string;
  enabled: boolean;
  input_price: number;
  output_price: number;
}

export interface ProxyConfig {
  id: number;
  port: number;
  openai_enabled: boolean;
  anthropic_enabled: boolean;
  default_model?: string;
  auto_start: boolean;
  log_requests: boolean;
  log_retention_days: number;
  budget_enabled: boolean;
  budget_monthly: number;
  budget_warning: number;
  max_retries: number;
  timeout_secs: number;
}
