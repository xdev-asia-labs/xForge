import type { Recipe } from '../lib/api';

interface RecipeCardProps {
  recipe: Recipe;
  onDeploy: (name: string) => void;
}

export default function RecipeCard({ recipe, onDeploy }: RecipeCardProps) {
  return (
    <div className="bg-gray-900 border border-gray-800 rounded-xl p-5 hover:border-gray-700 transition-colors">
      <div className="flex items-start justify-between mb-3">
        <div>
          <h3 className="text-lg font-semibold text-white">{recipe.name}</h3>
          <p className="text-xs text-gray-500">v{recipe.version}</p>
        </div>
      </div>

      <p className="text-sm text-gray-400 mb-4">{recipe.description}</p>

      {recipe.tags && recipe.tags.length > 0 && (
        <div className="flex flex-wrap gap-1.5 mb-3">
          {recipe.tags.map((tag) => (
            <span
              key={tag}
              className="px-2 py-0.5 bg-purple-600/20 text-purple-400 text-xs rounded-md"
            >
              {tag}
            </span>
          ))}
        </div>
      )}

      {recipe.requires && (
        <div className="text-xs text-gray-500 mb-3 space-y-1">
          {recipe.requires.min_servers && (
            <p>Min servers: <span className="text-gray-300">{recipe.requires.min_servers}</span></p>
          )}
          {recipe.requires.os && (
            <p>OS: <span className="text-gray-300">{recipe.requires.os}</span></p>
          )}
        </div>
      )}

      {recipe.params && recipe.params.length > 0 && (
        <div className="mb-4">
          <p className="text-xs text-gray-500 mb-1.5">Parameters:</p>
          <div className="space-y-1">
            {recipe.params.map((param) => (
              <div key={param.name} className="text-xs font-mono">
                <span className="text-forge-400">{param.name}</span>
                <span className="text-gray-600">: {param.type}</span>
                {param.default !== undefined && (
                  <span className="text-gray-500"> = {String(param.default)}</span>
                )}
              </div>
            ))}
          </div>
        </div>
      )}

      <button
        onClick={() => onDeploy(recipe.name)}
        className="w-full px-4 py-2 bg-forge-600 text-white text-sm font-medium rounded-lg hover:bg-forge-700 transition-colors"
      >
        Deploy
      </button>
    </div>
  );
}
