#!/bin/bash
# Production build script for Rext Demo Project
# Builds both frontend and backend for bare metal deployment

set -e

echo "🚀 Building Rext Demo Project for Production"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
FRONTEND_DIR="frontend"
DIST_DIR="dist"
TARGET_DIR="target/release"
BINARY_NAME="project_rext_1"

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check prerequisites
check_prerequisites() {
    print_status "Checking prerequisites..."

    if ! command -v node &> /dev/null; then
        print_error "Node.js is not installed. Please install Node.js 18+ to continue."
        exit 1
    fi

    if ! command -v npm &> /dev/null; then
        print_error "npm is not installed. Please install npm to continue."
        exit 1
    fi

    if ! command -v cargo &> /dev/null; then
        print_error "Rust/Cargo is not installed. Please install Rust to continue."
        exit 1
    fi

    print_success "All prerequisites satisfied"
}

# Build frontend
build_frontend() {
    print_status "Building frontend..."

    if [ ! -d "$FRONTEND_DIR" ]; then
        print_error "Frontend directory not found at ./$FRONTEND_DIR"
        exit 1
    fi

    cd "$FRONTEND_DIR"

    # Install dependencies if needed
    if [ ! -d "node_modules" ]; then
        print_status "Installing frontend dependencies..."
        npm install
    fi

    # Build frontend
    print_status "Running frontend build..."
    npm run build

    cd ..

    # Copy dist files to root dist directory
    if [ -d "$FRONTEND_DIR/dist" ]; then
        print_status "Copying frontend assets..."
        rm -rf "$DIST_DIR"
        cp -r "$FRONTEND_DIR/dist" "$DIST_DIR"
        print_success "Frontend build completed"
    else
        print_error "Frontend build failed - dist directory not found"
        exit 1
    fi
}

# Build backend
build_backend() {
    print_status "Building backend..."

    # Set environment for production build
    export ENVIRONMENT=production
    export BUILD_FRONTEND=false  # Frontend already built

    # Build with release optimizations
    cargo build --release

    if [ -f "$TARGET_DIR/$BINARY_NAME" ]; then
        print_success "Backend build completed"
    else
        print_error "Backend build failed - binary not found"
        exit 1
    fi
}

# Run database migrations
run_migrations() {
    print_status "Building migration binary..."

    sea-orm-cli migrate up

    if [ -f "target/release/migration" ]; then
        print_success "Migration binary built"
        print_warning "Run './target/release/migration' before starting the server"
    else
        print_error "Migration build failed"
        exit 1
    fi
}

# Create deployment package
create_package() {
    print_status "Creating deployment package..."

    PACKAGE_DIR="rext-deployment"

    # Clean and create package directory
    rm -rf "$PACKAGE_DIR"
    mkdir -p "$PACKAGE_DIR"

    # Copy binary
    cp "$TARGET_DIR/$BINARY_NAME" "$PACKAGE_DIR/rext-server"

    # Copy migration binary
    cp "target/release/migration" "$PACKAGE_DIR/migration"

    # Copy frontend assets
    cp -r "$DIST_DIR" "$PACKAGE_DIR/"

    # Copy example environment file
    cp "example.env" "$PACKAGE_DIR/.env.example"

    # Create startup script
    cat > "$PACKAGE_DIR/start.sh" << 'EOF'
#!/bin/bash
# Rext Server Startup Script

set -e

# Load environment variables
if [ -f ".env" ]; then
    export $(grep -v '^#' .env | xargs)
fi

# Set defaults
export ENVIRONMENT=${ENVIRONMENT:-production}
export DATABASE_URL=${DATABASE_URL:-sqlite:sqlite.db}
export RUST_LOG=${RUST_LOG:-info}

echo "🦖 Starting Rext Server"
echo "Environment: $ENVIRONMENT"
echo "Database: $DATABASE_URL"

# Run migrations
echo "Running database migrations..."
./migration

# Start server
echo "Starting server..."
./rext-server
EOF

    chmod +x "$PACKAGE_DIR/start.sh"

    # Create README
    cat > "$PACKAGE_DIR/README.md" << 'EOF'
# Rext Demo Project - Deployment Package

## Quick Start

1. Copy `.env.example` to `.env` and configure your environment variables
2. Run `./start.sh` to start the server

## Manual Setup

1. Set environment variables (see `.env.example`)
2. Run migrations: `./migration`
3. Start server: `./rext-server`

## Files

- `rext-server` - Main application binary
- `migration` - Database migration tool
- `dist/` - Frontend assets (served automatically)
- `start.sh` - Convenience startup script
- `.env.example` - Environment variables template

## Environment Variables

See `.env.example` for all available configuration options.

The server will run on port 3000 by default and serve both the API and frontend.
EOF

    print_success "Deployment package created in ./$PACKAGE_DIR"
}

# Main execution
main() {
    echo
    print_status "Starting production build process..."
    echo

    check_prerequisites
    echo

    build_frontend
    echo

    build_backend
    echo

    run_migrations
    echo

    create_package
    echo

    print_success "🎉 Production build completed!"
    echo
    print_status "Deployment package location: ./rext-deployment"
    print_status "To deploy:"
    print_status "  1. Copy the 'rext-deployment' directory to your server"
    print_status "  2. Configure environment variables (.env file)"
    print_status "  3. Run './start.sh' or manually run migrations and the server"
    echo
    print_warning "Don't forget to change default passwords in production!"
}

# Run main function
main "$@"