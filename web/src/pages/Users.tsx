import { useEffect, useState } from 'react';
import {
    createUser,
    deleteUser,
    getUsers,
    updateUser,
    type User,
} from '../lib/api';

export default function Users() {
  const [users, setUsers] = useState<User[]>([]);
  const [error, setError] = useState('');
  const [showForm, setShowForm] = useState(false);
  const [editingUser, setEditingUser] = useState<User | null>(null);
  const [form, setForm] = useState({
    username: '',
    password: '',
    role: 'operator',
    email: '',
    display_name: '',
  });

  const loadUsers = () => {
    getUsers()
      .then(setUsers)
      .catch((err) => setError(err.message));
  };

  useEffect(loadUsers, []);

  const resetForm = () => {
    setForm({ username: '', password: '', role: 'operator', email: '', display_name: '' });
    setShowForm(false);
    setEditingUser(null);
  };

  const handleCreate = async (e: React.FormEvent) => {
    e.preventDefault();
    setError('');
    try {
      await createUser({
        username: form.username,
        password: form.password,
        role: form.role,
        email: form.email || undefined,
        display_name: form.display_name || undefined,
      });
      resetForm();
      loadUsers();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to create user');
    }
  };

  const handleUpdate = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!editingUser) return;
    setError('');
    try {
      await updateUser(editingUser.id, {
        password: form.password || undefined,
        role: form.role,
        email: form.email || undefined,
        display_name: form.display_name || undefined,
      });
      resetForm();
      loadUsers();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to update user');
    }
  };

  const handleDelete = async (id: string) => {
    if (!confirm('Are you sure you want to delete this user?')) return;
    try {
      await deleteUser(id);
      loadUsers();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to delete user');
    }
  };

  const startEdit = (user: User) => {
    setEditingUser(user);
    setForm({
      username: user.username,
      password: '',
      role: user.role,
      email: user.email || '',
      display_name: user.display_name || '',
    });
    setShowForm(true);
  };

  const roleBadge: Record<string, string> = {
    admin: 'bg-red-600/20 text-red-400',
    operator: 'bg-blue-600/20 text-blue-400',
  };

  return (
    <div>
      <div className="flex items-center justify-between mb-6">
        <h2 className="text-2xl font-bold text-white">Users</h2>
        <button
          onClick={() => { resetForm(); setShowForm(!showForm); }}
          className="px-4 py-2 bg-forge-600 text-white text-sm font-medium rounded-lg hover:bg-forge-700 transition-colors"
        >
          {showForm ? 'Cancel' : '+ Add User'}
        </button>
      </div>

      {error && (
        <div className="bg-red-600/20 border border-red-600/30 text-red-400 text-sm p-3 rounded-lg mb-4">
          {error}
        </div>
      )}

      {showForm && (
        <form
          onSubmit={editingUser ? handleUpdate : handleCreate}
          className="bg-gray-900 border border-gray-800 rounded-xl p-6 mb-6 grid grid-cols-1 md:grid-cols-2 gap-4"
        >
          <div>
            <label className="block text-sm font-medium text-gray-300 mb-1">Username</label>
            <input
              type="text"
              value={form.username}
              onChange={(e) => setForm({ ...form, username: e.target.value })}
              className="w-full px-3 py-2 bg-gray-800 border border-gray-700 rounded-lg text-white text-sm focus:outline-none focus:ring-2 focus:ring-forge-500"
              required
              disabled={!!editingUser}
            />
          </div>
          <div>
            <label className="block text-sm font-medium text-gray-300 mb-1">
              Password {editingUser && '(leave blank to keep)'}
            </label>
            <input
              type="password"
              value={form.password}
              onChange={(e) => setForm({ ...form, password: e.target.value })}
              className="w-full px-3 py-2 bg-gray-800 border border-gray-700 rounded-lg text-white text-sm focus:outline-none focus:ring-2 focus:ring-forge-500"
              required={!editingUser}
              minLength={4}
            />
          </div>
          <div>
            <label className="block text-sm font-medium text-gray-300 mb-1">Role</label>
            <select
              value={form.role}
              onChange={(e) => setForm({ ...form, role: e.target.value })}
              className="w-full px-3 py-2 bg-gray-800 border border-gray-700 rounded-lg text-white text-sm focus:outline-none focus:ring-2 focus:ring-forge-500"
            >
              <option value="operator">Operator</option>
              <option value="admin">Admin</option>
            </select>
          </div>
          <div>
            <label className="block text-sm font-medium text-gray-300 mb-1">Email</label>
            <input
              type="email"
              value={form.email}
              onChange={(e) => setForm({ ...form, email: e.target.value })}
              className="w-full px-3 py-2 bg-gray-800 border border-gray-700 rounded-lg text-white text-sm focus:outline-none focus:ring-2 focus:ring-forge-500"
            />
          </div>
          <div className="md:col-span-2">
            <label className="block text-sm font-medium text-gray-300 mb-1">Display Name</label>
            <input
              type="text"
              value={form.display_name}
              onChange={(e) => setForm({ ...form, display_name: e.target.value })}
              className="w-full px-3 py-2 bg-gray-800 border border-gray-700 rounded-lg text-white text-sm focus:outline-none focus:ring-2 focus:ring-forge-500"
            />
          </div>
          <div className="md:col-span-2">
            <button
              type="submit"
              className="px-6 py-2 bg-forge-600 text-white text-sm font-medium rounded-lg hover:bg-forge-700 transition-colors"
            >
              {editingUser ? 'Update User' : 'Create User'}
            </button>
          </div>
        </form>
      )}

      <div className="bg-gray-900 border border-gray-800 rounded-xl overflow-hidden">
        <table className="w-full">
          <thead>
            <tr className="border-b border-gray-800 text-left text-xs text-gray-500 uppercase tracking-wider">
              <th className="px-6 py-3">Username</th>
              <th className="px-6 py-3">Display Name</th>
              <th className="px-6 py-3">Email</th>
              <th className="px-6 py-3">Role</th>
              <th className="px-6 py-3">Created</th>
              <th className="px-6 py-3 text-right">Actions</th>
            </tr>
          </thead>
          <tbody className="divide-y divide-gray-800">
            {users.map((user) => (
              <tr key={user.id} className="hover:bg-gray-800/50 transition-colors">
                <td className="px-6 py-4 text-sm text-white font-medium">{user.username}</td>
                <td className="px-6 py-4 text-sm text-gray-400">{user.display_name || '-'}</td>
                <td className="px-6 py-4 text-sm text-gray-400">{user.email || '-'}</td>
                <td className="px-6 py-4">
                  <span className={`px-2.5 py-1 text-xs font-medium rounded-md ${roleBadge[user.role] || 'bg-gray-600/20 text-gray-400'}`}>
                    {user.role}
                  </span>
                </td>
                <td className="px-6 py-4 text-xs text-gray-500">
                  {user.created_at ? new Date(user.created_at).toLocaleDateString() : '-'}
                </td>
                <td className="px-6 py-4 text-right">
                  <div className="flex justify-end gap-2">
                    <button
                      onClick={() => startEdit(user)}
                      className="px-3 py-1.5 bg-forge-600/20 text-forge-400 text-xs font-medium rounded-lg hover:bg-forge-600/30 transition-colors"
                    >
                      Edit
                    </button>
                    <button
                      onClick={() => handleDelete(user.id)}
                      className="px-3 py-1.5 bg-red-600/20 text-red-400 text-xs font-medium rounded-lg hover:bg-red-600/30 transition-colors"
                    >
                      Delete
                    </button>
                  </div>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
        {users.length === 0 && (
          <div className="text-center py-8 text-gray-500 text-sm">No users found</div>
        )}
      </div>
    </div>
  );
}
