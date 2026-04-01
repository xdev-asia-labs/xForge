import { create } from 'zustand';
import {
    cancelJob,
    createJob,
    getJob,
    getJobs,
    rerunJob,
    type Job,
} from '../lib/api';

interface JobsState {
  jobs: Job[];
  loading: boolean;
  error: string;

  fetchJobs: () => Promise<void>;
  getJob: (id: string) => Promise<Job>;
  runJob: (data: { recipe_name: string; server_ids: string[]; params?: Record<string, unknown> }) => Promise<Job>;
  cancel: (id: string) => Promise<void>;
  rerun: (id: string) => Promise<Job>;
  updateJob: (job: Job) => void;
}

export const useJobsStore = create<JobsState>((set) => ({
  jobs: [],
  loading: false,
  error: '',

  fetchJobs: async () => {
    set({ loading: true, error: '' });
    try {
      const data = await getJobs();
      set({ jobs: data });
    } catch (err) {
      set({ error: err instanceof Error ? err.message : 'Failed to fetch jobs' });
    } finally {
      set({ loading: false });
    }
  },

  getJob: async (id) => {
    return getJob(id);
  },

  runJob: async (data) => {
    const job = await createJob(data);
    set((state) => ({ jobs: [job, ...state.jobs] }));
    return job;
  },

  cancel: async (id) => {
    const updated = await cancelJob(id);
    set((state) => ({
      jobs: state.jobs.map((j) => (j.id === id ? updated : j)),
    }));
  },

  rerun: async (id) => {
    const job = await rerunJob(id);
    set((state) => ({ jobs: [job, ...state.jobs] }));
    return job;
  },

  updateJob: (job) => {
    set((state) => ({
      jobs: state.jobs.map((j) => (j.id === job.id ? job : j)),
    }));
  },
}));
