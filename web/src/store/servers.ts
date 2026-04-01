import { create } from 'zustand';
import {
    bulkHealthCheck,
    createServer,
    deleteServer,
    getServer,
    getServerGroups,
    getServers,
    healthCheckServer,
    updateServer,
    type Server,
    type ServerGroup,
} from '../lib/api';

interface ServersState {
  servers: Server[];
  groups: ServerGroup[];
  loading: boolean;
  error: string;

  fetchServers: () => Promise<void>;
  fetchGroups: () => Promise<void>;
  getServer: (id: string) => Promise<Server>;
  addServer: (data: Partial<Server>) => Promise<Server>;
  editServer: (id: string, data: Partial<Server>) => Promise<Server>;
  removeServer: (id: string) => Promise<void>;
  healthCheck: (id: string) => Promise<void>;
  bulkHealthCheck: (ids: string[]) => Promise<void>;
}

export const useServersStore = create<ServersState>((set) => ({
  servers: [],
  groups: [],
  loading: false,
  error: '',

  fetchServers: async () => {
    set({ loading: true, error: '' });
    try {
      const data = await getServers();
      set({ servers: data });
    } catch (err) {
      set({ error: err instanceof Error ? err.message : 'Failed to fetch servers' });
    } finally {
      set({ loading: false });
    }
  },

  fetchGroups: async () => {
    try {
      const data = await getServerGroups();
      set({ groups: data });
    } catch (err) {
      set({ error: err instanceof Error ? err.message : 'Failed to fetch groups' });
    }
  },

  getServer: async (id) => {
    return getServer(id);
  },

  addServer: async (data) => {
    const server = await createServer(data);
    set((state) => ({ servers: [...state.servers, server] }));
    return server;
  },

  editServer: async (id, data) => {
    const updated = await updateServer(id, data);
    set((state) => ({
      servers: state.servers.map((s) => (s.id === id ? updated : s)),
    }));
    return updated;
  },

  removeServer: async (id) => {
    await deleteServer(id);
    set((state) => ({ servers: state.servers.filter((s) => s.id !== id) }));
  },

  healthCheck: async (id) => {
    const result = await healthCheckServer(id);
    set((state) => ({
      servers: state.servers.map((s) =>
        s.id === id ? { ...s, status: result.status, last_health_check: result.checked_at } : s
      ),
    }));
  },

  bulkHealthCheck: async (ids) => {
    const results = await bulkHealthCheck(ids);
    set((state) => ({
      servers: state.servers.map((s) => {
        const r = results.find((r) => r.id === s.id);
        return r ? { ...s, status: r.status, last_health_check: r.checked_at } : s;
      }),
    }));
  },
}));
