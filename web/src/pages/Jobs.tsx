import { useEffect, useState } from 'react';
import JobLog from '../components/JobLog';
import { cancelJob, getJobs, rerunJob, type Job } from '../lib/api';

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

const ALL_STATUSES = ['success', 'failed', 'running', 'pending', 'cancelled'];

export default function Jobs() {
  const [jobs, setJobs] = useState<Job[]>([]);
  const [error, setError] = useState('');
  const [expandedJob, setExpandedJob] = useState<string | null>(null);
  const [filterStatus, setFilterStatus] = useState('');
  const [filterRecipe, setFilterRecipe] = useState('');
  const [search, setSearch] = useState('');

  const loadJobs = () => {
    getJobs()
      .then(setJobs)
      .catch((err) => setError(err.message));
  };

  useEffect(() => {
    loadJobs();
    const interval = setInterval(loadJobs, 5000);
    return () => clearInterval(interval);
  }, []);

  const handleCancel = async (id: string) => {
    try {
      await cancelJob(id);
      loadJobs();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to cancel job');
    }
  };

  const handleRerun = async (id: string) => {
    try {
      await rerunJob(id);
      loadJobs();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to re-run job');
    }
  };

  // Get unique recipe names for filter
  const recipeNames = [...new Set(jobs.map((j) => j.recipe_name))].sort();

  // Filter jobs
  const filtered = jobs.filter((j) => {
    if (filterStatus && j.status !== filterStatus) return false;
    if (filterRecipe && j.recipe_name !== filterRecipe) return false;
    if (search) {
      const q = search.toLowerCase();
      if (
        !j.recipe_name.toLowerCase().includes(q) &&
        !j.id.toLowerCase().includes(q)
      )
        return false;
    }
    return true;
  });

  const formatDuration = (job: Job): string | null => {
    if (!job.started_at) return null;
    const start = new Date(job.started_at).getTime();
    const end = job.finished_at ? new Date(job.finished_at).getTime() : Date.now();
    const secs = Math.round((end - start) / 1000);
    if (secs < 60) return `${secs}s`;
    const mins = Math.floor(secs / 60);
    return `${mins}m ${secs % 60}s`;
  };

  return (
    <div>
      <div className="flex items-center justify-between mb-6">
        <h2 className="text-2xl font-bold text-white">Jobs</h2>
        <button
          onClick={loadJobs}
          className="px-4 py-2 bg-gray-800 text-gray-300 text-sm font-medium rounded-lg hover:bg-gray-700 transition-colors"
        >
          ⟳ Refresh
        </button>
      </div>

      {error && (
        <div className="bg-red-600/20 border border-red-600/30 text-red-400 text-sm p-3 rounded-lg mb-4">
          {error}
        </div>
      )}

      {/* Filter Bar */}
      <div className="flex flex-wrap items-center gap-3 mb-4">
        <input
          type="text"
          value={search}
          onChange={(e) => setSearch(e.target.value)}
          placeholder="Search jobs..."
          className="flex-1 min-w-[200px] px-3 py-2 bg-gray-800 border border-gray-700 rounded-lg text-white text-sm focus:outline-none focus:ring-2 focus:ring-forge-500"
        />
        <select
          value={filterStatus}
          onChange={(e) => setFilterStatus(e.target.value)}
          className="px-3 py-2 bg-gray-800 border border-gray-700 rounded-lg text-white text-sm focus:outline-none focus:ring-2 focus:ring-forge-500"
        >
          <option value="">All Status</option>
          {ALL_STATUSES.map((s) => (
            <option key={s} value={s}>{s}</option>
          ))}
        </select>
        <select
          value={filterRecipe}
          onChange={(e) => setFilterRecipe(e.target.value)}
          className="px-3 py-2 bg-gray-800 border border-gray-700 rounded-lg text-white text-sm focus:outline-none focus:ring-2 focus:ring-forge-500"
        >
          <option value="">All Recipes</option>
          {recipeNames.map((r) => (
            <option key={r} value={r}>{r}</option>
          ))}
        </select>
      </div>

      <div className="space-y-3">
        {filtered.map((job) => (
          <div
            key={job.id}
            className="bg-gray-900 border border-gray-800 rounded-xl overflow-hidden"
          >
            <div
              className="px-6 py-4 flex items-center justify-between cursor-pointer hover:bg-gray-800/50 transition-colors"
              onClick={() =>
                setExpandedJob(expandedJob === job.id ? null : job.id)
              }
            >
              <div className="flex items-center gap-4">
                <span
                  className={`px-2.5 py-1 text-xs font-medium rounded-md ${statusBg[job.status] || ''} ${statusColors[job.status] || ''}`}
                >
                  {job.status}
                </span>
                <div>
                  <div className="text-sm font-medium text-white">
                    {job.recipe_name}
                  </div>
                  <div className="text-xs text-gray-500 font-mono">
                    {job.id.slice(0, 8)}... • {job.server_ids.length} server{job.server_ids.length !== 1 ? 's' : ''}
                  </div>
                </div>
              </div>

              <div className="flex items-center gap-4">
                <div className="text-right">
                  <div className="text-xs text-gray-500">
                    {job.created_at
                      ? new Date(job.created_at).toLocaleString()
                      : '-'}
                  </div>
                  {formatDuration(job) && (
                    <div className="text-xs text-gray-600">
                      Duration: {formatDuration(job)}
                    </div>
                  )}
                </div>

                <div className="flex gap-2">
                  {(job.status === 'success' || job.status === 'failed' || job.status === 'cancelled') && (
                    <button
                      onClick={(e) => {
                        e.stopPropagation();
                        handleRerun(job.id);
                      }}
                      className="px-3 py-1.5 bg-forge-600/20 text-forge-400 text-xs font-medium rounded-lg hover:bg-forge-600/30 transition-colors"
                    >
                      ⟳ Re-run
                    </button>
                  )}

                  {(job.status === 'pending' || job.status === 'running') && (
                    <button
                      onClick={(e) => {
                        e.stopPropagation();
                        handleCancel(job.id);
                      }}
                      className="px-3 py-1.5 bg-red-600/20 text-red-400 text-xs font-medium rounded-lg hover:bg-red-600/30 transition-colors"
                    >
                      Cancel
                    </button>
                  )}
                </div>

                <span className="text-gray-600 text-sm">
                  {expandedJob === job.id ? '▾' : '▸'}
                </span>
              </div>
            </div>

            {expandedJob === job.id && (
              <div className="px-6 pb-4">
                {job.params && (
                  <div className="mb-3">
                    <span className="text-xs text-gray-500">Params: </span>
                    <span className="text-xs text-gray-400 font-mono">
                      {JSON.stringify(job.params)}
                    </span>
                  </div>
                )}
                <JobLog
                  jobId={job.id}
                  initialOutput={job.output}
                  isRunning={job.status === 'running'}
                />
              </div>
            )}
          </div>
        ))}
      </div>

      {filtered.length === 0 && (
        <div className="text-center py-12 text-gray-500">
          <p className="text-lg mb-2">
            {jobs.length === 0 ? 'No jobs yet' : 'No jobs match your filter'}
          </p>
          <p className="text-sm">
            {jobs.length === 0
              ? 'Deploy a recipe to create your first job'
              : 'Try adjusting your filters'}
          </p>
        </div>
      )}
    </div>
  );
}
