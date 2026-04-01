import { create } from 'zustand';
import {
    createSchedule,
    deleteSchedule,
    getSchedules,
    updateSchedule,
    type Schedule,
} from '../lib/api';

interface SchedulesState {
  schedules: Schedule[];
  loading: boolean;
  error: string;

  fetchSchedules: () => Promise<void>;
  addSchedule: (data: {
    name: string;
    recipe_name: string;
    server_ids: string[];
    params?: Record<string, unknown>;
    cron_expression: string;
  }) => Promise<Schedule>;
  editSchedule: (
    id: string,
    data: {
      name?: string;
      cron_expression?: string;
      server_ids?: string[];
      params?: Record<string, unknown>;
      enabled?: boolean;
    }
  ) => Promise<Schedule>;
  removeSchedule: (id: string) => Promise<void>;
}

export const useSchedulesStore = create<SchedulesState>((set) => ({
  schedules: [],
  loading: false,
  error: '',

  fetchSchedules: async () => {
    set({ loading: true, error: '' });
    try {
      const data = await getSchedules();
      set({ schedules: data });
    } catch (err) {
      set({ error: err instanceof Error ? err.message : 'Failed to fetch schedules' });
    } finally {
      set({ loading: false });
    }
  },

  addSchedule: async (data) => {
    const schedule = await createSchedule(data);
    set((state) => ({ schedules: [...state.schedules, schedule] }));
    return schedule;
  },

  editSchedule: async (id, data) => {
    const updated = await updateSchedule(id, data);
    set((state) => ({
      schedules: state.schedules.map((s) => (s.id === id ? updated : s)),
    }));
    return updated;
  },

  removeSchedule: async (id) => {
    await deleteSchedule(id);
    set((state) => ({ schedules: state.schedules.filter((s) => s.id !== id) }));
  },
}));
