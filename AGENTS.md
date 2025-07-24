# Project REXT 1 - Development Guide

## 🏗️ Project Overview

This is a full-stack web application built with Rust backend and Vue.js frontend. The project features a modern, type-safe architecture with comprehensive testing and development tooling.

The goal of this project is to serve as a demo/template for Rext, a fullstack, rust powered, batteries-included framework in active development.

### Architecture
- **Backend**: Rust with Axum web framework, Sea-ORM, SQLite database
- **Frontend**: Vue 3 with TypeScript, Vite build tool, Pinia state management
- **Database**: SQLite with Sea-ORM migrations
- **Authentication**: JWT-based authentication with Argon2 password hashing
- **API Documentation**: OpenAPI/Swagger with utoipa

## 🛠️ Technology Stack

### Backend (Rust)
- **Web Framework**: Axum 0.8.4
- **ORM**: Sea-ORM 1.1.13 with SQLite
- **Authentication**: JWT (jsonwebtoken 9.3.1) + Argon2 0.5.3
- **API Documentation**: utoipa 5.4.0 + utoipa-swagger-ui 9.0.2
- **Runtime**: Tokio (async runtime)
- **Environment**: dotenvy for .env support

### Frontend (Vue.js)
- **Framework**: Vue 3.5.17 with TypeScript
- **Build Tool**: Vite 7.0.0
- **State Management**: Pinia 3.0.3
- **Routing**: Vue Router 4.5.1
- **HTTP Client**: Axios 1.10.0
- **Schema Validation**: Zod 4.0.5
- **OpenAPI**: openapi-typescript 7.8.0

### Development Tools
- **Testing**: Vitest (unit), Playwright (e2e)
- **Linting**: ESLint with Vue/TypeScript configs
- **Formatting**: Prettier
- **Type Checking**: TypeScript + vue-tsc

## 📁 Project Structure

### Quick Reference for Agents

**Where to add code:**
- **Backend API**: `backend/bridge/` (handlers, routes, middleware, types)
- **Business Logic**: `backend/control/services/`
- **Database Models**: `backend/entity/models/`
- **Frontend Views**: `frontend/src/appearance/views/`
- **Frontend Components**: `frontend/src/appearance/components/`
- **API Client**: `frontend/src/bridge/client/` (generated)
- **Database Migrations**: `migration/src/`

### Current Structure

```
example_rext_project/
├── backend/                   # Rust backend (layered architecture)
│   ├── main.rs               # Server entry point
│   ├── bridge/               # HTTP API layer
│   │   ├── handlers/         # Request handlers (add new endpoints here)
│   │   ├── routes/           # Route definitions
│   │   ├── middleware/       # HTTP middleware
│   │   └── types/            # API types
│   ├── control/              # Business logic layer
│   │   └── services/         # Service implementations
│   ├── domain/               # Domain models
│   ├── entity/               # Database layer (Sea-ORM)
│   │   └── models/           # Entity models (add new tables here)
│   └── infrastructure/       # Cross-cutting concerns
├── frontend/                 # Vue.js frontend
│   ├── src/
│   │   ├── appearance/       # UI layer
│   │   │   ├── views/        # Page components (add new pages here)
│   │   │   └── components/   # Reusable components
│   │   └── bridge/           # API client layer (generated)
│   └── e2e/                  # End-to-end tests
├── migration/                # Database migrations (Sea-ORM)
│   └── src/                  # Migration files (add new migrations here)
└── Cargo.toml               # Rust workspace config
```

## 🗄️ Database Schema

### Users Table
The project currently implements a users table with the following schema:

```sql
CREATE TABLE users (
    id UUID PRIMARY KEY,           -- Unique user identifier
    email VARCHAR UNIQUE NOT NULL, -- User email (unique constraint)
    password_hash VARCHAR NOT NULL, -- Argon2 hashed password
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);
```

**Entity Model** (`src/entities/users.rs`):
- Uses Sea-ORM derive macros for automatic entity generation
- UUID primary key for better security and scalability
- Email uniqueness enforced at database level
- Password stored as Argon2 hash (never plaintext)
- Timestamps for audit trails

## 🔐 Authentication System

The project includes a complete authentication flow:

### Frontend Components
- **LoginView.vue**: User login form with email/password validation
- **RegisterView.vue**: User registration with form validation
- **ProfileView.vue**: User profile management page
- **HomeView.vue**: Landing/welcome page

### Authentication Features
- Form validation and error handling
- Loading states during authentication requests
- Router integration for protected routes
- Responsive form design

## 🎯 Current Development State

This project is beyond a basic template - it includes:

✅ **Implemented Features:**
- Complete user authentication UI (login, register, profile)
- Database schema with users table
- Sea-ORM entity models and migrations
- TypeScript frontend with Vue 3
- Routing between authentication pages

🚧 **In Progress/Planned:**
- Backend API endpoints for authentication
- JWT token handling and validation
- Password hashing with Argon2
- API integration with frontend forms
- User session management

## 🚀 Quick Start

### Prerequisites
- **Rust** (latest stable) - [Install from rustup.rs](https://rustup.rs/)
- **Node.js** (v18+) - [Install from nodejs.org](https://nodejs.org/)
- **Git**

### Initial Setup

1. **Clone and setup the project:**
   ```bash
   git clone <repository-url>
   cd project_rext_1
   ```

2. **Backend setup:**
   ```bash
   # Install Rust dependencies
   cargo build

   # Set up environment variables
   cp .env.example .env  # Create if needed
   # Edit .env with your configuration
   ```

3. **Database setup:**
   ```bash
   # Run migrations
   cd migration
   cargo run
   cd ..
   ```

4. **Frontend setup:**
   ```bash
   cd frontend
   npm install
   cd ..
   ```

## 🔧 Development Commands

### Backend (Rust)

#### Development
```bash
# Run backend in development mode with hot reload
cargo watch -x run

# Run backend (single execution)
cargo run

# Check code without building
cargo check

# Run with specific environment
RUST_LOG=debug cargo run
```

#### Testing
```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_name

# Run tests in specific module
cargo test module_name::
```

#### Linting & Formatting
```bash
# Check formatting
cargo fmt --check

# Format code
cargo fmt

# Run clippy linter
cargo clippy

# Fix clippy issues automatically
cargo clippy --fix
```

#### Build & Release
```bash
# Debug build
cargo build

# Release build (optimized)
cargo build --release

# Clean build artifacts
cargo clean
```

### Frontend (Vue.js)

#### Development
```bash
cd frontend

# Start development server (http://localhost:5173)
npm run dev

# Start development server with specific port
npm run dev -- --port 3000
```

#### Testing
```bash
cd frontend

# Run unit tests (Vitest)
npm run test:unit

# Run unit tests in watch mode
npm run test:unit -- --watch

# Run e2e tests (Playwright)
npm run test:e2e

# Run e2e tests in headed mode
npm run test:e2e -- --headed

# Open Playwright UI
npx playwright test --ui
```

#### Linting & Formatting
```bash
cd frontend

# Run ESLint with auto-fix
npm run lint

# Format code with Prettier
npm run format

# Type checking only
npm run type-check
```

#### Build & Preview
```bash
cd frontend

# Production build
npm run build

# Preview production build
npm run preview

# Build with type checking
npm run build
```

### Database (Sea-ORM Migrations)

#### Migration Commands
```bash
cd migration

# Run all pending migrations
cargo run

# Create a new migration
sea-orm-cli migrate generate create_table_name

# Check migration status
sea-orm-cli migrate status

# Rollback last migration
sea-orm-cli migrate down

# Fresh database (reset and run all migrations)
sea-orm-cli migrate fresh
```

#### Entity Generation
```bash
# Generate entities from database
sea-orm-cli generate entity -o src/entities

# Generate entities for specific tables
sea-orm-cli generate entity -t users,posts -o src/entities
```

## 🔄 Common Development Workflows

### Full Development Setup
```bash
# Terminal 1: Backend
cargo watch -x run

# Terminal 2: Frontend
cd frontend && npm run dev

# Terminal 3: Available for commands
```

### Authentication Development
```bash
# Work with user entity
cargo run  # Start backend with user routes

# Test authentication flow
cd frontend && npm run dev  # Start frontend
# Navigate to /login, /register, /profile
```

### Database Workflow
```bash
# Current migration (already exists)
cd migration
cargo run  # Runs m20250720_000001_create_users

# Create new migration
sea-orm-cli migrate generate create_new_table

# Reset database (CAUTION: deletes all data)
rm sqlite.db && cargo run
```

### Running Tests
```bash
# Backend tests
cargo test

# Frontend unit tests
cd frontend && npm run test:unit

# Frontend e2e tests (requires backend running)
cd frontend && npm run test:e2e
```

### Code Quality Check
```bash
# Backend
cargo fmt --check && cargo clippy && cargo test

# Frontend
cd frontend && npm run lint && npm run type-check && npm run test:unit
```

### Production Build
```bash
# Build backend
cargo build --release

# Build frontend
cd frontend && npm run build
```

## 🐛 Troubleshooting

### Common Issues

1. **Database locked error**
   ```bash
   # Stop all running processes and try again
   pkill -f "cargo run"
   pkill -f "npm run dev"
   ```

2. **Migration errors**
   ```bash
   # Reset database (CAUTION: deletes all data)
   rm sqlite.db
   cd migration && cargo run
   ```

3. **Authentication flow issues**
   ```bash
   # Check if users table exists
   sqlite3 sqlite.db ".schema users"

   # Verify migration status
   cd migration && cargo run
   ```

4. **Frontend build errors**
   ```bash
   cd frontend
   rm -rf node_modules package-lock.json
   npm install
   ```

5. **Port already in use**
   ```bash
   # Kill process on port 3000 (backend)
   lsof -ti:3000 | xargs kill -9

   # Kill process on port 5173 (frontend)
   lsof -ti:5173 | xargs kill -9
   ```

6. **Entity/Database sync issues**
   ```bash
   # Regenerate entities from current database
   sea-orm-cli generate entity -o src/entities

   # Fresh database setup
   rm sqlite.db && cd migration && cargo run
   ```

### Performance Tips
- Use `cargo check` for faster compilation during development
- Enable `incremental` compilation in Cargo.toml for debug builds
- Use `npm run dev` instead of `npm run preview` for faster frontend iteration

### Environment Variables
Create a `.env` file in the project root:
```env
DATABASE_URL=sqlite:sqlite.db
JWT_SECRET=your-secret-key-here
RUST_LOG=debug
ENVIRONMENT=development
ALLOWED_ORIGIN=https://yourdomain.com
```

**CORS Configuration:**
- **Development**: Allows `http://localhost:5173` (Vue dev server)
- **Production**: Uses `ALLOWED_ORIGIN` environment variable
- **Headers**: Restricted to essential headers only (authorization, content-type, etc.)
- **Credentials**: Enabled for authentication
- **Cache**: 1-hour preflight cache in production

## 📝 Additional Notes

- **Database**: SQLite file is gitignored but migrations are tracked
- **Hot Reload**: Both frontend and backend support hot reload in development
- **API Docs**: Available at `/swagger-ui` when backend is running
- **CORS**: Configured for local development (frontend → backend)
- **TypeScript**: Strict mode enabled with comprehensive type checking
- **Testing**: Comprehensive test setup with both unit and e2e tests
- **Code Quality**: ESLint, Prettier, and Clippy configured for consistent code style

## 🎯 Development Goals

- **Type Safety**: Full TypeScript on frontend, strong typing in Rust
- **Testing**: High test coverage with unit and integration tests
- **Documentation**: API documentation and code comments
- **Performance**: Optimized builds and efficient database queries
- **Security**: JWT authentication, password hashing, input validation
- **Developer Experience**: Hot reload, good error messages, helpful tooling