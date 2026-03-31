import { useEffect, useState } from 'react';
import { getDashboard, type Job } from '../lib/api';

const statusColors: Record<string, string> = {
  success: 'text-green-400',
  failed: 'text-red-400',
  running: 'text-yellow-400',
  pending: 'text-blue-400',
  cancelled: 'text-gray-400',
};

const statusBg: Record<string, string> = {
  success: 'bg-green-600/20',
  failed: 'bg-red-600/20',
  running: 'bg-yellow-600/20',
  pending: 'bg-blue-600/20',
  cancelled: 'bg-gray-600/20',
};

export default function Dashboard() {
  const [stats, setStats] = useState<{
    server_count: number;
    active_jobs: number;
    recent_jobs: Job[];
  } | null>(null);
  const [error, setError] = useState('');

  useEffect(() => {
    getDashboard()
      .then(setStats)
      .catch((err) => setError(err.message));
  }, []);

  if (error) {
    return (
      <div className="bg-red-600/20 border border-red-600/30 text-red-400 p-4 rounded-lg">
        {error}
      </div>
    );
  }

  if (!stats) {
    return <div className="text-gray-500">Loading...</div>;
  }

  return (
    <div>
      <h2 className="text-2xl font-bold text-white mb-6">Dashboard</h2>

      {/* Stats Grid */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-6 mb-8">
        <div className="bg-gray-900 border border-gray-800 rounded-xl p-6">
          <div className="text-sm text-gray-400 mb-1">Total Servers</div>
          <div className="text-3xl font-bold text-white">{stats.server_count}</div>
        </div>
        <div className="bg-gray-900 border border-gray-800 rounded-xl p-6">
          <div className="text-sm text-gray-400 mb-1">Active Jobs</div>
          <div className="text-3xl font-bold text-yellow-400">{stats.active_jobs}</div>
        </div>
        <div className="bg-gray-900 border border-gray-800 rounded-xl p-6">
          <div className="text-sm text-gray-400 mb-1">Recent Jobs</div>
          <div className="text-3xl font-bold text-white">{stats.recent_jobs.length}</div>
        </div>
      </div>

      {/* Recent Jobs */}
      <div className="bg-gray-900 border border-gray-800 rounded-xl">
        <div className="px-6 py-4 border-b border-gray-800">
          <h3 className="text-lg font-semibold text-white">Recent Jobs</h3>
        </div>
        <div className="divide-y divide-gray-800">
          {stats.recent_jobs.length === 0 && (
            <div className="px-6 py-8 text-center text-gray-500">No jobs yet</div>
          )}
          {stats.recent_jobs.map((job) => (
            <div key={job.id} className="px-6 py-4 flex items-center justify-between">
              <div>
                <div className="text-sm font-medium text-white">{job.recipe_name}</div>
                <div className="text-xs text-gray-500 font-mono">{job.id.slice(0, 8)}</div>
              </div>
              <div className="flex items-center gap-4">
                <span className="text-xs text-gray-500">
                  {job.created_at ? new Date(job.created_at).toLocaleString() : '-'}
                </span>
                <span
                  className={`px-2.5 py-1 text-xs font-medium rounded-md ${statusBg[job.status] || ''} ${statusColors[job.status] || ''}`}
                >
                  {job.status}
                </span>
              </div>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}
