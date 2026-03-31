# xForge

Web-based infrastructure management tool built with Rust (Axum) and React.

## Features

-   **Server Inventory**: Manage Ubuntu servers with SSH connectivity health checks
-   **Recipe System**: YAML-defined infrastructure recipes backed by Ansible playbooks
-   **Job Execution**: Run Ansible playbooks with real-time WebSocket log streaming
-   **Authentication**: JWT-based auth with bcrypt password hashing
-   **Dashboard**: Overview of servers, active jobs, and recent history

## Tech Stack

**Backend**: Rust, Axum, SQLx (SQLite), JWT, WebSocket  
**Frontend**: React 18, TypeScript, Vite, TailwindCSS  
**Automation**: Ansible playbooks via subprocess execution

## Quick Start

### Prerequisites

-   Rust 1.77+ (with cargo)
-   Node.js 18+ (with npm)
-   Ansible (for job execution)

### Development Setup

1.  **Clone and configure**:
    ```bash
    cp .env.example .env
    ```

2.  **Build the frontend**:
    ```bash
    cd web
    npm install
    npm run build
    cd ..
    ```

3.  **Run the backend**:
    ```bash
    cargo run
    ```

4.  **Open** http&#x3A;//localhost:3000

    Default credentials: `admin` / `admin`

### Frontend Development (with hot reload)

Run the backend and frontend dev server simultaneously:

```bash
# Terminal 1 - Backend
cargo run

# Terminal 2 - Frontend (proxies API to backend)
cd web
npm run dev
```

Frontend dev server runs at http&#x3A;//localhost:5173 with API proxy to port 3000.

## Docker

### Build and run:

```bash
docker compose up -d
```

The app will be available at http&#x3A;//localhost:3000.

### Configuration

Set environment variables in `docker-compose.yml` or pass via `.env`:

| Variable       | Default                     | Description                                     |
| -------------- | --------------------------- | ----------------------------------------------- |
| `DATABASE_URL` | `sqlite:xforge.db?mode=rwc` | SQLite database path                            |
| `JWT_SECRET`   | (dev default)               | JWT signing secret --- **change in production** |
| `HOST`         | `0.0.0.0`                   | Bind address                                    |
| `PORT`         | `3000`                      | HTTP port                                       |
| `RECIPES_DIR`  | `./recipes`                 | Path to recipe definitions                      |
| `RUST_LOG`     | `xforge=info`               | Log level filter                                |

## API Endpoints

### Authentication

| Method | Path              | Description              |
| ------ | ----------------- | ------------------------ |
| POST   | `/api/auth/login` | Login, returns JWT token |

### Dashboard

| Method | Path             | Description                            |
| ------ | ---------------- | -------------------------------------- |
| GET    | `/api/dashboard` | Server count, active jobs, recent jobs |

### Servers

| Method | Path                      | Description          |
| ------ | ------------------------- | -------------------- |
| GET    | `/api/servers`            | List all servers     |
| POST   | `/api/servers`            | Create a server      |
| GET    | `/api/servers/:id`        | Get server details   |
| PUT    | `/api/servers/:id`        | Update a server      |
| DELETE | `/api/servers/:id`        | Delete a server      |
| POST   | `/api/servers/:id/health` | Run SSH health check |

### Recipes

| Method | Path                 | Description            |
| ------ | -------------------- | ---------------------- |
| GET    | `/api/recipes`       | List available recipes |
| GET    | `/api/recipes/:name` | Get recipe details     |

### Jobs

| Method | Path                   | Description            |
| ------ | ---------------------- | ---------------------- |
| GET    | `/api/jobs`            | List all jobs          |
| POST   | `/api/jobs`            | Create and start a job |
| GET    | `/api/jobs/:id`        | Get job details        |
| POST   | `/api/jobs/:id/cancel` | Cancel a running job   |

### WebSocket

| Path                  | Description                       |
| --------------------- | --------------------------------- |
| `/api/ws?job_id=<id>` | Real-time log streaming for a job |

## Recipe Format

Recipes are defined in YAML and stored in the `recipes/` directory:

```yaml
name: my-recipe
version: "1.0"
description: "What this recipe does"
params:
  - name: param_name
    type: string
    default: "value"
requires:
  min_servers: 1
  os: "ubuntu-22.04+"
playbook: playbook.yml
tags:
  - tag1
  - tag2
```

Each recipe directory contains:

-   `recipe.yaml` --- Recipe metadata and parameters
-   `playbook.yml` --- Ansible playbook to execute

## Included Recipes

| Recipe               | Description                       |
| -------------------- | --------------------------------- |
| `k8s-cluster`        | Kubernetes cluster with kubeadm   |
| `postgresql-patroni` | PostgreSQL HA with Patroni + etcd |
| `docker-host`        | Docker CE installation            |

## Project Structure

    ├── Cargo.toml              # Rust dependencies
    ├── src/
    │   ├── main.rs             # Entry point, router setup
    │   ├── config.rs           # App configuration
    │   ├── api/                # HTTP handlers
    │   │   ├── auth.rs         # Login, JWT middleware
    │   │   ├── servers.rs      # Server CRUD
    │   │   ├── recipes.rs      # Recipe listing
    │   │   ├── jobs.rs         # Job management
    │   │   └── ws.rs           # WebSocket log streaming
    │   ├── core/               # Business logic
    │   │   ├── executor.rs     # Ansible playbook runner
    │   │   ├── job_queue.rs    # Background job processor
    │   │   ├── inventory.rs    # Ansible inventory generator
    │   │   └── recipe.rs       # Recipe YAML parser
    │   ├── db/                 # Database layer
    │   │   ├── mod.rs          # Connection pool
    │   │   └── models.rs       # Data structures
    │   └── ssh/
    │       └── mod.rs          # SSH health checks
    ├── recipes/                # Infrastructure recipes
    ├── web/                    # React frontend
    ├── migrations/             # SQL migrations
    ├── Dockerfile
    └── docker-compose.yml

## License

See [LICENSE](LICENSE) for details.
