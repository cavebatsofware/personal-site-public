# Personal Site Deployment Makefile
# Follows the deployment instructions from README.md

# Load environment variables from .env file if it exists
ifneq (,$(wildcard .env))
include .env
export
endif

# Configuration - Uses values from .env or environment variables
DOCKER_IMAGE := personal-site
ECR_REGISTRY ?= $(if $(ECR_REGISTRY_URL),$(ECR_REGISTRY_URL),$(error ECR_REGISTRY_URL not found. Create .env file or set environment variable))
ECR_REPOSITORY ?= $(if $(ECR_REPO_NAME),$(ECR_REPO_NAME),$(error ECR_REPO_NAME not found. Create .env file or set environment variable))
ECR_REGION ?= us-east-2
ECR_IMAGE := $(ECR_REGISTRY)/$(ECR_REPOSITORY):latest

# Default target
.PHONY: help
help:
	@echo "Personal Site - Development & Deployment Commands"
	@echo ""
	@echo "🗄️  Database Commands:"
	@echo "  make db-up          - Start PostgreSQL database"
	@echo "  make db-down        - Stop PostgreSQL database"
	@echo "  make db-logs        - View database logs"
	@echo "  make db-shell       - Open PostgreSQL shell"
	@echo "  make db-migrate     - Run database migrations"
	@echo "  make db-reset       - Reset database (WARNING: deletes all data)"
	@echo "  make db-backup      - Backup database to ./backups/"
	@echo "  make db-restore     - Restore database from backup"
	@echo ""
	@echo "🛠️  Development Commands:"
	@echo "  make dev            - Start database and run app locally"
	@echo "  make dev-logs       - Tail application and database logs"
	@echo "  make clippy         - Run clippy linter"
	@echo ""
	@echo "🐳 Docker Commands:"
	@echo "  make build          - Build Docker image locally"
	@echo "  make run            - Run container locally (requires ACCESS_CODES env var)"
	@echo "  make deploy         - Complete deployment: build + push to ECR"
	@echo "  make push-ecr       - Push to ECR (after build)"
	@echo "  make login-ecr      - Login to ECR"
	@echo "  make clean          - Remove local Docker images"
	@echo ""
	@echo "📋 Configuration:"
	@echo "  make show-config    - Display current configuration"
	@echo "  make check-prereqs  - Check for required tools"
	@echo ""
	@echo "Quick start:"
	@echo "  cp .env.example .env"
	@echo "  # Edit .env with your values"
	@echo "  make db-up          # Start database"
	@echo "  make db-migrate     # Run migrations"
	@echo "  make dev            # Start development server"

# Build the Docker image
.PHONY: build
build: admin-build
	@echo "🔨 Building Docker image..."
	docker build \
		--build-arg SITE_DOMAIN=$(SITE_DOMAIN) \
		-t $(DOCKER_IMAGE) .
	@echo "✅ Build complete: $(DOCKER_IMAGE)"

# Login to ECR
.PHONY: login-ecr
login-ecr:
	@echo "🔐 Logging into ECR..."
	aws ecr get-login-password --region $(ECR_REGION) | docker login --username AWS --password-stdin $(ECR_REGISTRY)
	@echo "✅ ECR login successful"

# Tag and push to ECR
.PHONY: push-ecr
push-ecr: login-ecr
	@echo "🏷️  Tagging image for ECR..."
	docker tag $(DOCKER_IMAGE):latest $(ECR_IMAGE)
	@echo "📤 Pushing to ECR..."
	docker push $(ECR_IMAGE)
	@echo "✅ Push complete: $(ECR_IMAGE)"

# Complete deployment (build + push)
.PHONY: deploy
deploy: build push-ecr
	@echo ""
	@echo "🚀 Deployment complete!"
	@echo "📋 Image pushed to: $(ECR_IMAGE)"
	@echo ""
	@echo "Next steps:"
	@echo "1. The image is now available in ECR"
	@echo "2. The vpn-server docker-compose will pull this image automatically"
	@echo "3. Deploy infrastructure changes if needed via vpn-server project"

# Run locally for testing
.PHONY: run
run:
ifndef ACCESS_CODES
	$(error ACCESS_CODES environment variable is required. Example: make run ACCESS_CODES="test123,demo456")
endif
	@echo "🏃 Running container locally..."
	docker run -p 3000:3000 -e ACCESS_CODES="$(ACCESS_CODES)" $(DOCKER_IMAGE)

# Clean up local images
.PHONY: clean
clean:
	@echo "🧹 Cleaning up local Docker images..."
	-docker rmi $(DOCKER_IMAGE):latest
	-docker rmi $(ECR_IMAGE)
	@echo "✅ Cleanup complete"

# Check prerequisites
.PHONY: check-prereqs
check-prereqs:
	@echo "🔍 Checking prerequisites..."
	@command -v docker >/dev/null 2>&1 || { echo "❌ Docker is required but not installed"; exit 1; }
	@command -v aws >/dev/null 2>&1 || { echo "❌ AWS CLI is required but not installed"; exit 1; }
	@aws sts get-caller-identity >/dev/null 2>&1 || { echo "❌ AWS CLI not configured or no permissions"; exit 1; }
	@echo "✅ All prerequisites met"

# Show current configuration
.PHONY: show-config
show-config:
	@echo "📋 Current Configuration:"
	@echo "  Docker Image: $(DOCKER_IMAGE)"
	@echo "  ECR Registry: $(if $(ECR_REGISTRY_URL),$(ECR_REGISTRY_URL),❌ Not set)"
	@echo "  ECR Repository: $(if $(ECR_REPO_NAME),$(ECR_REPO_NAME),❌ Not set)"
	@echo "  ECR Region: $(ECR_REGION)"
	@echo "  Full ECR Image: $(if $(ECR_REGISTRY_URL),$(if $(ECR_REPO_NAME),$(ECR_IMAGE),❌ Missing repo name),❌ Missing registry)"
	@echo "  Database URL: $(if $(DATABASE_URL),✅ Set,❌ Not set)"
	@echo "  Access Codes: $(if $(ACCESS_CODES),✅ Set,❌ Not set)"

#
# Database Management Commands
#

# Start PostgreSQL database
.PHONY: db-up
db-up:
	@echo "🚀 Starting PostgreSQL database..."
	docker-compose up -d postgres
	@echo "⏳ Waiting for database to be ready..."
	@sleep 5
	@docker-compose exec postgres pg_isready -U $${POSTGRES_USER:-personal_site_user} || echo "Waiting..."
	@echo "✅ Database is ready!"
	@echo "📍 Connection: postgresql://$${POSTGRES_USER:-personal_site_user}:****@localhost:$${POSTGRES_PORT:-5432}/$${POSTGRES_DB:-personal_site}"

# Stop PostgreSQL database
.PHONY: db-down
db-down:
	@echo "🛑 Stopping PostgreSQL database..."
	docker-compose down
	@echo "✅ Database stopped"

# View database logs
.PHONY: db-logs
db-logs:
	docker-compose logs -f postgres

# Open PostgreSQL shell
.PHONY: db-shell
db-shell:
	@echo "🐘 Opening PostgreSQL shell..."
	docker-compose exec postgres psql -U $${POSTGRES_USER:-personal_site_user} -d $${POSTGRES_DB:-personal_site}

# Run database migrations
.PHONY: db-migrate
db-migrate:
	@echo "🔄 Running database migrations..."
	MIGRATE_DB=true cargo run -- migrate
	@echo "✅ Migrations complete!"

# Reset database (WARNING: deletes all data)
.PHONY: db-reset
db-reset:
	@echo "⚠️  WARNING: This will delete all data in the database!"
	@read -p "Are you sure? Type 'yes' to continue: " confirm; \
	if [ "$$confirm" = "yes" ]; then \
		echo "🗑️  Resetting database..."; \
		docker-compose down -v; \
		docker-compose up -d postgres; \
		sleep 5; \
		MIGRATE_DB=true cargo run -- migrate; \
		echo "✅ Database reset complete!"; \
	else \
		echo "❌ Reset cancelled"; \
	fi

# Backup database
.PHONY: db-backup
db-backup:
	@echo "💾 Creating database backup..."
	@mkdir -p backups
	@BACKUP_FILE="backups/personal_site_$$(date +%Y%m%d_%H%M%S).sql"; \
	docker-compose exec -T postgres pg_dump -U $${POSTGRES_USER:-personal_site_user} $${POSTGRES_DB:-personal_site} > $$BACKUP_FILE; \
	echo "✅ Backup created: $$BACKUP_FILE"

# Restore database from backup
.PHONY: db-restore
db-restore:
	@echo "📂 Available backups:"
	@ls -lh backups/*.sql 2>/dev/null || echo "No backups found"
	@read -p "Enter backup filename (e.g., backups/personal_site_20250119_120000.sql): " backup; \
	if [ -f "$$backup" ]; then \
		echo "♻️  Restoring from $$backup..."; \
		docker-compose exec -T postgres psql -U $${POSTGRES_USER:-personal_site_user} $${POSTGRES_DB:-personal_site} < $$backup; \
		echo "✅ Restore complete!"; \
	else \
		echo "❌ Backup file not found: $$backup"; \
	fi

#
# Development Commands
#

# Start development environment
.PHONY: dev
dev: db-up admin-build
	@echo "🔧 Starting development server..."
	@echo "📝 Logs will appear below. Press Ctrl+C to stop."
	@echo ""
	cargo run

# Tail development logs
.PHONY: dev-logs
dev-logs:
	@echo "📋 Tailing logs (Ctrl+C to exit)..."
	docker-compose logs -f postgres

# Run clippy
.PHONY: clippy
clippy:
	@echo "📎 Running clippy..."
	cargo clippy -- -D warnings

# Full development setup
.PHONY: setup
setup:
	@echo "🚀 Setting up development environment..."
	@if [ ! -f .env ]; then \
		echo "📝 Creating .env from .env.example..."; \
		cp .env.example .env; \
		echo "⚠️  Please edit .env with your configuration"; \
	else \
		echo "✅ .env file already exists"; \
	fi
	@echo "📦 Installing admin frontend dependencies..."
	npm install
	@echo "🔄 Running migrations..."
	@$(MAKE) db-migrate
	@echo ""
	@echo "✅ Setup complete! Run 'make dev' to start the server"

#
# Admin Frontend Commands
#

# Build admin frontend for production
.PHONY: admin-build
admin-build:
	@echo "🔨 Building admin frontend..."
	@if [ ! -d "node_modules" ]; then \
		echo "📦 Installing dependencies first..."; \
		npm install; \
	fi
	npm run build
	@echo "✅ Admin frontend built to assets/admin/"
