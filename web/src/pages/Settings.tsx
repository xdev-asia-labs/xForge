import { useEffect, useState } from 'react';
import {
    createNotificationChannel,
    deleteNotificationChannel,
    getNotificationChannels,
    type NotificationChannel,
} from '../lib/api';

const EVENTS = ['job.success', 'job.failed'];

export default function Settings() {
  const [channels, setChannels] = useState<NotificationChannel[]>([]);
  const [error, setError] = useState('');
  const [showForm, setShowForm] = useState(false);
  const [form, setForm] = useState({
    name: '',
    url: '',
    events: ['job.failed'] as string[],
  });

  const loadChannels = () => {
    getNotificationChannels()
      .then(setChannels)
      .catch((err) => setError(err.message));
  };

  useEffect(loadChannels, []);

  const toggleEvent = (event: string) => {
    setForm((prev) => ({
      ...prev,
      events: prev.events.includes(event)
        ? prev.events.filter((e) => e !== event)
        : [...prev.events, event],
    }));
  };

  const handleCreate = async (e: React.FormEvent) => {
    e.preventDefault();
    setError('');
    try {
      await createNotificationChannel({
        name: form.name,
        channel_type: 'webhook',
        config: { url: form.url },
        events: form.events,
      });
      setForm({ name: '', url: '', events: ['job.failed'] });
      setShowForm(false);
      loadChannels();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to create channel');
    }
  };

  const handleDelete = async (id: string) => {
    if (!confirm('Delete this notification channel?')) return;
    try {
      await deleteNotificationChannel(id);
      loadChannels();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to delete channel');
    }
  };

  return (
    <div>
      <h2 className="text-2xl font-bold text-white mb-6">Settings</h2>

      {/* Notification Channels */}
      <div className="mb-8">
        <div className="flex items-center justify-between mb-4">
          <h3 className="text-lg font-semibold text-white">Notification Channels</h3>
          <button
            onClick={() => setShowForm(!showForm)}
            className="px-4 py-2 bg-forge-600 text-white text-sm font-medium rounded-lg hover:bg-forge-700 transition-colors"
          >
            {showForm ? 'Cancel' : '+ Add Webhook'}
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
            className="bg-gray-900 border border-gray-800 rounded-xl p-6 mb-4 space-y-4"
          >
            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
              <div>
                <label className="block text-sm font-medium text-gray-300 mb-1">Name</label>
                <input
                  type="text"
                  value={form.name}
                  onChange={(e) => setForm({ ...form, name: e.target.value })}
                  className="w-full px-3 py-2 bg-gray-800 border border-gray-700 rounded-lg text-white text-sm focus:outline-none focus:ring-2 focus:ring-forge-500"
                  placeholder="Slack alerts"
                  required
                />
              </div>
              <div>
                <label className="block text-sm font-medium text-gray-300 mb-1">Webhook URL</label>
                <input
                  type="url"
                  value={form.url}
                  onChange={(e) => setForm({ ...form, url: e.target.value })}
                  className="w-full px-3 py-2 bg-gray-800 border border-gray-700 rounded-lg text-white text-sm focus:outline-none focus:ring-2 focus:ring-forge-500"
                  placeholder="https://hooks.slack.com/..."
                  required
                />
              </div>
            </div>
            <div>
              <label className="block text-sm font-medium text-gray-300 mb-2">Events</label>
              <div className="flex gap-3">
                {EVENTS.map((event) => (
                  <label key={event} className="flex items-center gap-2 cursor-pointer">
                    <input
                      type="checkbox"
                      checked={form.events.includes(event)}
                      onChange={() => toggleEvent(event)}
                      className="rounded border-gray-600 bg-gray-800 text-forge-600 focus:ring-forge-500"
                    />
                    <span className="text-sm text-gray-300">{event}</span>
                  </label>
                ))}
              </div>
            </div>
            <button
              type="submit"
              className="px-6 py-2 bg-forge-600 text-white text-sm font-medium rounded-lg hover:bg-forge-700 transition-colors"
            >
              Create Channel
            </button>
          </form>
        )}

        <div className="space-y-3">
          {channels.map((ch) => (
            <div
              key={ch.id}
              className="bg-gray-900 border border-gray-800 rounded-xl p-5 hover:border-gray-700 transition-colors"
            >
              <div className="flex items-center justify-between">
                <div>
                  <h4 className="text-sm font-semibold text-white">{ch.name}</h4>
                  <div className="flex items-center gap-3 mt-1">
                    <span className="px-2 py-0.5 bg-blue-600/20 text-blue-400 text-xs rounded-md">
                      {ch.channel_type}
                    </span>
                    {ch.events.map((ev) => (
                      <span
                        key={ev}
                        className="px-2 py-0.5 bg-gray-700 text-gray-300 text-xs rounded-md"
                      >
                        {ev}
                      </span>
                    ))}
                  </div>
                </div>
                <button
                  onClick={() => handleDelete(ch.id)}
                  className="px-3 py-1.5 bg-red-600/20 text-red-400 text-xs font-medium rounded-lg hover:bg-red-600/30 transition-colors"
                >
                  Delete
                </button>
              </div>
            </div>
          ))}
        </div>

        {channels.length === 0 && !showForm && (
          <div className="text-center py-8 text-gray-500 text-sm">
            No notification channels configured
          </div>
        )}
      </div>
    </div>
  );
}
