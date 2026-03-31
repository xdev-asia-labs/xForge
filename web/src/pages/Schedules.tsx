import { useEffect, useState } from 'react';
import {
    createSchedule,
    deleteSchedule,
    getRecipes,
    getSchedules,
    getServers,
    updateSchedule,
    type Recipe,
    type Schedule,
    type Server,
} from '../lib/api';

export default function Schedules() {
  const [schedules, setSchedules] = useState<Schedule[]>([]);
  const [recipes, setRecipes] = useState<Recipe[]>([]);
  const [servers, setServers] = useState<Server[]>([]);
  const [error, setError] = useState('');
  const [showForm, setShowForm] = useState(false);
  const [form, setForm] = useState({
    name: '',
    recipe_name: '',
    server_ids: [] as string[],
    cron_expression: '',
  });

  const loadData = () => {
    getSchedules().then(setSchedules).catch((err) => setError(err.message));
    getRecipes().then(setRecipes).catch(() => {});
    getServers().then(setServers).catch(() => {});
  };

  useEffect(loadData, []);

  const handleCreate = async (e: React.FormEvent) => {
    e.preventDefault();
    setError('');
    try {
      await createSchedule({
        name: form.name,
        recipe_name: form.recipe_name,
        server_ids: form.server_ids,
        cron_expression: form.cron_expression,
      });
      setForm({ name: '', recipe_name: '', server_ids: [], cron_expression: '' });
      setShowForm(false);
      loadData();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to create schedule');
    }
  };

  const handleToggle = async (schedule: Schedule) => {
    try {
      await updateSchedule(schedule.id, { enabled: !schedule.enabled });
      loadData();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to update schedule');
    }
  };

  const handleDelete = async (id: string) => {
    if (!confirm('Are you sure you want to delete this schedule?')) return;
    try {
      await deleteSchedule(id);
      loadData();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to delete schedule');
    }
  };

  const toggleServer = (id: string) => {
    setForm((prev) => ({
      ...prev,
      server_ids: prev.server_ids.includes(id)
        ? prev.server_ids.filter((s) => s !== id)
        : [...prev.server_ids, id],
    }));
  };

  return (
    <div>
      <div className="flex items-center justify-between mb-6">
        <h2 className="text-2xl font-bold text-white">Schedules</h2>
        <button
          onClick={() => setShowForm(!showForm)}
          className="px-4 py-2 bg-forge-600 text-white text-sm font-medium rounded-lg hover:bg-forge-700 transition-colors"
        >
          {showForm ? 'Cancel' : '+ New Schedule'}
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
                placeholder="Daily backup"
                required
              />
            </div>
            <div>
              <label className="block text-sm font-medium text-gray-300 mb-1">
                Cron Expression
                <span className="text-gray-500 font-normal ml-1">(min hour dom month dow)</span>
              </label>
              <input
                type="text"
                value={form.cron_expression}
                onChange={(e) => setForm({ ...form, cron_expression: e.target.value })}
                className="w-full px-3 py-2 bg-gray-800 border border-gray-700 rounded-lg text-white text-sm focus:outline-none focus:ring-2 focus:ring-forge-500 font-mono"
                placeholder="0 2 * * *"
                required
              />
            </div>
          </div>

          <div>
            <label className="block text-sm font-medium text-gray-300 mb-1">Recipe</label>
            <select
              value={form.recipe_name}
              onChange={(e) => setForm({ ...form, recipe_name: e.target.value })}
              className="w-full px-3 py-2 bg-gray-800 border border-gray-700 rounded-lg text-white text-sm focus:outline-none focus:ring-2 focus:ring-forge-500"
              required
            >
              <option value="">Select a recipe</option>
              {recipes.map((r) => (
                <option key={r.name} value={r.name}>{r.name}</option>
              ))}
            </select>
          </div>

          <div>
            <label className="block text-sm font-medium text-gray-300 mb-2">Servers</label>
            <div className="flex flex-wrap gap-2">
              {servers.map((s) => (
                <button
                  key={s.id}
                  type="button"
                  onClick={() => toggleServer(s.id)}
                  className={`px-3 py-1.5 text-xs font-medium rounded-lg transition-colors ${
                    form.server_ids.includes(s.id)
                      ? 'bg-forge-600 text-white'
                      : 'bg-gray-800 text-gray-400 hover:bg-gray-700'
                  }`}
                >
                  {s.name}
                </button>
              ))}
            </div>
          </div>

          <button
            type="submit"
            className="px-6 py-2 bg-forge-600 text-white text-sm font-medium rounded-lg hover:bg-forge-700 transition-colors"
          >
            Create Schedule
          </button>
        </form>
      )}

      <div className="space-y-3">
        {schedules.map((schedule) => (
          <div
            key={schedule.id}
            className="bg-gray-900 border border-gray-800 rounded-xl p-5 hover:border-gray-700 transition-colors"
          >
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-4">
                <button
                  onClick={() => handleToggle(schedule)}
                  className={`w-10 h-5 rounded-full relative transition-colors ${
                    schedule.enabled ? 'bg-forge-600' : 'bg-gray-700'
                  }`}
                >
                  <span
                    className={`absolute top-0.5 w-4 h-4 rounded-full bg-white transition-transform ${
                      schedule.enabled ? 'left-5' : 'left-0.5'
                    }`}
                  />
                </button>
                <div>
                  <h3 className="text-sm font-semibold text-white">{schedule.name}</h3>
                  <div className="flex items-center gap-3 mt-1">
                    <span className="text-xs text-gray-500 font-mono">{schedule.cron_expression}</span>
                    <span className="text-xs text-gray-500">•</span>
                    <span className="text-xs text-gray-400">{schedule.recipe_name}</span>
                    <span className="text-xs text-gray-500">•</span>
                    <span className="text-xs text-gray-500">
                      {schedule.server_ids.length} server{schedule.server_ids.length !== 1 ? 's' : ''}
                    </span>
                  </div>
                </div>
              </div>

              <div className="flex items-center gap-4">
                <div className="text-right">
                  {schedule.next_run_at && (
                    <div className="text-xs text-gray-400">
                      Next: {new Date(schedule.next_run_at).toLocaleString()}
                    </div>
                  )}
                  {schedule.last_run_at && (
                    <div className="text-xs text-gray-500">
                      Last: {new Date(schedule.last_run_at).toLocaleString()}
                    </div>
                  )}
                </div>
                <button
                  onClick={() => handleDelete(schedule.id)}
                  className="px-3 py-1.5 bg-red-600/20 text-red-400 text-xs font-medium rounded-lg hover:bg-red-600/30 transition-colors"
                >
                  Delete
                </button>
              </div>
            </div>
          </div>
        ))}
      </div>

      {schedules.length === 0 && !showForm && (
        <div className="text-center py-12 text-gray-500">
          <p className="text-lg mb-2">No schedules yet</p>
          <p className="text-sm">Create a schedule to run recipes automatically</p>
        </div>
      )}
    </div>
  );
}
