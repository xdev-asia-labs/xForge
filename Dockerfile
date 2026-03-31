# Build frontend
FROM node:20-alpine AS frontend
WORKDIR /app/web
COPY web/package.json web/package-lock.json* ./
RUN npm install
COPY web/ .
RUN npm run build

# Build backend
FROM rust:1.77-slim AS backend
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY Cargo.toml Cargo.lock* ./
COPY src/ src/
COPY migrations/ migrations/
COPY --from=frontend /app/web/dist web/dist
RUN cargo build --release

# Runtime
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y \
    ca-certificates \
    openssh-client \
    ansible \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=backend /app/target/release/xforge /app/xforge
COPY recipes/ /app/recipes/
COPY migrations/ /app/migrations/

ENV DATABASE_URL=sqlite:/app/data/xforge.db?mode=rwc
ENV RECIPES_DIR=/app/recipes
ENV HOST=0.0.0.0
ENV PORT=3000

VOLUME ["/app/data"]

EXPOSE 3000

CMD ["/app/xforge"]
