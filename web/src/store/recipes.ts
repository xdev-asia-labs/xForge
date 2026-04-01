import { create } from 'zustand';
import { getRecipe, getRecipes, type Recipe } from '../lib/api';

interface RecipesState {
  recipes: Recipe[];
  loading: boolean;
  error: string;

  fetchRecipes: () => Promise<void>;
  getRecipe: (name: string) => Promise<Recipe>;
}

export const useRecipesStore = create<RecipesState>((set) => ({
  recipes: [],
  loading: false,
  error: '',

  fetchRecipes: async () => {
    set({ loading: true, error: '' });
    try {
      const data = await getRecipes();
      set({ recipes: data });
    } catch (err) {
      set({ error: err instanceof Error ? err.message : 'Failed to fetch recipes' });
    } finally {
      set({ loading: false });
    }
  },

  getRecipe: async (name) => {
    return getRecipe(name);
  },
}));
