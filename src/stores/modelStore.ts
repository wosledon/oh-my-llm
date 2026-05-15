import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';
import type { ModelMapping, ModelInput } from '../types';

interface ModelState {
  models: ModelMapping[];
  loading: boolean;
  error: string | null;
  fetchModels: (providerId?: string) => Promise<void>;
  addModel: (input: ModelInput) => Promise<void>;
  updateModel: (id: string, input: ModelInput) => Promise<void>;
  deleteModel: (id: string) => Promise<void>;
}

export const useModelStore = create<ModelState>((set, get) => ({
  models: [],
  loading: false,
  error: null,

  fetchModels: async (providerId) => {
    set({ loading: true, error: null });
    try {
      const models = await invoke<ModelMapping[]>('list_models', { providerId });
      set({ models, loading: false });
    } catch (e) {
      set({ error: String(e), loading: false });
    }
  },

  addModel: async (input) => {
    set({ loading: true, error: null });
    try {
      const model = await invoke<ModelMapping>('add_model', { input });
      set({ models: [...get().models, model], loading: false });
    } catch (e) {
      set({ error: String(e), loading: false });
    }
  },

  updateModel: async (id, input) => {
    set({ loading: true, error: null });
    try {
      const model = await invoke<ModelMapping>('update_model', { id, input });
      set({
        models: get().models.map((m) => (m.id === id ? model : m)),
        loading: false,
      });
    } catch (e) {
      set({ error: String(e), loading: false });
    }
  },

  deleteModel: async (id) => {
    set({ loading: true, error: null });
    try {
      await invoke('delete_model', { id });
      set({
        models: get().models.filter((m) => m.id !== id),
        loading: false,
      });
    } catch (e) {
      set({ error: String(e), loading: false });
    }
  },
}));
