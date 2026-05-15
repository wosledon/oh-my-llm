import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';
import type { ProxyConfig } from '../types';

interface ProxyState {
  config: ProxyConfig | null;
  running: boolean;
  loading: boolean;
  error: string | null;
  fetchConfig: () => Promise<void>;
  updateConfig: (config: ProxyConfig) => Promise<void>;
  startProxy: () => Promise<void>;
  stopProxy: () => Promise<void>;
  getStatus: () => Promise<void>;
}

export const useProxyStore = create<ProxyState>((set) => ({
  config: null,
  running: false,
  loading: false,
  error: null,

  fetchConfig: async () => {
    set({ loading: true, error: null });
    try {
      const config = await invoke<ProxyConfig>('get_proxy_config');
      set({ config, loading: false });
    } catch (e) {
      set({ error: String(e), loading: false });
    }
  },

  updateConfig: async (config) => {
    set({ loading: true, error: null });
    try {
      const updated = await invoke<ProxyConfig>('update_proxy_config', { config });
      set({ config: updated, loading: false });
    } catch (e) {
      set({ error: String(e), loading: false });
    }
  },

  startProxy: async () => {
    set({ loading: true, error: null });
    try {
      await invoke('start_proxy');
      set({ running: true, loading: false });
    } catch (e) {
      set({ error: String(e), loading: false });
    }
  },

  stopProxy: async () => {
    set({ loading: true, error: null });
    try {
      await invoke('stop_proxy');
      set({ running: false, loading: false });
    } catch (e) {
      set({ error: String(e), loading: false });
    }
  },

  getStatus: async () => {
    try {
      const running = await invoke<boolean>('get_proxy_status');
      set({ running });
    } catch (e) {
      set({ error: String(e) });
    }
  },
}));
