# Multi-stage build for optimized production deployment
# Stage 1: Build environment with Zig
FROM ubuntu:22.04 AS zig-builder

# Install Zig build dependencies
RUN apt-get update && apt-get install -y \
    wget \
    xz-utils \
    && rm -rf /var/lib/apt/lists/*

# Install Zig 0.14
RUN wget https://ziglang.org/builds/zig-linux-x86_64-0.14.0-dev.1911+3bf89f55c.tar.xz \
    && tar -xf zig-linux-x86_64-0.14.0-dev.1911+3bf89f55c.tar.xz \
    && mv zig-linux-x86_64-0.14.0-dev.1911+3bf89f55c /opt/zig

ENV PATH="/opt/zig:${PATH}"

# Copy Zig source code
WORKDIR /build/zig
COPY zig/ .

# Build Zig components
RUN zig build -Doptimize=ReleaseFast

# Stage 2: Rust build environment
FROM rust:1.75-slim AS rust-builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libpq-dev \
    libsqlite3-dev \
    protobuf-compiler \
    build-essential \
    && rm -rf /var/lib/apt/lists/*

# Create build directory
WORKDIR /build

# Copy Rust source code
COPY rust/ rust/

# Copy Zig build artifacts
COPY --from=zig-builder /build/zig/zig-out/ rust/zig-out/

# Build Rust application with optimizations
WORKDIR /build/rust
RUN cargo build --release --locked

# Stage 3: Production runtime image
FROM ubuntu:22.04

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    libpq5 \
    libsqlite3-0 \
    curl \
    postgresql-client \
    redis-tools \
    && rm -rf /var/lib/apt/lists/* \
    && apt-get clean

# Create non-root user
RUN groupadd -r opensim && useradd -r -g opensim -u 1000 opensim

# Create application directories
RUN mkdir -p /app /config /assets /logs /tmp/opensim \
    && chown -R opensim:opensim /app /config /assets /logs /tmp/opensim

# Copy application binary
COPY --from=rust-builder /build/rust/target/release/opensim-next /app/opensim-next
COPY --from=rust-builder /build/rust/zig-out/ /app/zig-out/

# Copy configuration templates
COPY deployment/config/ /config/

# Create version information
RUN echo "$(date -u +%Y%m%d-%H%M%S)" > /app/VERSION

# Set permissions
RUN chmod +x /app/opensim-next \
    && chown -R opensim:opensim /app

# Switch to non-root user
USER opensim

# Set working directory
WORKDIR /app

# Environment variables
ENV RUST_LOG=info
ENV RUST_BACKTRACE=1
ENV OPENSIM_CONFIG_DIR=/config
ENV OPENSIM_ASSETS_DIR=/assets
ENV OPENSIM_LOGS_DIR=/logs

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=30s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# Expose ports
EXPOSE 9000 9001 8080 8090 9100

# Entry point
ENTRYPOINT ["/app/opensim-next"]
CMD ["server", "--config", "/config/server.yaml"]

# Build metadata
LABEL org.opencontainers.image.title="OpenSim Next"
LABEL org.opencontainers.image.description="High-performance virtual world server with Rust/Zig hybrid architecture"
LABEL org.opencontainers.image.version="1.0.0"
LABEL org.opencontainers.image.vendor="OpenSim Next Project"
LABEL org.opencontainers.image.licenses="BSD-3-Clause"
LABEL org.opencontainers.image.source="https://github.com/opensim/opensim-next"