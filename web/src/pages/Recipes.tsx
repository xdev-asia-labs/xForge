import { useEffect, useState } from 'react';
import RecipeCard from '../components/RecipeCard';
import { createJob, getRecipes, getServers, type Recipe, type Server } from '../lib/api';

export default function Recipes() {
  const [recipes, setRecipes] = useState<Recipe[]>([]);
  const [servers, setServers] = useState<Server[]>([]);
  const [error, setError] = useState('');
  const [deployRecipe, setDeployRecipe] = useState<string | null>(null);
  const [selectedServers, setSelectedServers] = useState<string[]>([]);
  const [params, setParams] = useState<Record<string, string>>({});
  const [deploying, setDeploying] = useState(false);

  useEffect(() => {
    getRecipes()
      .then(setRecipes)
      .catch((err) => setError(err.message));
    getServers()
      .then(setServers)
      .catch(() => {});
  }, []);

  const handleDeploy = (name: string) => {
    setDeployRecipe(name);
    setSelectedServers([]);
    setParams({});

    const recipe = recipes.find((r) => r.name === name);
    if (recipe?.params) {
      const defaults: Record<string, string> = {};
      recipe.params.forEach((p) => {
        if (p.default !== undefined) {
          defaults[p.name] = String(p.default);
        }
      });
      setParams(defaults);
    }
  };

  const handleSubmitDeploy = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!deployRecipe || selectedServers.length === 0) return;

    setDeploying(true);
    setError('');

    try {
      await createJob({
        recipe_name: deployRecipe,
        server_ids: selectedServers,
        params: Object.keys(params).length > 0 ? params : undefined,
      });
      setDeployRecipe(null);
      alert('Job created successfully! Check the Jobs page for status.');
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to create job');
    } finally {
      setDeploying(false);
    }
  };

  const toggleServer = (id: string) => {
    setSelectedServers((prev) =>
      prev.includes(id) ? prev.filter((s) => s !== id) : [...prev, id]
    );
  };

  const activeRecipe = recipes.find((r) => r.name === deployRecipe);

  return (
    <div>
      <h2 className="text-2xl font-bold text-white mb-6">Recipes</h2>

      {error && (
        <div className="bg-red-600/20 border border-red-600/30 text-red-400 text-sm p-3 rounded-lg mb-4">
          {error}
        </div>
      )}

      {/* Deploy Dialog */}
      {deployRecipe && activeRecipe && (
        <div className="bg-gray-900 border border-forge-600/30 rounded-xl p-6 mb-6">
          <h3 className="text-lg font-semibold text-white mb-4">
            Deploy: {activeRecipe.name}
          </h3>
          <form onSubmit={handleSubmitDeploy} className="space-y-4">
            <div>
              <label className="block text-sm font-medium text-gray-300 mb-2">
                Select Servers
                {activeRecipe.requires?.min_servers && (
                  <span className="text-gray-500 ml-2">
                    (min: {activeRecipe.requires.min_servers})
                  </span>
                )}
              </label>
              <div className="grid grid-cols-2 md:grid-cols-3 gap-2">
                {servers.map((server) => (
                  <label
                    key={server.id}
                    className={`flex items-center gap-2 p-3 rounded-lg border cursor-pointer transition-colors ${
                      selectedServers.includes(server.id)
                        ? 'border-forge-500 bg-forge-600/10'
                        : 'border-gray-700 bg-gray-800 hover:border-gray-600'
                    }`}
                  >
                    <input
                      type="checkbox"
                      checked={selectedServers.includes(server.id)}
                      onChange={() => toggleServer(server.id)}
                      className="accent-forge-500"
                    />
                    <div>
                      <div className="text-sm text-white">{server.name}</div>
                      <div className="text-xs text-gray-500">{server.host}</div>
                    </div>
                  </label>
                ))}
              </div>
              {servers.length === 0 && (
                <p className="text-sm text-gray-500">
                  No servers available. Add servers first.
                </p>
              )}
            </div>

            {activeRecipe.params && activeRecipe.params.length > 0 && (
              <div>
                <label className="block text-sm font-medium text-gray-300 mb-2">
                  Parameters
                </label>
                <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
                  {activeRecipe.params.map((param) => (
                    <div key={param.name}>
                      <label className="block text-xs text-gray-400 mb-1">
                        {param.name}{' '}
                        <span className="text-gray-600">({param.type})</span>
                      </label>
                      <input
                        type="text"
                        value={params[param.name] || ''}
                        onChange={(e) =>
                          setParams({ ...params, [param.name]: e.target.value })
                        }
                        className="w-full px-3 py-2 bg-gray-800 border border-gray-700 rounded-lg text-white text-sm focus:outline-none focus:ring-2 focus:ring-forge-500"
                        placeholder={
                          param.default !== undefined
                            ? String(param.default)
                            : ''
                        }
                      />
                    </div>
                  ))}
                </div>
              </div>
            )}

            <div className="flex gap-3">
              <button
                type="submit"
                disabled={deploying || selectedServers.length === 0}
                className="px-6 py-2 bg-forge-600 text-white text-sm font-medium rounded-lg hover:bg-forge-700 disabled:opacity-50 transition-colors"
              >
                {deploying ? 'Creating Job...' : 'Deploy'}
              </button>
              <button
                type="button"
                onClick={() => setDeployRecipe(null)}
                className="px-6 py-2 bg-gray-800 text-gray-300 text-sm font-medium rounded-lg hover:bg-gray-700 transition-colors"
              >
                Cancel
              </button>
            </div>
          </form>
        </div>
      )}

      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
        {recipes.map((recipe) => (
          <RecipeCard key={recipe.name} recipe={recipe} onDeploy={handleDeploy} />
        ))}
      </div>

      {recipes.length === 0 && (
        <div className="text-center py-12 text-gray-500">
          <p className="text-lg mb-2">No recipes found</p>
          <p className="text-sm">Add recipe YAML files to the recipes/ directory</p>
        </div>
      )}
    </div>
  );
}
