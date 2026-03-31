import { useEffect, useState } from 'react';
import { createKey, deleteKey, getKeys, type KeyStoreEntry } from '../lib/api';

export default function KeyStore() {
  const [keys, setKeys] = useState<KeyStoreEntry[]>([]);
  const [error, setError] = useState('');
  const [showForm, setShowForm] = useState(false);
  const [form, setForm] = useState({
    name: '',
    key_type: 'ssh_key',
    key_data: '',
    description: '',
  });

  const loadKeys = () => {
    getKeys()
      .then(setKeys)
      .catch((err) => setError(err.message));
  };

  useEffect(loadKeys, []);

  const handleCreate = async (e: React.FormEvent) => {
    e.preventDefault();
    setError('');
    try {
      await createKey({
        name: form.name,
        key_type: form.key_type,
        key_data: form.key_data,
        description: form.description || undefined,
      });
      setForm({ name: '', key_type: 'ssh_key', key_data: '', description: '' });
      setShowForm(false);
      loadKeys();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to create key');
    }
  };

  const handleDelete = async (id: string) => {
    if (!confirm('Are you sure you want to delete this key?')) return;
    try {
      await deleteKey(id);
      loadKeys();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to delete key');
    }
  };

  const typeBadge: Record<string, string> = {
    ssh_key: 'bg-green-600/20 text-green-400',
    login_password: 'bg-yellow-600/20 text-yellow-400',
    token: 'bg-blue-600/20 text-blue-400',
  };

  return (
    <div>
      <div className="flex items-center justify-between mb-6">
        <h2 className="text-2xl font-bold text-white">Key Store</h2>
        <button
          onClick={() => setShowForm(!showForm)}
          className="px-4 py-2 bg-forge-600 text-white text-sm font-medium rounded-lg hover:bg-forge-700 transition-colors"
        >
          {showForm ? 'Cancel' : '+ Add Key'}
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
          className="bg-gray-900 border border-gray-800 rounded-xl p-6 mb-6 space-y-4"
        >
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            <div>
              <label className="block text-sm font-medium text-gray-300 mb-1">Name</label>
              <input
                type="text"
                value={form.name}
                onChange={(e) => setForm({ ...form, name: e.target.value })}
                className="w-full px-3 py-2 bg-gray-800 border border-gray-700 rounded-lg text-white text-sm focus:outline-none focus:ring-2 focus:ring-forge-500"
                placeholder="my-ssh-key"
                required
              />
            </div>
            <div>
              <label className="block text-sm font-medium text-gray-300 mb-1">Type</label>
              <select
                value={form.key_type}
                onChange={(e) => setForm({ ...form, key_type: e.target.value })}
                className="w-full px-3 py-2 bg-gray-800 border border-gray-700 rounded-lg text-white text-sm focus:outline-none focus:ring-2 focus:ring-forge-500"
              >
                <option value="ssh_key">SSH Key</option>
                <option value="login_password">Login Password</option>
                <option value="token">Token</option>
              </select>
            </div>
          </div>
          <div>
            <label className="block text-sm font-medium text-gray-300 mb-1">Key Data</label>
            <textarea
              value={form.key_data}
              onChange={(e) => setForm({ ...form, key_data: e.target.value })}
              className="w-full px-3 py-2 bg-gray-800 border border-gray-700 rounded-lg text-white text-sm focus:outline-none focus:ring-2 focus:ring-forge-500 font-mono h-32"
              placeholder={form.key_type === 'ssh_key' ? '-----BEGIN OPENSSH PRIVATE KEY-----\n...' : 'Enter value'}
              required
            />
          </div>
          <div>
            <label className="block text-sm font-medium text-gray-300 mb-1">Description</label>
            <input
              type="text"
              value={form.description}
              onChange={(e) => setForm({ ...form, description: e.target.value })}
              className="w-full px-3 py-2 bg-gray-800 border border-gray-700 rounded-lg text-white text-sm focus:outline-none focus:ring-2 focus:ring-forge-500"
              placeholder="Production server key"
            />
          </div>
          <button
            type="submit"
            className="px-6 py-2 bg-forge-600 text-white text-sm font-medium rounded-lg hover:bg-forge-700 transition-colors"
          >
            Save Key
          </button>
        </form>
      )}

      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
        {keys.map((key) => (
          <div
            key={key.id}
            className="bg-gray-900 border border-gray-800 rounded-xl p-5 hover:border-gray-700 transition-colors"
          >
            <div className="flex items-start justify-between mb-3">
              <div>
                <h3 className="text-sm font-semibold text-white">{key.name}</h3>
                {key.description && (
                  <p className="text-xs text-gray-500 mt-1">{key.description}</p>
                )}
              </div>
              <span className={`px-2 py-0.5 text-xs font-medium rounded-md ${typeBadge[key.key_type] || 'bg-gray-600/20 text-gray-400'}`}>
                {key.key_type.replace('_', ' ')}
              </span>
            </div>
            <div className="flex items-center justify-between mt-4">
              <span className="text-xs text-gray-500">
                {key.created_at ? new Date(key.created_at).toLocaleDateString() : '-'}
              </span>
              <button
                onClick={() => handleDelete(key.id)}
                className="px-3 py-1.5 bg-red-600/20 text-red-400 text-xs font-medium rounded-lg hover:bg-red-600/30 transition-colors"
              >
                Delete
              </button>
            </div>
          </div>
        ))}
      </div>

      {keys.length === 0 && !showForm && (
        <div className="text-center py-12 text-gray-500">
          <p className="text-lg mb-2">No keys stored</p>
          <p className="text-sm">Add SSH keys or credentials for server access</p>
        </div>
      )}
    </div>
  );
}
