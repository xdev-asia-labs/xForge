import { useEffect, useState } from 'react';
import ServerCard from '../components/ServerCard';
import {
    createServer,
    deleteServer,
    getServers,
    healthCheckServer,
    type Server,
} from '../lib/api';

export default function Servers() {
  const [servers, setServers] = useState<Server[]>([]);
  const [error, setError] = useState('');
  const [showForm, setShowForm] = useState(false);
  const [form, setForm] = useState({
    name: '',
    host: '',
    port: '22',
    ssh_user: 'root',
    ssh_key_path: '',
    labels: '',
    group_name: '',
  });

  const loadServers = () => {
    getServers()
      .then(setServers)
      .catch((err) => setError(err.message));
  };

  useEffect(loadServers, []);

  const handleCreate = async (e: React.FormEvent) => {
    e.preventDefault();
    setError('');
    try {
      await createServer({
        name: form.name,
        host: form.host,
        port: parseInt(form.port) || 22,
        ssh_user: form.ssh_user || 'root',
        ssh_key_path: form.ssh_key_path || undefined,
        labels: form.labels
          ? form.labels.split(',').map((l) => l.trim())
          : [],
        group_name: form.group_name || undefined,
      });
      setForm({ name: '', host: '', port: '22', ssh_user: 'root', ssh_key_path: '', labels: '', group_name: '' });
      setShowForm(false);
      loadServers();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to create server');
    }
  };

  const handleHealthCheck = async (id: string) => {
    try {
      await healthCheckServer(id);
      loadServers();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Health check failed');
    }
  };

  const handleDelete = async (id: string) => {
    if (!confirm('Are you sure you want to delete this server?')) return;
    try {
      await deleteServer(id);
      loadServers();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to delete server');
    }
  };

  return (
    <div>
      <div className="flex items-center justify-between mb-6">
        <h2 className="text-2xl font-bold text-white">Servers</h2>
        <button
          onClick={() => setShowForm(!showForm)}
          className="px-4 py-2 bg-forge-600 text-white text-sm font-medium rounded-lg hover:bg-forge-700 transition-colors"
        >
          {showForm ? 'Cancel' : '+ Add Server'}
        </button>
      </div>

      {error && (
        <div className="bg-red-600/20 border border-red-600/30 text-red-400 text-sm p-3 rounded-lg mb-4">
          {error}
        </div>
      )}

      {showForm && (
        <form
          onSubmit={handleCreate}
          className="bg-gray-900 border border-gray-800 rounded-xl p-6 mb-6 grid grid-cols-1 md:grid-cols-2 gap-4"
        >
          <div>
            <label className="block text-sm font-medium text-gray-300 mb-1">Name</label>
            <input
              type="text"
              value={form.name}
              onChange={(e) => setForm({ ...form, name: e.target.value })}
              className="w-full px-3 py-2 bg-gray-800 border border-gray-700 rounded-lg text-white text-sm focus:outline-none focus:ring-2 focus:ring-forge-500"
              placeholder="web-server-01"
              required
            />
          </div>
          <div>
            <label className="block text-sm font-medium text-gray-300 mb-1">Host</label>
            <input
              type="text"
              value={form.host}
              onChange={(e) => setForm({ ...form, host: e.target.value })}
              className="w-full px-3 py-2 bg-gray-800 border border-gray-700 rounded-lg text-white text-sm focus:outline-none focus:ring-2 focus:ring-forge-500"
              placeholder="192.168.1.100"
              required
            />
          </div>
          <div>
            <label className="block text-sm font-medium text-gray-300 mb-1">Port</label>
            <input
              type="number"
              value={form.port}
              onChange={(e) => setForm({ ...form, port: e.target.value })}
              className="w-full px-3 py-2 bg-gray-800 border border-gray-700 rounded-lg text-white text-sm focus:outline-none focus:ring-2 focus:ring-forge-500"
            />
          </div>
          <div>
            <label className="block text-sm font-medium text-gray-300 mb-1">SSH User</label>
            <input
              type="text"
              value={form.ssh_user}
              onChange={(e) => setForm({ ...form, ssh_user: e.target.value })}
              className="w-full px-3 py-2 bg-gray-800 border border-gray-700 rounded-lg text-white text-sm focus:outline-none focus:ring-2 focus:ring-forge-500"
            />
          </div>
          <div>
            <label className="block text-sm font-medium text-gray-300 mb-1">SSH Key Path</label>
            <input
              type="text"
              value={form.ssh_key_path}
              onChange={(e) => setForm({ ...form, ssh_key_path: e.target.value })}
              className="w-full px-3 py-2 bg-gray-800 border border-gray-700 rounded-lg text-white text-sm focus:outline-none focus:ring-2 focus:ring-forge-500"
              placeholder="~/.ssh/id_rsa"
            />
          </div>
          <div>
            <label className="block text-sm font-medium text-gray-300 mb-1">Group</label>
            <input
              type="text"
              value={form.group_name}
              onChange={(e) => setForm({ ...form, group_name: e.target.value })}
              className="w-full px-3 py-2 bg-gray-800 border border-gray-700 rounded-lg text-white text-sm focus:outline-none focus:ring-2 focus:ring-forge-500"
              placeholder="production"
            />
          </div>
          <div className="md:col-span-2">
            <label className="block text-sm font-medium text-gray-300 mb-1">Labels (comma separated)</label>
            <input
              type="text"
              value={form.labels}
              onChange={(e) => setForm({ ...form, labels: e.target.value })}
              className="w-full px-3 py-2 bg-gray-800 border border-gray-700 rounded-lg text-white text-sm focus:outline-none focus:ring-2 focus:ring-forge-500"
              placeholder="web, ubuntu, 22.04"
            />
          </div>
          <div className="md:col-span-2">
            <button
              type="submit"
              className="px-6 py-2 bg-forge-600 text-white text-sm font-medium rounded-lg hover:bg-forge-700 transition-colors"
            >
              Create Server
            </button>
          </div>
        </form>
      )}

      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
        {servers.map((server) => (
          <ServerCard
            key={server.id}
            server={server}
            onHealthCheck={handleHealthCheck}
            onDelete={handleDelete}
          />
        ))}
      </div>

      {servers.length === 0 && !showForm && (
        <div className="text-center py-12 text-gray-500">
          <p className="text-lg mb-2">No servers yet</p>
          <p className="text-sm">Add your first server to get started</p>
        </div>
      )}
    </div>
  );
}
