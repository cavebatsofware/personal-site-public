# Personal Site

A guarded personal site built with Rust and Axum that serves a document only to users with valid access codes.

## Features

- **Code-gated access**: Documents only accessible with valid codes
- **PostgreSQL database**: Persistent storage with SeaORM
- **Access tracking**: Monitor and analyze who accesses your site with full visibility
- **Security features**:
  - Rate limiting (configurable per minute/hour)
  - Full access logs with IP addresses and access codes for friend tracking
- **Static file serving**: CSS and other assets served securely
- **Deploy-ready**: Connection retry logic, health checks, and migrations

## Quick Start

### Prerequisites

- Rust (latest stable version)
- Docker and Docker Compose
- PostgreSQL client tools (optional, for database management)

### Development Setup

The easiest way to get started is using the included Makefile:

```bash
# 1. Setup environment and database
make setup

# 2. Start development server (automatically starts database)
make dev
```

The application will be available at `http://localhost:3000`.

### Manual Setup

If you prefer to set things up manually:

```bash
# 1. Create environment configuration
cp .env.example .env
# Edit .env with your database and AWS settings

# 2. Start PostgreSQL database
make db-up

# 3. Run database migrations
make db-migrate

# 4. Start the application
cargo run

# 5. Access admin panel to create access codes
# http://localhost:3000/admin
```

## Database Management

### Local Development

The project uses Docker Compose for local PostgreSQL development:

```bash
# Start database
make db-up

# Stop database
make db-down

# View database logs
make db-logs

# Open PostgreSQL shell
make db-shell

# Run migrations
make db-migrate

# Backup database
make db-backup

# Restore from backup
make db-restore

# Reset database (WARNING: deletes all data)
make db-reset
```

Visit `http://localhost:5050` and login with credentials from `.env` file.

### Production Database

For production, update your `.env` file with your hosted PostgreSQL database:

```bash
# Production database URL
DATABASE_URL=postgresql://user:password@your-db-host:5432/personal_site
```

The application includes automatic retry logic for deployment:
- connection attempts with exponential backoff
- Health checks and connection verification
- Graceful error handling

## Development Commands

The project includes a comprehensive Makefile for development tasks:

```bash
# Code quality
make clippy         # Run clippy linter

# Development workflow
make dev            # Start database and run app locally
make dev-logs       # Tail application and database logs

# View all available commands
make help
```

## Usage

### Accessing the Site

Visit: `http://localhost:3000/access/{your-code}`

For example, with a configured access code:
- `http://localhost:3000/access/your-secret-code`

### Endpoints

- `/` - Landing page
- `/access/{code}` - Site page (code-gated)
- `/access/{code}/download` - Download PDF resume
- `/health` - Health check endpoint
- `/assets/*` - Static assets (CSS, icons, etc.)

### Invalid Codes

Attempting to access with an invalid code will return a 404 error.

## File Structure

```
personal-site/
├── Cargo.toml              # Rust dependencies
├── Dockerfile              # Alpine-based container build
├── docker-compose.yml      # PostgreSQL development environment
├── Makefile                # Development and deployment automation
├── .env.example            # Environment configuration template
├── src/
│   ├── main.rs             # Application entry point
│   ├── app.rs              # Application state management
│   ├── database.rs         # Database connection and management
│   ├── security.rs         # Security service (rate limiting, hashing)
│   ├── errors.rs           # Custom error types
│   ├── lib.rs              # Library exports
│   ├── entities/           # SeaORM database entities
│   │   ├── mod.rs
│   │   └── access_log.rs   # Access log entity
│   ├── middleware/         # Axum middleware
│   │   ├── mod.rs
│   │   ├── security.rs     # Security context extraction
│   │   ├── rate_limit.rs   # Rate limiting middleware
│   │   └── access_log.rs   # Access logging middleware
│   ├── migration/          # Database migrations
│   │   ├── mod.rs
│   │   └── m20250119_000001_create_access_log.rs
│   └── tests/              # Test suites
│       ├── mod.rs
│       ├── unit_tests.rs
│       ├── security_tests.rs
│       ├── middleware_tests.rs
│       └── database_tests.rs
├── scripts/                # Utility scripts
│   ├── init-db.sh          # Database initialization
│   └── wait-for-db.sh      # Database readiness check
├── landing.html            # Landing page
├── assets/                 # Static assets (CSS, icons, etc.)
└── README.md               # This file
```

## Security

- **Code-gated access**: Server-side validation of access codes
- **Rate limiting**: Configurable requests per minute/hour to prevent abuse
- **Abuse protection**: Automatic IP blocking for suspicious activity
- **Access logging**: All attempts logged to database with full IP and code visibility
- **No data exposure**: Invalid attempts return generic 404 errors

## Deployment

### Local Production Build

For local production deployment:

1. Configure environment variables (DATABASE_URL, SITE_DOMAIN, SITE_URL, AWS credentials)
2. Run database migrations
3. Build and run the application:

```bash
# Run migrations
MIGRATE_DB=true cargo run -- migrate

# Start application
cargo build --release
./target/release/personal-site
```

### Docker Deployment

#### Prerequisites

- Docker installed
- AWS CLI configured with appropriate permissions
- Access to your ECR repository

#### Quick Deployment with Makefile

The easiest way to deploy is using the included Makefile:

```bash
# 1. Set up environment configuration
cp .env.example .env
# Edit .env with your ECR registry URL, repository name, and resume codes

# 2. Deploy to ECR (builds, tags, and pushes automatically)
make deploy

# 3. Test locally first (optional)
make test-local
```

#### Manual Docker Commands

If you prefer manual commands:

```bash
# Build the Docker image
docker build -t personal-site .

# Login to ECR (replace with your registry URL)
aws ecr get-login-password --region us-east-2 | docker login --username AWS --password-stdin YOUR_ACCOUNT.dkr.ecr.us-east-2.amazonaws.com

# Tag the image for ECR
docker tag personal-site:latest YOUR_ACCOUNT.dkr.ecr.us-east-2.amazonaws.com/YOUR_REPO:latest

# Push to ECR
docker push YOUR_ACCOUNT.dkr.ecr.us-east-2.amazonaws.com/YOUR_REPO:latest
```

#### Available Makefile Commands

```bash
make help           # Show all available commands
make build          # Build Docker image locally
make deploy         # Complete deployment (build + push to ECR)
make run            # Run locally (requires ACCESS_CODES environment variable)
make clean          # Remove local Docker images
make show-config    # Display current configuration
make check-prereqs  # Verify Docker and AWS CLI setup
```

#### Environment Configuration

Create a `.env` file with your configuration:

```bash
# Required: Your ECR registry URL
ECR_REGISTRY_URL=your-account.dkr.ecr.us-east-2.amazonaws.com

# Required: Your ECR repository name
ECR_REPO_NAME=your-personal-site-repo

# Optional: AWS region (default: us-east-2)
ECR_REGION=us-east-2

# Required for local testing: Comma-separated resume access codes
ACCESS_CODES=your-secret-code,another-code

# Optional: Port override (default: 3000)
PORT=3000
```

#### Docker Build Details

The Dockerfile uses an optimized Alpine Linux multi-stage build:
- **Build stage**: Uses `rust:1.89-alpine3.x` with musl for static compilation
- **Runtime stage**: Uses `alpine:3.x` for minimal production image (~27MB)
- **Security**:
  - Alpine Linux
  - Runs as non-root user `appuser`
  - Static OpenSSL linking for enhanced security
- **Port**: Exposes port 3000

## Customization

- **Resume content**: Edit `index.html` to update site information - uses s3 location, see source code for details
- **Styling**: Modify `styles.css` to change the appearance
- **Access codes**: Update the `ACCESS_CODES` environment variable
- **Server configuration**: Modify `src/main.rs` for additional features
