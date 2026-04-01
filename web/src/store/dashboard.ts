import { create } from 'zustand';
import { getDashboard, type DashboardStats } from '../lib/api';

interface DashboardState {
  stats: DashboardStats | null;
  loading: boolean;
  error: string;

  fetchStats: () => Promise<void>;
}

export const useDashboardStore = create<DashboardState>((set) => ({
  stats: null,
  loading: false,
  error: '',

  fetchStats: async () => {
    set({ loading: true, error: '' });
    try {
      const data = await getDashboard();
      set({ stats: data });
    } catch (err) {
      set({ error: err instanceof Error ? err.message : 'Failed to fetch dashboard stats' });
    } finally {
      set({ loading: false });
    }
  },
}));
