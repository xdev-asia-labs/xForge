import { create } from 'zustand';
import {
    createNotificationChannel,
    deleteNotificationChannel,
    getNotificationChannels,
    type NotificationChannel,
} from '../lib/api';

interface NotificationsState {
  channels: NotificationChannel[];
  loading: boolean;
  error: string;

  fetchChannels: () => Promise<void>;
  addChannel: (data: {
    name: string;
    channel_type: string;
    config: Record<string, unknown>;
    events: string[];
  }) => Promise<NotificationChannel>;
  removeChannel: (id: string) => Promise<void>;
}

export const useNotificationsStore = create<NotificationsState>((set) => ({
  channels: [],
  loading: false,
  error: '',

  fetchChannels: async () => {
    set({ loading: true, error: '' });
    try {
      const data = await getNotificationChannels();
      set({ channels: data });
    } catch (err) {
      set({ error: err instanceof Error ? err.message : 'Failed to fetch channels' });
    } finally {
      set({ loading: false });
    }
  },

  addChannel: async (data) => {
    const channel = await createNotificationChannel(data);
    set((state) => ({ channels: [...state.channels, channel] }));
    return channel;
  },

  removeChannel: async (id) => {
    await deleteNotificationChannel(id);
    set((state) => ({ channels: state.channels.filter((c) => c.id !== id) }));
  },
}));
