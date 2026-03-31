import { useEffect, useState } from 'react';
import {
    addSource,
    deleteSource,
    getSources,
    installRecipe,
    syncSource,
    type RecipeSource,
} from '../lib/api';

const statusColors: Record<string, string> = {
  pending: 'bg-gray-600/20 text-gray-400',
  syncing: 'bg-yellow-600/20 text-yellow-400',
  synced: 'bg-green-600/20 text-green-400',
  error: 'bg-red-600/20 text-red-400',
};

export default function Marketplace() {
  const [sources, setSources] = useState<RecipeSource[]>([]);
  const [error, setError] = useState('');
  const [showAddModal, setShowAddModal] = useState(false);
  const [addUrl, setAddUrl] = useState('');
  const [addDesc, setAddDesc] = useState('');
  const [adding, setAdding] = useState(false);
  const [syncing, setSyncing] = useState<string | null>(null);
  const [installing, setInstalling] = useState<string | null>(null);

  const load = () => {
    getSources()
      .then(setSources)
      .catch((err) => setError(err instanceof Error ? err.message : 'Failed to load sources'));
  };

  useEffect(() => {
    load();
  }, []);

  const handleAdd = async () => {
    if (!addUrl.trim()) return;
    setAdding(true);
    setError('');
    try {
      await addSource({ url: addUrl.trim(), description: addDesc.trim() || undefined });
      setShowAddModal(false);
      setAddUrl('');
      setAddDesc('');
      load();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to add source');
    } finally {
      setAdding(false);
    }
  };

  const handleSync = async (id: string) => {
    setSyncing(id);
    setError('');
    try {
      await syncSource(id);
      load();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Sync failed');
    } finally {
      setSyncing(null);
    }
  };

  const handleDelete = async (id: string) => {
    if (!confirm('Remove this source? This will unregister it (installed recipes are not removed).')) return;
    try {
      await deleteSource(id);
      load();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to delete source');
    }
  };

  const handleInstall = async (sourceId: string, slug: string) => {
    const key = `${sourceId}/${slug}`;
    setInstalling(key);
    setError('');
    try {
      await installRecipe(sourceId, slug);
      load();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Install failed');
    } finally {
      setInstalling(null);
    }
  };

  return (
    <div>
      <div className="flex items-center justify-between mb-6">
        <div>
          <h2 className="text-2xl font-bold text-white">Marketplace</h2>
          <p className="text-sm text-gray-500 mt-1">Import Ansible playbooks from GitHub repositories</p>
        </div>
        <button
          onClick={() => setShowAddModal(true)}
          className="px-4 py-2 bg-forge-600 text-white text-sm font-medium rounded-lg hover:bg-forge-700 transition-colors"
        >
          + Add Source
        </button>
      </div>

      {error && (
        <div className="bg-red-600/20 border border-red-600/30 text-red-400 text-sm p-3 rounded-lg mb-4">
          {error}
          <button onClick={() => setError('')} className="float-right text-red-400 hover:text-red-300">✕</button>
        </div>
      )}

      {/* Empty state */}
      {sources.length === 0 && (
        <div className="text-center py-16 text-gray-500">
          <div className="text-4xl mb-3">⬡</div>
          <p className="text-lg mb-1">No recipe sources yet</p>
          <p className="text-sm mb-6">Add a GitHub repository to discover and install recipes</p>
          <button
            onClick={() => setShowAddModal(true)}
            className="px-4 py-2 bg-forge-600 text-white text-sm font-medium rounded-lg hover:bg-forge-700 transition-colors"
          >
            + Add your first source
          </button>
        </div>
      )}

      {/* Sources list */}
      <div className="space-y-4">
        {sources.map((source) => (
          <div
            key={source.id}
            className="bg-gray-900 border border-gray-800 rounded-xl p-5 hover:border-gray-700 transition-colors"
          >
            {/* Source header */}
            <div className="flex items-start justify-between mb-4">
              <div className="flex-1 min-w-0">
                <div className="flex items-center gap-2 mb-1">
                  <h3 className="text-base font-semibold text-white truncate">{source.name}</h3>
                  <span className={`px-2 py-0.5 text-xs font-medium rounded-md ${statusColors[source.status] || 'bg-gray-600/20 text-gray-400'}`}>
                    {syncing === source.id ? 'syncing...' : source.status}
                  </span>
                </div>
                <a
                  href={source.url}
                  target="_blank"
                  rel="noopener noreferrer"
                  className="text-xs text-forge-400 hover:text-forge-300 font-mono break-all"
                >
                  {source.url}
                </a>
                {source.description && (
                  <p className="text-sm text-gray-400 mt-1">{source.description}</p>
                )}
                {source.sync_error && (
                  <p className="text-xs text-red-400 mt-1 font-mono">{source.sync_error}</p>
                )}
                {source.last_synced_at && (
                  <p className="text-xs text-gray-600 mt-1">
                    Last synced: {new Date(source.last_synced_at).toLocaleString()}
                  </p>
                )}
              </div>
              <div className="flex items-center gap-2 ml-4 shrink-0">
                <button
                  onClick={() => handleSync(source.id)}
                  disabled={syncing === source.id}
                  className="px-3 py-1.5 bg-gray-800 text-gray-300 text-xs font-medium rounded-lg hover:bg-gray-700 transition-colors disabled:opacity-50"
                >
                  {syncing === source.id ? '⟳ Syncing...' : '⟳ Sync'}
                </button>
                <button
                  onClick={() => handleDelete(source.id)}
                  className="px-3 py-1.5 bg-red-600/10 text-red-400 text-xs font-medium rounded-lg hover:bg-red-600/20 transition-colors"
                >
                  Remove
                </button>
              </div>
            </div>

            {/* Recipes discovered from this source */}
            {source.recipes && source.recipes.length > 0 ? (
              <div>
                <p className="text-xs text-gray-500 uppercase tracking-wider mb-2">
                  {source.recipes.length} recipe{source.recipes.length !== 1 ? 's' : ''} discovered
                </p>
                <div className="grid gap-2">
                  {source.recipes.map((recipe) => {
                    const installKey = `${source.id}/${recipe.slug}`;
                    return (
                      <div
                        key={recipe.id}
                        className="flex items-center justify-between bg-gray-800/50 border border-gray-700/50 rounded-lg px-4 py-3"
                      >
                        <div className="flex-1 min-w-0">
                          <div className="flex items-center gap-2 mb-0.5">
                            <span className="text-sm font-medium text-white">{recipe.name}</span>
                            {recipe.installed && (
                              <span className="px-1.5 py-0.5 bg-green-600/20 text-green-400 text-xs rounded-md">
                                Installed
                              </span>
                            )}
                            {recipe.version !== '1.0.0' && recipe.version !== 'auto' && (
                              <span className="text-xs text-gray-500">v{recipe.version}</span>
                            )}
                          </div>
                          {recipe.description && (
                            <p className="text-xs text-gray-500">{recipe.description}</p>
                          )}
                          <div className="flex items-center gap-1 mt-1">
                            <span className="text-xs text-gray-600 font-mono">{recipe.playbook}</span>
                            {recipe.tags.map((tag) => (
                              <span key={tag} className="px-1.5 py-0.5 bg-forge-600/20 text-forge-400 text-xs rounded-md">
                                {tag}
                              </span>
                            ))}
                          </div>
                        </div>
                        <div className="ml-4 shrink-0">
                          {recipe.installed ? (
                            <span className="text-xs text-green-400">✓ Ready</span>
                          ) : (
                            <button
                              onClick={() => handleInstall(source.id, recipe.slug)}
                              disabled={installing === installKey}
                              className="px-3 py-1.5 bg-forge-600 text-white text-xs font-medium rounded-lg hover:bg-forge-700 transition-colors disabled:opacity-50"
                            >
                              {installing === installKey ? 'Installing...' : 'Install'}
                            </button>
                          )}
                        </div>
                      </div>
                    );
                  })}
                </div>
              </div>
            ) : source.status === 'synced' ? (
              <p className="text-sm text-gray-500 italic">No recipes discovered in this repository.</p>
            ) : (
              <p className="text-sm text-gray-600 italic">
                {source.status === 'pending' ? 'Click "Sync" to discover recipes →' : ''}
              </p>
            )}
          </div>
        ))}
      </div>

      {/* Add Source Modal */}
      {showAddModal && (
        <div className="fixed inset-0 bg-black/60 flex items-center justify-center z-50 p-4">
          <div className="bg-gray-900 border border-gray-700 rounded-xl p-6 w-full max-w-lg">
            <h3 className="text-lg font-semibold text-white mb-4">Add Recipe Source</h3>

            <div className="space-y-4">
              <div>
                <label className="block text-sm text-gray-400 mb-1">GitHub URL *</label>
                <input
                  type="url"
                  placeholder="https://github.com/org/repo"
                  value={addUrl}
                  onChange={(e) => setAddUrl(e.target.value)}
                  className="w-full px-3 py-2 bg-gray-800 border border-gray-700 rounded-lg text-white text-sm focus:outline-none focus:ring-2 focus:ring-forge-500"
                />
              </div>
              <div>
                <label className="block text-sm text-gray-400 mb-1">Description (optional)</label>
                <input
                  type="text"
                  placeholder="Brief description of what this repo contains"
                  value={addDesc}
                  onChange={(e) => setAddDesc(e.target.value)}
                  className="w-full px-3 py-2 bg-gray-800 border border-gray-700 rounded-lg text-white text-sm focus:outline-none focus:ring-2 focus:ring-forge-500"
                />
              </div>

              <div className="bg-gray-800/50 border border-gray-700/50 rounded-lg p-3 text-xs text-gray-400">
                <p className="font-medium text-gray-300 mb-1">How it works:</p>
                <ol className="list-decimal list-inside space-y-0.5">
                  <li>Add the GitHub URL</li>
                  <li>Click <strong>Sync</strong> to clone the repository</li>
                  <li>xForge auto-discovers Ansible playbooks</li>
                  <li>Click <strong>Install</strong> to make a recipe available for deployment</li>
                </ol>
              </div>
            </div>

            {error && (
              <div className="bg-red-600/20 border border-red-600/30 text-red-400 text-xs p-2 rounded-lg mt-3">
                {error}
              </div>
            )}

            <div className="flex justify-end gap-3 mt-5">
              <button
                onClick={() => { setShowAddModal(false); setError(''); }}
                className="px-4 py-2 bg-gray-800 text-gray-300 text-sm font-medium rounded-lg hover:bg-gray-700 transition-colors"
              >
                Cancel
              </button>
              <button
                onClick={handleAdd}
                disabled={!addUrl.trim() || adding}
                className="px-4 py-2 bg-forge-600 text-white text-sm font-medium rounded-lg hover:bg-forge-700 transition-colors disabled:opacity-50"
              >
                {adding ? 'Adding...' : 'Add Source'}
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
