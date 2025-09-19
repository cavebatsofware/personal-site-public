# Build stage
FROM rust:1.89-alpine3.20 AS builder

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
FROM alpine:3.20

WORKDIR /app

# Install minimal runtime dependencies
RUN apk add --no-cache \
    ca-certificates \
    libssl3

# Copy the binary from builder stage
COPY --from=builder /app/target/release/personal-site ./personal-site

# Build arguments for configurable asset paths
ARG HTML_SRC_DIR="."
ARG PDF_SRC_DIR="."
ARG ASSETS_SRC_DIR="assets"

# Copy static assets - paths can be overridden during build
# Example: docker build --build-arg HTML_SRC_DIR="custom" .
# It is recommended to use a different repository and copy them over during deployment
# so that repo can remain private and this one can be shared publicly.
COPY ${HTML_SRC_DIR}/index.html ${HTML_SRC_DIR}/landing.html ./
COPY ${PDF_SRC_DIR}/*.pdf ./
COPY ${ASSETS_SRC_DIR} ./assets

# Create non-root user (Alpine style)
RUN adduser -D -s /bin/false appuser && \
    chown -R appuser:appuser /app

USER appuser

EXPOSE 3000

# Start the application
CMD ["./personal-site"]