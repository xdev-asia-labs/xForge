# xForge Project Guidelines

## Architecture
- **Backend**: Rust with Axum 0.7, SQLx (SQLite), Tokio async runtime
- **Frontend**: React 18 + TypeScript + Vite + TailwindCSS
- **Automation**: Ansible playbooks executed as subprocesses
- **Structure**: `src/` (Rust backend), `web/` (React frontend), `recipes/` (YAML + playbooks), `migrations/` (SQL)

## Build & Run
```bash
# Frontend
cd web && npm install && npm run build

# Backend (serves built frontend via rust-embed)
cargo run

# Frontend dev (hot reload, proxies /api to :3000)
cd web && npm run dev
```

## Rust Conventions
- Error return type: `Result<T, (StatusCode, Json<serde_json::Value>)>` for API handlers
- Error body format: `Json(json!({"error": "message"}))` — no nesting, no error codes
- Status codes: 200 implicit, 201 for POST creation, 400 validation, 401 auth, 404 not found, 500 internal
- Database models derive `sqlx::FromRow`; API responses use separate structs with `impl From<Model>`
- JSON columns in SQLite stored as strings, parsed via `serde_json::from_str` in From impls
- Shared state: `AppState` with `SqlitePool`, `Arc<AppConfig>`, `broadcast::Sender<String>`
- UUIDs as strings (not Uuid type) in database models
- Import order: framework → external crates → internal modules

## React/TypeScript Conventions
- Strict TypeScript (`strict: true`, `noUnusedLocals`, `noUnusedParameters`)
- Functional components with explicit `useState<T>()` typing
- Form state as single object updated via spread: `setForm({ ...form, field: value })`
- Error catches: `err instanceof Error ? err.message : 'Fallback message'`
- API client in `lib/api.ts` uses generic `request<T>()` wrapper with auto auth injection
- Type imports: `import { func, type Type } from './module'`

## TailwindCSS Theme
- Dark theme only — backgrounds: `bg-gray-950`/`bg-gray-900`, borders: `border-gray-800`
- Brand color: `forge-*` palette (primary: `bg-forge-600`, hover: `hover:bg-forge-700`)
- Cards: `bg-gray-900 border border-gray-800 rounded-xl p-5 hover:border-gray-700 transition-colors`
- Inputs: `bg-gray-800 border border-gray-700 rounded-lg text-white text-sm focus:ring-2 focus:ring-forge-500`
- Errors: `bg-red-600/20 border border-red-600/30 text-red-400`
- Status colors use transparency: `bg-green-600/20 text-green-400`, `bg-yellow-600/20 text-yellow-400`

## Recipes
- YAML format in `recipes/<name>/recipe.yaml` with companion `playbook.yml`
- Required fields: `name`, `version`, `description`, `playbook`
- Optional: `params` (list of `{name, type, default}`), `requires` (`{min_servers, os}`), `tags`
