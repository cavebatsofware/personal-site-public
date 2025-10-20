# Frontend build stage
FROM node:22-alpine AS frontend-builder

WORKDIR /app

# Copy package files and install dependencies
COPY package*.json ./
RUN npm ci

# Copy vite config and frontend source
COPY vite.config.js ./
COPY admin-frontend ./admin-frontend

# Build frontend (requires SITE_DOMAIN to be set)
ARG SITE_DOMAIN=example.com
ENV SITE_DOMAIN=${SITE_DOMAIN}
RUN npm run build

# Rust build stage
FROM rust:alpine3.22 AS builder

WORKDIR /app

# Install build dependencies for Alpine/musl compatibility
RUN apk add --no-cache \
    musl-dev \
    openssl-dev \
    openssl-libs-static \
    pkgconfig

# Copy manifest files
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src ./src

# Build with vendored OpenSSL for static musl compatibility
ENV OPENSSL_STATIC=1
RUN cargo build --release

# Runtime stage
FROM alpine:3.22.1

WORKDIR /app

# Install minimal runtime dependencies
RUN apk add --no-cache \
    ca-certificates \
    libssl3

# Copy the binary from builder stage
COPY --from=builder /app/target/release/personal-site ./personal-site

# Copy built frontend from frontend-builder stage
COPY --from=frontend-builder /app/admin-assets ./admin-assets

# Copy static assets
COPY assets ./assets
COPY landing.html ./landing.html
COPY entrypoint.sh ./entrypoint.sh

# Create non-root user (Alpine style)
RUN adduser -D -s /bin/false appuser && \
    chown -R appuser:appuser /app && \
    chmod +x ./entrypoint.sh

USER appuser

EXPOSE 3000

# Start the application
ENTRYPOINT ["./entrypoint.sh"]
