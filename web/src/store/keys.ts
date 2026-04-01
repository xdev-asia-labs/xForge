import { create } from 'zustand';
import {
    createKey,
    deleteKey,
    getKeys,
    type KeyStoreEntry,
} from '../lib/api';

interface KeysState {
  keys: KeyStoreEntry[];
  loading: boolean;
  error: string;

  fetchKeys: () => Promise<void>;
  addKey: (data: {
    name: string;
    key_type: string;
    key_data: string;
    description?: string;
  }) => Promise<KeyStoreEntry>;
  removeKey: (id: string) => Promise<void>;
}

export const useKeysStore = create<KeysState>((set) => ({
  keys: [],
  loading: false,
  error: '',

  fetchKeys: async () => {
    set({ loading: true, error: '' });
    try {
      const data = await getKeys();
      set({ keys: data });
    } catch (err) {
      set({ error: err instanceof Error ? err.message : 'Failed to fetch keys' });
    } finally {
      set({ loading: false });
    }
  },

  addKey: async (data) => {
    const key = await createKey(data);
    set((state) => ({ keys: [...state.keys, key] }));
    return key;
  },

  removeKey: async (id) => {
    await deleteKey(id);
    set((state) => ({ keys: state.keys.filter((k) => k.id !== id) }));
  },
}));
