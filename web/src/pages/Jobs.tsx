import { useEffect, useState } from 'react';
import JobLog from '../components/JobLog';
import { cancelJob, getJobs, type Job } from '../lib/api';

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

export default function Jobs() {
  const [jobs, setJobs] = useState<Job[]>([]);
  const [error, setError] = useState('');
  const [expandedJob, setExpandedJob] = useState<string | null>(null);

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

      <div className="space-y-3">
        {jobs.map((job) => (
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
                  {job.finished_at && (
                    <div className="text-xs text-gray-600">
                      Finished:{' '}
                      {new Date(job.finished_at).toLocaleString()}
                    </div>
                  )}
                </div>

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

      {jobs.length === 0 && (
        <div className="text-center py-12 text-gray-500">
          <p className="text-lg mb-2">No jobs yet</p>
          <p className="text-sm">Deploy a recipe to create your first job</p>
        </div>
      )}
    </div>
  );
}
