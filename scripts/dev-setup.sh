#!/bin/bash
# Development setup script for Rext Demo Project

set -e

echo "🛠️ Setting up Rext Demo Project for Development"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

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
    
    NODE_VERSION=$(node --version | cut -d'v' -f2 | cut -d'.' -f1)
    if [ "$NODE_VERSION" -lt 18 ]; then
        print_warning "Node.js version $NODE_VERSION detected. Version 18+ recommended."
    fi
    
    print_success "Prerequisites satisfied"
}

# Setup environment
setup_environment() {
    print_status "Setting up environment..."
    
    if [ ! -f ".env" ]; then
        if [ -f "example.env" ]; then
            cp example.env .env
            print_success "Created .env file from example.env"
            print_warning "Please review and update .env file with your settings"
        else
            print_error "No example.env file found. Please create a .env file manually."
            exit 1
        fi
    else
        print_status ".env file already exists"
    fi
}

# Install backend dependencies
setup_backend() {
    print_status "Setting up backend dependencies..."
    
    # Check if cargo dependencies need installing
    print_status "Installing Rust dependencies..."
    cargo check
    
    print_success "Backend dependencies installed"
}

# Install frontend dependencies
setup_frontend() {
    print_status "Setting up frontend dependencies..."
    
    cd frontend
    
    if [ ! -d "node_modules" ]; then
        print_status "Installing frontend dependencies..."
        npm install
    else
        print_status "Frontend dependencies already installed"
    fi
    
    cd ..
    
    print_success "Frontend dependencies installed"
}

# Setup database
setup_database() {
    print_status "Setting up database..."
    
    # Check if sea-orm-cli is available
    if ! command -v sea-orm-cli &> /dev/null; then
        print_status "Installing sea-orm-cli..."
        cargo install sea-orm-cli
    fi
    
    # Run migrations
    print_status "Running database migrations..."
    cd migration
    cargo run
    cd ..
    
    print_success "Database setup completed"
}

# Install development tools
install_dev_tools() {
    print_status "Installing development tools..."
    
    # Install cargo-watch for hot reload
    if ! command -v cargo-watch &> /dev/null; then
        print_status "Installing cargo-watch for hot reload..."
        cargo install cargo-watch
    fi
    
    print_success "Development tools installed"
}

# Create development scripts
create_dev_scripts() {
    print_status "Creating development convenience scripts..."
    
    # Backend dev script
    cat > "dev-backend.sh" << 'EOF'
#!/bin/bash
echo "🦖 Starting backend in development mode..."
export ENVIRONMENT=development
cargo watch -x run
EOF
    chmod +x dev-backend.sh
    
    # Frontend dev script
    cat > "dev-frontend.sh" << 'EOF'
#!/bin/bash
echo "🎨 Starting frontend in development mode..."
cd frontend && npm run dev
EOF
    chmod +x dev-frontend.sh
    
    # Full dev script
    cat > "dev-full.sh" << 'EOF'
#!/bin/bash
echo "🚀 Starting full development environment..."
echo "This will start both backend and frontend servers"
echo "Backend: http://localhost:3000"
echo "Frontend: http://localhost:5173"
echo ""

# Kill any existing processes on the ports
pkill -f "cargo.*run" || true
pkill -f "npm.*dev" || true

# Function to cleanup on exit
cleanup() {
    echo ""
    echo "Shutting down development servers..."
    pkill -f "cargo.*run" || true
    pkill -f "npm.*dev" || true
    exit
}

# Set trap to cleanup on script exit
trap cleanup INT TERM

# Start backend in background
export ENVIRONMENT=development
cargo watch -x run &
BACKEND_PID=$!

# Wait for backend to start
echo "Waiting for backend to start..."
sleep 5

# Start frontend in background
cd frontend && npm run dev &
FRONTEND_PID=$!

echo ""
echo "✅ Development servers started!"
echo "Backend: http://localhost:3000"
echo "Frontend: http://localhost:5173"
echo "API Docs: http://localhost:3000/scalar"
echo ""
echo "Press Ctrl+C to stop all servers"

# Wait for processes
wait $BACKEND_PID $FRONTEND_PID
EOF
    chmod +x dev-full.sh
    
    print_success "Development scripts created"
}

# Main execution
main() {
    echo
    print_status "Starting development setup..."
    echo
    
    check_prerequisites
    echo
    
    setup_environment
    echo
    
    setup_backend
    echo
    
    setup_frontend
    echo
    
    setup_database
    echo
    
    install_dev_tools
    echo
    
    create_dev_scripts
    echo
    
    print_success "🎉 Development setup completed!"
    echo
    print_status "To start developing:"
    print_status "  • Run './dev-full.sh' to start both backend and frontend"
    print_status "  • Run './dev-backend.sh' to start only the backend"
    print_status "  • Run './dev-frontend.sh' to start only the frontend"
    echo
    print_status "URLs:"
    print_status "  • Frontend: http://localhost:5173"
    print_status "  • Backend API: http://localhost:3000"
    print_status "  • API Documentation: http://localhost:3000/scalar"
    print_status "  • Admin Panel: http://localhost:5173/admin/login"
    echo
    print_warning "Don't forget to review your .env file settings!"
}

# Run main function
main "$@"