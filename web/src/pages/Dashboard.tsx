import { useEffect, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { getDashboard, type DashboardStats, type Job } from '../lib/api';

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
  const [stats, setStats] = useState<DashboardStats | null>(null);
  const [error, setError] = useState('');
  const navigate = useNavigate();

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

  const successRate = stats.total_jobs > 0
    ? Math.round((stats.successful_jobs / stats.total_jobs) * 100)
    : 0;

  return (
    <div>
      <h2 className="text-2xl font-bold text-white mb-6">Dashboard</h2>

      {/* Stats Grid */}
      <div className="grid grid-cols-2 md:grid-cols-4 gap-4 mb-8">
        <div className="bg-gray-900 border border-gray-800 rounded-xl p-5">
          <div className="text-xs text-gray-500 mb-1">Servers</div>
          <div className="text-2xl font-bold text-white">{stats.server_count}</div>
          <div className="flex gap-3 mt-2 text-xs">
            <span className="text-green-400">{stats.servers_online} online</span>
            <span className="text-red-400">{stats.servers_offline} offline</span>
          </div>
        </div>
        <div className="bg-gray-900 border border-gray-800 rounded-xl p-5">
          <div className="text-xs text-gray-500 mb-1">Active Jobs</div>
          <div className="text-2xl font-bold text-yellow-400">{stats.active_jobs}</div>
          <div className="text-xs text-gray-500 mt-2">{stats.total_jobs} total</div>
        </div>
        <div className="bg-gray-900 border border-gray-800 rounded-xl p-5">
          <div className="text-xs text-gray-500 mb-1">Success Rate</div>
          <div className={`text-2xl font-bold ${successRate >= 80 ? 'text-green-400' : successRate >= 50 ? 'text-yellow-400' : 'text-red-400'}`}>
            {successRate}%
          </div>
          <div className="flex gap-3 mt-2 text-xs">
            <span className="text-green-400">{stats.successful_jobs} passed</span>
            <span className="text-red-400">{stats.failed_jobs} failed</span>
          </div>
        </div>
        <div className="bg-gray-900 border border-gray-800 rounded-xl p-5">
          <div className="text-xs text-gray-500 mb-1">Schedules</div>
          <div className="text-2xl font-bold text-forge-400">{stats.active_schedules}</div>
          <div className="text-xs text-gray-500 mt-2">active</div>
        </div>
      </div>

      {/* Recent Jobs */}
      <div className="bg-gray-900 border border-gray-800 rounded-xl">
        <div className="px-6 py-4 border-b border-gray-800 flex items-center justify-between">
          <h3 className="text-lg font-semibold text-white">Recent Jobs</h3>
          <button
            onClick={() => navigate('/jobs')}
            className="text-xs text-forge-400 hover:text-forge-300 transition-colors"
          >
            View all →
          </button>
        </div>
        <div className="divide-y divide-gray-800">
          {stats.recent_jobs.length === 0 && (
            <div className="px-6 py-8 text-center text-gray-500">No jobs yet</div>
          )}
          {stats.recent_jobs.map((job: Job) => (
            <div key={job.id} className="px-6 py-4 flex items-center justify-between hover:bg-gray-800/50 transition-colors">
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
