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

# Optional build arguments for custom asset source directories
HTML_SRC_DIR ?= .
PDF_SRC_DIR ?= .
ASSETS_SRC_DIR ?= assets

# Default target
.PHONY: help
help:
	@echo "Personal Site Deployment Commands:"
	@echo ""
	@echo "  make build          - Build Docker image locally"
	@echo "  make run            - Run container locally (requires ACCESS_CODES env var)"
	@echo "  make deploy         - Complete deployment: build + push to ECR"
	@echo "  make push-ecr       - Push to ECR (after build)"
	@echo "  make login-ecr      - Login to ECR"
	@echo "  make test-local     - Test local build with sample codes"
	@echo "  make clean          - Remove local Docker images"
	@echo ""
	@echo "Environment Configuration:"
	@echo "  The Makefile automatically loads variables from .env file"
	@echo "  Copy .env.example to .env and configure your values"
	@echo ""
	@echo "Required in .env file:"
	@echo "  ECR_REGISTRY_URL    - Your ECR registry URL"
	@echo "  ECR_REPO_NAME       - Your ECR repository name"
	@echo "  ACCESS_CODES        - Required for local testing"
	@echo ""
	@echo "Optional build customization:"
	@echo "  HTML_SRC_DIR        - Source directory for HTML files (default: .)"
	@echo "  PDF_SRC_DIR         - Source directory for PDF files (default: .)"
	@echo "  ASSETS_SRC_DIR      - Source directory for assets (default: assets)"
	@echo ""
	@echo "Quick start:"
	@echo "  cp .env.example .env"
	@echo "  # Edit .env with your values"
	@echo "  make deploy"

# Build the Docker image
.PHONY: build
build:
	@echo "🔨 Building Docker image..."
	docker build \
		--build-arg HTML_SRC_DIR="$(HTML_SRC_DIR)" \
		--build-arg PDF_SRC_DIR="$(PDF_SRC_DIR)" \
		--build-arg ASSETS_SRC_DIR="$(ASSETS_SRC_DIR)" \
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

# Test with sample codes
.PHONY: test-local
test-local: build
	@echo "🧪 Testing container with sample codes..."
	@echo "Access URL: http://localhost:3000/resume/test123"
	docker run -p 3000:3000 -e ACCESS_CODES="test123,demo456" $(DOCKER_IMAGE)

# Run from ECR image
.PHONY: run-ecr
run-ecr:
ifndef ACCESS_CODES
	$(error ACCESS_CODES environment variable is required. Example: make run-ecr ACCESS_CODES="test123,demo456")
endif
	@echo "🏃 Running ECR image locally..."
	docker run -p 3000:3000 -e ACCESS_CODES="$(ACCESS_CODES)" $(ECR_IMAGE)

# Clean up local images
.PHONY: clean
clean:
	@echo "🧹 Cleaning up local Docker images..."
	-docker rmi $(DOCKER_IMAGE):latest
	-docker rmi $(ECR_IMAGE)
	@echo "✅ Cleanup complete"

# Show current image sizes
.PHONY: images
images:
	@echo "📊 Local Docker images:"
	docker images | grep -E "($(DOCKER_IMAGE)|$(shell basename $(ECR_REPOSITORY)))" || echo "No local images found"

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
	@echo ""
	@echo "📁 Asset Source Configuration:"
	@echo "  HTML Source Directory: $(HTML_SRC_DIR)"
	@echo "  PDF Source Directory: $(PDF_SRC_DIR)"
	@echo "  Assets Source Directory: $(ASSETS_SRC_DIR)"