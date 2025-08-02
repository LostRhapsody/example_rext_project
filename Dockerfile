# Multi-stage Dockerfile for Rext Demo Project
# Builds both frontend and backend into a single production image

# ==============================================================================
# Frontend Build Stage
# ==============================================================================
FROM node:22-alpine AS frontend-builder

WORKDIR /app/frontend

# Copy frontend package files
COPY frontend/package*.json ./

# Install dependencies with clean cache
RUN npm ci --only=production --no-audit --no-fund

# Copy frontend source code
COPY frontend/ ./

# Build frontend for production
RUN npm run build

# ==============================================================================
# Rust Build Stage
# ==============================================================================
FROM rust:1.88-alpine AS rust-builder

# Install build dependencies
RUN apk add --no-cache \
    musl-dev \
    pkgconfig \
    openssl-dev \
    sqlite-dev \
    curl

WORKDIR /app

# Copy Cargo files for dependency caching
COPY Cargo.toml Cargo.lock ./
COPY migration/Cargo.toml ./migration/

# Create dummy source files to build dependencies first
RUN mkdir -p backend/src migration/src && \
    echo "fn main() {}" > backend/src/main.rs && \
    echo "fn main() {}" > migration/src/main.rs && \
    touch backend/src/lib.rs migration/src/lib.rs

# Build dependencies (this layer will be cached)
RUN cargo build --release

# Remove dummy files
RUN rm -rf backend/src migration/src

# Copy actual source code
COPY backend/ ./backend/
RUN ls -l ./backend

COPY migration/ ./migration/

# Copy frontend dist files from frontend builder
COPY --from=frontend-builder /app/frontend/dist ./dist

# Build the application
RUN cargo build --release

# ==============================================================================
# Production Runtime Stage
# ==============================================================================
FROM alpine:latest AS production

# Install runtime dependencies
RUN apk add --no-cache \
    sqlite \
    ca-certificates \
    curl

# Create app user for security
RUN addgroup -g 1001 -S appgroup && \
    adduser -u 1001 -S appuser -G appgroup

# Create directories with proper permissions
RUN mkdir -p /app/data /app/dist && \
    chown -R appuser:appgroup /app

WORKDIR /app

# Copy binary from rust builder
COPY --from=rust-builder /app/target/release/project_rext_1 ./rext-server

# Copy frontend assets from rust builder (which got them from frontend builder)
COPY --from=rust-builder /app/dist ./dist

# Copy migration binary for database setup
COPY --from=rust-builder /app/target/release/migration ./migration

# Make binaries executable
RUN chmod +x ./rext-server ./migration

# Switch to non-root user
USER appuser

# Expose port
EXPOSE 3000

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:3000/ || exit 1

# Set default environment
ENV ENVIRONMENT=production
ENV DATABASE_URL=sqlite:/app/data/sqlite.db
ENV RUST_LOG=info

# Start script that runs migrations then the server
CMD ["sh", "-c", "\
    echo 'Starting Rext Server...' && \
    echo 'Running database migrations...' && \
    DATABASE_URL=${DATABASE_URL} ./migration && \
    echo 'Starting server...' && \
    ./rext-server \
"]

# ==============================================================================
# Development Stage (for development with Docker)
# ==============================================================================
FROM rust:1.88-alpine AS development

# Install development dependencies
RUN apk add --no-cache \
    musl-dev \
    pkgconfig \
    openssl-dev \
    sqlite-dev \
    curl

# Install cargo-watch for hot reload
RUN cargo install cargo-watch

WORKDIR /app

# Copy Cargo files
COPY Cargo.toml Cargo.lock ./
COPY migration/Cargo.toml ./migration/

# Create directory structure
RUN mkdir -p backend migration/src && \
    echo "fn main() {}" > backend/main.rs && \
    echo "fn main() {}" > migration/src/main.rs

# Pre-build dependencies
RUN cargo build

# Remove dummy files
RUN rm -rf backend/main.rs migration/src/main.rs

EXPOSE 3000

# Default command for development
CMD ["cargo", "watch", "-x", "run"]