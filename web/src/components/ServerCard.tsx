import type { Server } from '../lib/api';

interface ServerCardProps {
  server: Server;
  onHealthCheck: (id: string) => void;
  onDelete: (id: string) => void;
}

const statusColors: Record<string, string> = {
  online: 'bg-green-500',
  offline: 'bg-red-500',
  unknown: 'bg-gray-500',
};

export default function ServerCard({ server, onHealthCheck, onDelete }: ServerCardProps) {
  return (
    <div className="bg-gray-900 border border-gray-800 rounded-xl p-5 hover:border-gray-700 transition-colors">
      <div className="flex items-start justify-between mb-3">
        <div>
          <h3 className="text-lg font-semibold text-white">{server.name}</h3>
          <p className="text-sm text-gray-400 font-mono">
            {server.ssh_user}@{server.host}:{server.port}
          </p>
        </div>
        <div className="flex items-center gap-2">
          <span
            className={`w-2.5 h-2.5 rounded-full ${statusColors[server.status] || statusColors.unknown}`}
          />
          <span className="text-xs text-gray-400 capitalize">{server.status}</span>
        </div>
      </div>

      {server.labels.length > 0 && (
        <div className="flex flex-wrap gap-1.5 mb-3">
          {server.labels.map((label) => (
            <span
              key={label}
              className="px-2 py-0.5 bg-forge-600/20 text-forge-400 text-xs rounded-md"
            >
              {label}
            </span>
          ))}
        </div>
      )}

      {server.group_name && (
        <p className="text-xs text-gray-500 mb-3">
          Group: <span className="text-gray-300">{server.group_name}</span>
        </p>
      )}

      {server.last_health_check && (
        <p className="text-xs text-gray-500 mb-4">
          Last check: {new Date(server.last_health_check).toLocaleString()}
        </p>
      )}

      <div className="flex gap-2">
        <button
          onClick={() => onHealthCheck(server.id)}
          className="flex-1 px-3 py-1.5 bg-forge-600/20 text-forge-400 text-xs font-medium rounded-lg hover:bg-forge-600/30 transition-colors"
        >
          Health Check
        </button>
        <button
          onClick={() => onDelete(server.id)}
          className="px-3 py-1.5 bg-red-600/20 text-red-400 text-xs font-medium rounded-lg hover:bg-red-600/30 transition-colors"
        >
          Delete
        </button>
      </div>
    </div>
  );
}
