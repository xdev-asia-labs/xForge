const API_BASE = '/api';

function getToken(): string | null {
  return localStorage.getItem('xforge_token');
}

async function request<T>(
  path: string,
  options: RequestInit = {}
): Promise<T> {
  const token = getToken();
  const headers: Record<string, string> = {
    'Content-Type': 'application/json',
    ...(options.headers as Record<string, string> || {}),
  };

  if (token) {
    headers['Authorization'] = `Bearer ${token}`;
  }

  const res = await fetch(`${API_BASE}${path}`, {
    ...options,
    headers,
  });

  if (res.status === 401) {
    localStorage.removeItem('xforge_token');
    window.location.reload();
    throw new Error('Unauthorized');
  }

  if (res.status === 204) {
    return undefined as T;
  }

  if (!res.ok) {
    const body = await res.json().catch(() => ({}));
    throw new Error(body.error || `HTTP ${res.status}`);
  }

  return res.json();
}

// Auth
export async function login(username: string, password: string) {
  return request<{ token: string; user: { id: string; username: string; role: string } }>(
    '/auth/login',
    {
      method: 'POST',
      body: JSON.stringify({ username, password }),
    }
  );
}

// Dashboard
export async function getDashboard() {
  return request<{
    server_count: number;
    active_jobs: number;
    recent_jobs: Job[];
  }>('/dashboard');
}

// Servers
export interface Server {
  id: string;
  name: string;
  host: string;
  port: number;
  ssh_user: string;
  ssh_key_path: string | null;
  labels: string[];
  group_name: string | null;
  status: string;
  last_health_check: string | null;
  created_at: string;
}

export async function getServers() {
  return request<Server[]>('/servers');
}

export async function getServer(id: string) {
  return request<Server>(`/servers/${id}`);
}

export async function createServer(data: Partial<Server>) {
  return request<Server>('/servers', {
    method: 'POST',
    body: JSON.stringify(data),
  });
}

export async function updateServer(id: string, data: Partial<Server>) {
  return request<Server>(`/servers/${id}`, {
    method: 'PUT',
    body: JSON.stringify(data),
  });
}

export async function deleteServer(id: string) {
  return request<void>(`/servers/${id}`, { method: 'DELETE' });
}

export async function healthCheckServer(id: string) {
  return request<{ id: string; status: string; checked_at: string }>(
    `/servers/${id}/health`,
    { method: 'POST' }
  );
}

// Recipes
export interface RecipeParam {
  name: string;
  type: string;
  default?: string | number | boolean;
}

export interface Recipe {
  name: string;
  version: string;
  description: string;
  params: RecipeParam[] | null;
  requires: { min_servers?: number; os?: string } | null;
  playbook: string;
  tags: string[] | null;
}

export async function getRecipes() {
  return request<Recipe[]>('/recipes');
}

export async function getRecipe(name: string) {
  return request<Recipe>(`/recipes/${name}`);
}

// Jobs
export interface Job {
  id: string;
  recipe_name: string;
  server_ids: string[];
  params: Record<string, unknown> | null;
  status: string;
  output: string | null;
  started_at: string | null;
  finished_at: string | null;
  created_by: string | null;
  created_at: string;
}

export async function getJobs() {
  return request<Job[]>('/jobs');
}

export async function getJob(id: string) {
  return request<Job>(`/jobs/${id}`);
}

export async function createJob(data: {
  recipe_name: string;
  server_ids: string[];
  params?: Record<string, unknown>;
}) {
  return request<Job>('/jobs', {
    method: 'POST',
    body: JSON.stringify(data),
  });
}

export async function cancelJob(id: string) {
  return request<Job>(`/jobs/${id}/cancel`, { method: 'POST' });
}

// ─── Marketplace ──────────────────────────────────────────────────────────────

export interface SourceRecipeItem {
  id: string;
  source_id: string;
  slug: string;
  name: string;
  description: string | null;
  playbook: string;
  version: string;
  tags: string[];
  installed: boolean;
  created_at: string | null;
}

export interface RecipeSource {
  id: string;
  name: string;
  url: string;
  description: string | null;
  status: string;
  sync_error: string | null;
  last_synced_at: string | null;
  created_at: string | null;
  recipes: SourceRecipeItem[];
}

export async function getSources() {
  return request<RecipeSource[]>('/sources');
}

export async function addSource(data: { url: string; description?: string }) {
  return request<RecipeSource>('/sources', {
    method: 'POST',
    body: JSON.stringify(data),
  });
}

export async function syncSource(id: string) {
  return request<RecipeSource>(`/sources/${id}/sync`, { method: 'POST' });
}

export async function deleteSource(id: string) {
  return request<void>(`/sources/${id}`, { method: 'DELETE' });
}

export async function installRecipe(sourceId: string, slug: string) {
  return request<SourceRecipeItem>(
    `/sources/${sourceId}/recipes/${encodeURIComponent(slug)}/install`,
    { method: 'POST' }
  );
}
