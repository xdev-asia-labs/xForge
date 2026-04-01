import { create } from 'zustand';
import {
    addSource,
    deleteSource,
    getSources,
    installRecipe,
    syncSource,
    type RecipeSource,
    type SourceRecipeItem,
} from '../lib/api';

interface MarketplaceState {
  sources: RecipeSource[];
  loading: boolean;
  error: string;

  fetchSources: () => Promise<void>;
  addSource: (data: { url: string; description?: string }) => Promise<RecipeSource>;
  syncSource: (id: string) => Promise<void>;
  removeSource: (id: string) => Promise<void>;
  install: (sourceId: string, slug: string) => Promise<SourceRecipeItem>;
}

export const useMarketplaceStore = create<MarketplaceState>((set) => ({
  sources: [],
  loading: false,
  error: '',

  fetchSources: async () => {
    set({ loading: true, error: '' });
    try {
      const data = await getSources();
      set({ sources: data });
    } catch (err) {
      set({ error: err instanceof Error ? err.message : 'Failed to fetch sources' });
    } finally {
      set({ loading: false });
    }
  },

  addSource: async (data) => {
    const source = await addSource(data);
    set((state) => ({ sources: [...state.sources, source] }));
    return source;
  },

  syncSource: async (id) => {
    const updated = await syncSource(id);
    set((state) => ({
      sources: state.sources.map((s) => (s.id === id ? updated : s)),
    }));
  },

  removeSource: async (id) => {
    await deleteSource(id);
    set((state) => ({ sources: state.sources.filter((s) => s.id !== id) }));
  },

  install: async (sourceId, slug) => {
    const item = await installRecipe(sourceId, slug);
    set((state) => ({
      sources: state.sources.map((s) =>
        s.id === sourceId
          ? {
              ...s,
              recipes: s.recipes.map((r) =>
                r.slug === slug ? { ...r, installed: true } : r
              ),
            }
          : s
      ),
    }));
    return item;
  },
}));
