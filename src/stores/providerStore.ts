import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';
import type { Provider, ProviderInput } from '../types';

interface ProviderState {
  providers: Provider[];
  loading: boolean;
  error: string | null;
  fetchProviders: () => Promise<void>;
  addProvider: (input: ProviderInput) => Promise<void>;
  updateProvider: (id: string, input: ProviderInput) => Promise<void>;
  deleteProvider: (id: string) => Promise<void>;
}

export const useProviderStore = create<ProviderState>((set, get) => ({
  providers: [],
  loading: false,
  error: null,

  fetchProviders: async () => {
    set({ loading: true, error: null });
    try {
      const providers = await invoke<Provider[]>('list_providers');
      set({ providers, loading: false });
    } catch (e) {
      set({ error: String(e), loading: false });
    }
  },

  addProvider: async (input) => {
    set({ loading: true, error: null });
    try {
      const provider = await invoke<Provider>('add_provider', { input });
      set({ providers: [...get().providers, provider], loading: false });
    } catch (e) {
      set({ error: String(e), loading: false });
    }
  },

  updateProvider: async (id, input) => {
    set({ loading: true, error: null });
    try {
      const provider = await invoke<Provider>('update_provider', { id, input });
      set({
        providers: get().providers.map((p) => (p.id === id ? provider : p)),
        loading: false,
      });
    } catch (e) {
      set({ error: String(e), loading: false });
    }
  },

  deleteProvider: async (id) => {
    set({ loading: true, error: null });
    try {
      await invoke('delete_provider', { id });
      set({
        providers: get().providers.filter((p) => p.id !== id),
        loading: false,
      });
    } catch (e) {
      set({ error: String(e), loading: false });
    }
  },
}));
