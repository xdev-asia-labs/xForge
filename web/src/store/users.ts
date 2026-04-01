import { create } from 'zustand';
import {
    createUser,
    deleteUser,
    getCurrentUser,
    getUsers,
    updateUser,
    type User,
} from '../lib/api';

interface UsersState {
  users: User[];
  currentUser: User | null;
  loading: boolean;
  error: string;

  fetchUsers: () => Promise<void>;
  fetchCurrentUser: () => Promise<void>;
  addUser: (data: {
    username: string;
    password: string;
    role: string;
    email?: string;
    display_name?: string;
  }) => Promise<User>;
  editUser: (
    id: string,
    data: { password?: string; role?: string; email?: string; display_name?: string }
  ) => Promise<User>;
  removeUser: (id: string) => Promise<void>;
}

export const useUsersStore = create<UsersState>((set) => ({
  users: [],
  currentUser: null,
  loading: false,
  error: '',

  fetchUsers: async () => {
    set({ loading: true, error: '' });
    try {
      const data = await getUsers();
      set({ users: data });
    } catch (err) {
      set({ error: err instanceof Error ? err.message : 'Failed to fetch users' });
    } finally {
      set({ loading: false });
    }
  },

  fetchCurrentUser: async () => {
    try {
      const user = await getCurrentUser();
      set({ currentUser: user });
    } catch (err) {
      set({ error: err instanceof Error ? err.message : 'Failed to fetch current user' });
    }
  },

  addUser: async (data) => {
    const user = await createUser(data);
    set((state) => ({ users: [...state.users, user] }));
    return user;
  },

  editUser: async (id, data) => {
    const updated = await updateUser(id, data);
    set((state) => ({
      users: state.users.map((u) => (u.id === id ? updated : u)),
    }));
    return updated;
  },

  removeUser: async (id) => {
    await deleteUser(id);
    set((state) => ({ users: state.users.filter((u) => u.id !== id) }));
  },
}));
