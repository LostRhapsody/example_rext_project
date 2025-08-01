# Rext Demo Project - Deployment Guide

This guide covers both **Docker deployment** and **bare metal deployment** options for the Rext Demo Project.

## 🐳 Docker Deployment (Recommended)

Docker deployment provides the easiest and most reliable deployment method.

### Prerequisites

- Docker 20.10+ and Docker Compose 2.0+
- 2GB+ RAM available
- Port 3000 available

### Quick Start with Docker

1. **Clone and configure:**
   ```bash
   git clone <repository-url>
   cd example_rext_project
   cp example.env .env
   # Edit .env with your production settings
   ```

2. **Deploy:**
   ```bash
   docker-compose up -d
   ```

3. **Access the application:**
   - Application: http://localhost:3000
   - Admin Panel: http://localhost:3000/admin/login
   - API Documentation: http://localhost:3000/scalar

### Docker Environment Configuration

Create or modify your `.env` file with production settings:

```env
# Production Environment
ENVIRONMENT=production

# Database
DATABASE_URL=sqlite:/app/data/sqlite.db

# Security (CHANGE THESE IN PRODUCTION!)
JWT_SECRET=your-super-secure-jwt-secret-key-here
ADMIN_EMAIL=admin@yourdomain.com
ADMIN_PASSWORD=your-secure-admin-password

# Server Configuration
RUST_LOG=info
ALLOWED_ORIGIN=https://yourdomain.com

# Role Configuration
CREATE_ADMIN_USER=true
CREATE_DEFAULT_ROLES=true
DEFAULT_ROLES=admin,user
```

### Docker Commands

```bash
# Start services
docker-compose up -d

# View logs
docker-compose logs -f

# Stop services
docker-compose down

# Rebuild and restart
docker-compose up -d --build

# Development mode with hot reload
docker-compose --profile dev up -d
```

### Docker Production Considerations

1. **Database Persistence**: Data is stored in Docker volume `rext_data`
2. **SSL/TLS**: Add reverse proxy (nginx/traefik) for HTTPS
3. **Monitoring**: Check logs with `docker-compose logs`
4. **Backups**: Backup the `rext_data` volume regularly

---

## 🖥️ Bare Metal Deployment

Deploy directly on your server without Docker.

### Prerequisites

- **Rust** 1.70+ - [Install from rustup.rs](https://rustup.rs/)
- **Node.js** 18+ - [Install from nodejs.org](https://nodejs.org/)
- **SQLite** 3.35+ (usually pre-installed)
- **Git**

### Automated Setup

Use our setup scripts for quick deployment:

1. **Development setup:**
   ```bash
   chmod +x scripts/dev-setup.sh
   ./scripts/dev-setup.sh
   ```

2. **Production build:**
   ```bash
   chmod +x scripts/build-production.sh
   ./scripts/build-production.sh
   ```

### Manual Deployment Steps

#### 1. Environment Setup

```bash
# Clone repository
git clone <repository-url>
cd example_rext_project

# Copy environment configuration
cp example.env .env
# Edit .env with your settings
```

#### 2. Build Frontend

```bash
cd frontend
npm install
npm run build
cd ..

# Copy frontend assets
rm -rf dist
cp -r frontend/dist dist
```

#### 3. Build Backend

```bash
# Set production environment
export ENVIRONMENT=production
export BUILD_FRONTEND=false

# Build with optimizations
cargo build --release
```

#### 4. Setup Database

```bash
cd migration
cargo build --release
cd ..

# Run migrations
./target/release/migration
```

#### 5. Deploy

```bash
# Start the server
export ENVIRONMENT=production
./target/release/project_rext_1
```

### Production Deployment Package

The build script creates a `rext-deployment` package with everything needed:

```
rext-deployment/
├── rext-server          # Main application binary
├── migration           # Database migration tool
├── dist/               # Frontend assets
├── start.sh            # Startup script
├── .env.example        # Environment template
└── README.md          # Quick deployment guide
```

**To deploy the package:**

1. Copy the `rext-deployment` directory to your server
2. Create `.env` file from `.env.example`
3. Run `./start.sh`

### Service Configuration

#### Systemd Service (Linux)

Create `/etc/systemd/system/rext.service`:

```ini
[Unit]
Description=Rext Demo Server
After=network.target

[Service]
Type=simple
User=www-data
WorkingDirectory=/opt/rext-deployment
Environment=ENVIRONMENT=production
EnvironmentFile=/opt/rext-deployment/.env
ExecStart=/opt/rext-deployment/rext-server
Restart=on-failure
RestartSec=5

[Install]
WantedBy=multi-user.target
```

Enable and start:
```bash
sudo systemctl enable rext
sudo systemctl start rext
sudo systemctl status rext
```

#### Process Manager (PM2)

```bash
# Install PM2
npm install -g pm2

# Start with PM2
pm2 start rext-server --name rext-demo

# Save PM2 configuration
pm2 save
pm2 startup
```

---

## 🔧 Configuration

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `ENVIRONMENT` | development | Environment mode (development/production) |
| `DATABASE_URL` | sqlite:sqlite.db | Database connection URL |
| `JWT_SECRET` | ⚠️ **Required** | JWT signing secret |
| `RUST_LOG` | info | Log level (error/warn/info/debug/trace) |
| `ADMIN_EMAIL` | admin@localhost.com | Default admin email |
| `ADMIN_PASSWORD` | admin123 | Default admin password |
| `CREATE_ADMIN_USER` | true | Create admin user on startup |
| `CREATE_DEFAULT_ROLES` | true | Create default roles |
| `DEFAULT_ROLES` | admin,user | Comma-separated list of roles to create |
| `ALLOWED_ORIGIN` | http://localhost:5173 | CORS allowed origin for production |

### Security Considerations

1. **Change Default Passwords**: Update `ADMIN_PASSWORD` before deployment
2. **JWT Secret**: Use a strong, random `JWT_SECRET` in production
3. **HTTPS**: Use a reverse proxy for SSL/TLS in production
4. **Firewall**: Restrict access to port 3000
5. **Database**: Secure SQLite file permissions (600)

### Performance Tuning

#### Backend
```env
# Optimize for production
RUST_LOG=warn
ENVIRONMENT=production
```

#### Frontend
The frontend is pre-built and optimized with:
- Code splitting
- Asset compression
- Cache-friendly headers

### Monitoring

#### Health Check
```bash
curl -f http://localhost:3000/ || echo "Server down"
```

#### Logs
```bash
# With systemd
journalctl -u rext -f

# With PM2
pm2 logs rext-demo

# With Docker
docker-compose logs -f
```

---

## 🚀 Reverse Proxy Setup

### Nginx Configuration

```nginx
server {
    listen 80;
    server_name yourdomain.com;
    
    location / {
        proxy_pass http://localhost:3000;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
    
    # WebSocket support for admin panel
    location /api/v1/admin/ws {
        proxy_pass http://localhost:3000;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host $host;
    }
}
```

### Traefik Configuration

```yaml
version: '3.8'
services:
  rext-app:
    # ... your existing service config
    labels:
      - "traefik.enable=true"
      - "traefik.http.routers.rext.rule=Host(`yourdomain.com`)"
      - "traefik.http.routers.rext.tls.certresolver=letsencrypt"
      - "traefik.http.services.rext.loadbalancer.server.port=3000"
```

---

## 📊 Scaling Considerations

### Horizontal Scaling
- Use a load balancer (nginx, HAProxy)
- Share SQLite database file via NFS, or migrate to PostgreSQL
- Configure session storage (Redis)

### Database Migration
For production workloads, consider migrating from SQLite to PostgreSQL:

1. Update `Cargo.toml` dependencies
2. Modify `DATABASE_URL` to PostgreSQL connection string
3. Update Sea-ORM configuration
4. Run migrations with new database

### Container Orchestration
For Kubernetes deployment:
- Create ConfigMaps for environment variables
- Use Secrets for sensitive data
- Configure persistent volumes for database
- Set up ingress controllers

---

## 🐛 Troubleshooting

### Common Issues

#### Port Already in Use
```bash
# Find process using port 3000
lsof -ti:3000
# Kill the process
kill -9 $(lsof -ti:3000)
```

#### Database Locked
```bash
# Check for running processes
ps aux | grep rext
# Stop all processes and restart
```

#### Frontend Assets Not Loading
```bash
# Verify dist directory exists
ls -la dist/
# Rebuild frontend
cd frontend && npm run build
```

#### Permission Denied
```bash
# Fix binary permissions
chmod +x target/release/project_rext_1
# Fix database permissions
chmod 600 sqlite.db
```

### Debug Mode

Enable debug logging:
```env
RUST_LOG=debug
ENVIRONMENT=development
```

### Getting Help

1. Check logs for error messages
2. Verify all environment variables are set
3. Ensure all prerequisites are installed
4. Check firewall settings
5. Verify port availability

---

## 📝 Maintenance

### Regular Tasks

1. **Log Rotation**: Configure log rotation to prevent disk space issues
2. **Database Backup**: Regular SQLite file backups
3. **Security Updates**: Keep dependencies updated
4. **Monitoring**: Set up health checks and alerting

### Updates

```bash
# Pull latest changes
git pull origin main

# Rebuild and restart
docker-compose up -d --build

# Or for bare metal
./scripts/build-production.sh
sudo systemctl restart rext
```

### Backup Strategy

```bash
# Database backup
cp sqlite.db sqlite.db.backup.$(date +%Y%m%d)

# Full deployment backup
tar -czf rext-backup-$(date +%Y%m%d).tar.gz rext-deployment/
```