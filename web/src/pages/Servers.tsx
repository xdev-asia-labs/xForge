import { useEffect, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import ServerCard from '../components/ServerCard';
import {
    bulkHealthCheck,
    createServer,
    deleteServer,
    getServerGroups,
    getServers,
    healthCheckServer,
    type Server,
    type ServerGroup,
} from '../lib/api';

export default function Servers() {
  const [servers, setServers] = useState<Server[]>([]);
  const [groups, setGroups] = useState<ServerGroup[]>([]);
  const [error, setError] = useState('');
  const [showForm, setShowForm] = useState(false);
  const [search, setSearch] = useState('');
  const [filterGroup, setFilterGroup] = useState<string>('');
  const [selectedIds, setSelectedIds] = useState<Set<string>>(new Set());
  const [collapsedGroups, setCollapsedGroups] = useState<Set<string>>(new Set());
  const navigate = useNavigate();
  const [form, setForm] = useState({
    name: '',
    host: '',
    port: '22',
    ssh_user: 'root',
    ssh_key_path: '',
    labels: '',
    group_name: '',
  });

  const loadData = () => {
    getServers()
      .then(setServers)
      .catch((err) => setError(err.message));
    getServerGroups()
      .then(setGroups)
      .catch(() => {});
  };

  useEffect(loadData, []);

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
      loadData();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to create server');
    }
  };

  const handleHealthCheck = async (id: string) => {
    try {
      await healthCheckServer(id);
      loadData();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Health check failed');
    }
  };

  const handleDelete = async (id: string) => {
    if (!confirm('Are you sure you want to delete this server?')) return;
    try {
      await deleteServer(id);
      loadData();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to delete server');
    }
  };

  const handleBulkHealthCheck = async () => {
    if (selectedIds.size === 0) return;
    try {
      await bulkHealthCheck(Array.from(selectedIds));
      setSelectedIds(new Set());
      loadData();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Bulk health check failed');
    }
  };

  const toggleSelect = (id: string) => {
    setSelectedIds((prev) => {
      const next = new Set(prev);
      if (next.has(id)) next.delete(id);
      else next.add(id);
      return next;
    });
  };

  const toggleGroup = (name: string) => {
    setCollapsedGroups((prev) => {
      const next = new Set(prev);
      if (next.has(name)) next.delete(name);
      else next.add(name);
      return next;
    });
  };

  // Filter servers
  const filtered = servers.filter((s) => {
    if (search) {
      const q = search.toLowerCase();
      if (
        !s.name.toLowerCase().includes(q) &&
        !s.host.toLowerCase().includes(q) &&
        !s.labels.some((l) => l.toLowerCase().includes(q))
      )
        return false;
    }
    if (filterGroup && (s.group_name || 'Ungrouped') !== filterGroup)
      return false;
    return true;
  });

  // Organize by groups
  const groupedServers = filtered.reduce<Record<string, Server[]>>((acc, s) => {
    const g = s.group_name || 'Ungrouped';
    if (!acc[g]) acc[g] = [];
    acc[g].push(s);
    return acc;
  }, {});

  const groupNames = Object.keys(groupedServers).sort((a, b) =>
    a === 'Ungrouped' ? 1 : b === 'Ungrouped' ? -1 : a.localeCompare(b)
  );

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

      {/* Search & Filter Bar */}
      <div className="flex flex-wrap items-center gap-3 mb-4">
        <input
          type="text"
          value={search}
          onChange={(e) => setSearch(e.target.value)}
          placeholder="Search servers..."
          className="flex-1 min-w-[200px] px-3 py-2 bg-gray-800 border border-gray-700 rounded-lg text-white text-sm focus:outline-none focus:ring-2 focus:ring-forge-500"
        />
        <select
          value={filterGroup}
          onChange={(e) => setFilterGroup(e.target.value)}
          className="px-3 py-2 bg-gray-800 border border-gray-700 rounded-lg text-white text-sm focus:outline-none focus:ring-2 focus:ring-forge-500"
        >
          <option value="">All Groups</option>
          {groups.map((g) => (
            <option key={g.name} value={g.name}>
              {g.name} ({g.server_count})
            </option>
          ))}
        </select>
        {selectedIds.size > 0 && (
          <button
            onClick={handleBulkHealthCheck}
            className="px-4 py-2 bg-forge-600/20 text-forge-400 text-sm font-medium rounded-lg hover:bg-forge-600/30 transition-colors"
          >
            Health Check ({selectedIds.size})
          </button>
        )}
      </div>

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

      {/* Grouped Server List */}
      {groupNames.map((groupName) => (
        <div key={groupName} className="mb-6">
          <button
            onClick={() => toggleGroup(groupName)}
            className="flex items-center gap-2 mb-3 text-sm font-semibold text-gray-300 hover:text-white transition-colors"
          >
            <span className="text-xs">{collapsedGroups.has(groupName) ? '▸' : '▾'}</span>
            {groupName}
            <span className="text-xs font-normal text-gray-500">
              ({groupedServers[groupName].length})
            </span>
            {groups.find((g) => g.name === groupName) && (
              <span className="text-xs font-normal text-green-400 ml-1">
                {groups.find((g) => g.name === groupName)?.online_count} online
              </span>
            )}
          </button>

          {!collapsedGroups.has(groupName) && (
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
              {groupedServers[groupName].map((server) => (
                <div key={server.id} className="relative">
                  <div className="absolute top-3 left-3 z-10">
                    <input
                      type="checkbox"
                      checked={selectedIds.has(server.id)}
                      onChange={() => toggleSelect(server.id)}
                      className="rounded border-gray-600 bg-gray-800 text-forge-600 focus:ring-forge-500"
                    />
                  </div>
                  <ServerCard
                    server={server}
                    onHealthCheck={handleHealthCheck}
                    onDelete={handleDelete}
                  />
                  <button
                    onClick={() => navigate(`/terminal/${server.id}`)}
                    className="absolute top-3 right-3 px-2 py-1 bg-gray-800/80 text-gray-300 text-xs rounded hover:bg-forge-600/30 hover:text-forge-400 transition-colors"
                    title="Open Terminal"
                  >
                    ⏵ SSH
                  </button>
                  <button
                    onClick={() => navigate(`/servers/${server.id}/audit`)}
                    className="absolute top-12 right-3 px-2 py-1 bg-gray-800/80 text-gray-300 text-xs rounded hover:bg-forge-600/30 hover:text-forge-400 transition-colors"
                    title="Security Audit"
                  >
                    🛡 Audit
                  </button>
                </div>
              ))}
            </div>
          )}
        </div>
      ))}

      {filtered.length === 0 && !showForm && (
        <div className="text-center py-12 text-gray-500">
          <p className="text-lg mb-2">
            {servers.length === 0 ? 'No servers yet' : 'No servers match your filter'}
          </p>
          <p className="text-sm">
            {servers.length === 0 ? 'Add your first server to get started' : 'Try adjusting your search'}
          </p>
        </div>
      )}
    </div>
  );
}
