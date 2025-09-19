# Secure Document Server

A code-gated document server built with Rust and Axum that serves secure documents only to users with valid access codes.

## Features

- **🔐 Code-gated access**: Documents only accessible with valid access codes
- **🚀 High performance**: Built with Rust and Axum for speed and safety
- **🛡️ Secure by design**: Alpine Linux container with minimal attack surface
- **📦 Lightweight**: ~27MB container size with Alpine Linux
- **⚙️ Easy deployment**: Automated builds with Makefile and Docker

## Use Cases

- Personal portfolios and resumes
- Confidential reports and documents
- Private documentation
- Gated content delivery
- Secure file sharing

## Quick Start

### Prerequisites

- Rust (latest stable version)
- Docker (for containerized deployment)
- AWS CLI (for ECR deployment)

### Local Development

1. **Clone the repository**
   ```bash
   git clone <your-repo-url>
   cd personal-site-public
   ```

2. **Set up environment**
   ```bash
   cp .env.example .env
   # Edit .env with your access codes
   ```

3. **Run locally**
   ```bash
   cargo run
   ```

4. **Test access**
   - Visit: `http://localhost:3000/`
   - Access document: `http://localhost:3000/document/demo123`

### Docker Deployment

#### Quick Deployment with Makefile

```bash
# 1. Configure environment
cp .env.example .env
# Edit .env with your ECR details and access codes

# 2. Deploy to ECR
make deploy

# 3. Test locally (optional)
make test-local
```

#### Manual Docker Commands

```bash
# Build the image
docker build -t secure-document-server .

# Build with custom asset source directories
docker build \
  --build-arg HTML_SRC_DIR="custom" \
  --build-arg PDF_SRC_DIR="documents" \
  --build-arg ASSETS_SRC_DIR="static" \
  -t secure-document-server .

# Run locally
docker run -p 3000:3000 -e ACCESS_CODES="demo123,example456" secure-document-server

# For ECR deployment
aws ecr get-login-password --region us-east-2 | docker login --username AWS --password-stdin YOUR_ACCOUNT.dkr.ecr.us-east-2.amazonaws.com
docker tag secure-document-server:latest YOUR_ACCOUNT.dkr.ecr.us-east-2.amazonaws.com/YOUR_REPO:latest
docker push YOUR_ACCOUNT.dkr.ecr.us-east-2.amazonaws.com/YOUR_REPO:latest
```

## Configuration

### Environment Variables

Create a `.env` file with your configuration:

```bash
# Required: Your ECR registry URL (for deployment)
ECR_REGISTRY_URL=your-account.dkr.ecr.us-east-2.amazonaws.com

# Required: Your ECR repository name
ECR_REPO_NAME=your-document-server-repo

# Optional: AWS region (default: us-east-2)
ECR_REGION=us-east-2

# Required: Comma-separated document access codes
ACCESS_CODES=demo123,example456,test789

# Optional: Port override (default: 3000)
PORT=3000
```

### Access Codes

Access codes are configured via the `ACCESS_CODES` environment variable:

```bash
export ACCESS_CODES="secret1,secret2,secret3"
```

## Project Structure

```
secure-document-server/
├── Cargo.toml              # Rust dependencies
├── Dockerfile              # Alpine-based container build
├── Makefile                 # Deployment automation
├── .env.example             # Environment configuration template
├── src/
│   └── main.rs             # Server implementation
├── index.html              # Example secure document page
├── landing.html            # Landing page
├── example-document.pdf    # Example PDF document
├── assets/                 # Static assets
│   ├── styles.css          # Example stylesheet
│   └── icons/              # Application icons
└── README.md               # This file
```

## Available Makefile Commands

```bash
make help           # Show all available commands
make build          # Build Docker image locally
make deploy         # Complete deployment (build + push to ECR)
make test-local     # Test with example access codes
make run            # Run locally (requires ACCESS_CODES environment variable)
make clean          # Remove local Docker images
make show-config    # Display current configuration
make check-prereqs  # Verify Docker and AWS CLI setup
```

## API Endpoints

- `GET /` - Landing page
- `GET /document/{code}` - Secure document access (requires valid code)
- `GET /document/{code}/download` - Download document as PDF
- `GET /health` - Health check endpoint
- `GET /assets/*` - Static assets

## Docker Build Details

The Dockerfile uses an optimized Alpine Linux multi-stage build:

- **Build stage**: `rust:1.89-alpine3.20` with musl for static compilation
- **Runtime stage**: `alpine:3.20` for minimal production image (~27MB)
- **Security features**:
  - Alpine Linux with 0 CVEs vs Debian's 225+ vulnerabilities
  - Runs as non-root user `appuser`
  - Static OpenSSL linking for enhanced security
- **Performance**: 73% smaller than typical Debian-based images

## Security Features

- **Code-based access control**: Documents only accessible with valid codes
- **Server-side validation**: Access codes validated server-side only
- **Minimal attack surface**: Alpine Linux base with essential packages only
- **Non-root execution**: Container runs as unprivileged user
- **Static compilation**: Self-contained binary with no dynamic dependencies
- **Request logging**: Invalid access attempts are logged for monitoring

## Customization

### Configurable Asset Paths

The Dockerfile supports configurable paths for static assets, making it easy to use custom content:

```bash
# Environment variables or Makefile overrides
HTML_SRC_DIR="custom"
PDF_SRC_DIR="documents"
ASSETS_SRC_DIR="static"

# Build with custom source directories
make build HTML_SRC_DIR="$HTML_SRC_DIR" PDF_SRC_DIR="$PDF_SRC_DIR" ASSETS_SRC_DIR="$ASSETS_SRC_DIR"
```

**Use Cases:**
- **Private content repo**: Keep your actual content in a separate private repository
- **Multi-environment**: Different assets for dev/staging/prod
- **Custom structure**: Organize assets however you prefer

### Frontend Content

- **Document page**: Edit `index.html` to customize the secure document content
- **Landing page**: Modify `landing.html` to change the access page
- **Styling**: Update `assets/styles.css` for visual customization
- **Icons**: Replace files in `assets/icons/` with your application icons

### Backend Configuration

- **Access codes**: Update `ACCESS_CODES` environment variable
- **Server settings**: Modify `src/main.rs` for additional routes or functionality
- **Health checks**: Customize health check endpoint behavior

### Deployment

- **Container registry**: Update ECR settings in `.env` file
- **Infrastructure**: Use with your preferred container orchestration (ECS, Kubernetes, etc.)
- **SSL/TLS**: Deploy behind a load balancer or reverse proxy for HTTPS

## Development

### Running Tests

```bash
cargo test
```

### Local Development with Hot Reload

```bash
cargo install cargo-watch
cargo watch -x run
```

### Code Formatting

```bash
cargo fmt
```

### Linting

```bash
cargo clippy
```

## Production Deployment

### AWS ECS Example

1. Push image to ECR using `make deploy`
2. Create ECS task definition with your image
3. Set environment variables in task definition
4. Deploy behind Application Load Balancer for HTTPS

### Docker Compose Example

```yaml
version: '3.8'
services:
  document-server:
    image: your-account.dkr.ecr.us-east-2.amazonaws.com/your-repo:latest
    ports:
      - "3000:3000"
    environment:
      - ACCESS_CODES=your-production-codes
      - PORT=3000
    restart: unless-stopped
    healthcheck:
      test: ["CMD-SHELL", "curl -f http://localhost:3000/health || exit 1"]
      interval: 30s
      timeout: 10s
      retries: 3
```

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests if applicable
5. Submit a pull request

## License

This project is available under the MIT License. See the LICENSE file for more details.

## Support

For questions, issues, or feature requests, please open an issue in the GitHub repository.

---

**Built with ❤️ using Rust, Axum, and Alpine Linux**