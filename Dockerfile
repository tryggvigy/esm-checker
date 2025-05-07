# Build stage
FROM rust:1.75-slim-bullseye as builder

WORKDIR /usr/src/app

# Copy the entire workspace
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates

# Install Node.js and npm in a single layer with minimal dependencies
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    curl \
    ca-certificates \
    gnupg && \
    curl -fsSL https://deb.nodesource.com/setup_20.x | bash - && \
    apt-get install -y --no-install-recommends nodejs && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/* && \
    npm install -g npm@latest

# Build the application from the workspace root
RUN cargo build --release -p web_server

# Runtime stage
FROM debian:bullseye-slim

WORKDIR /usr/local/bin

# Install Node.js and npm in runtime image with minimal dependencies
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    curl \
    ca-certificates \
    gnupg && \
    curl -fsSL https://deb.nodesource.com/setup_20.x | bash - && \
    apt-get install -y --no-install-recommends nodejs && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/* && \
    npm install -g npm@latest && \
    mkdir -p /var/cache/npm && \
    chmod 777 /var/cache/npm

# Copy the binary from builder
COPY --from=builder /usr/src/app/target/release/web_server .

# Copy static files
COPY --from=builder /usr/src/app/crates/web_server/static ./static

# Set environment variables
ENV PORT=3000 \
    RUST_LOG=info \
    NPM_CACHE_DIR=/var/cache/npm

# Create a volume for npm cache
VOLUME ["/var/cache/npm"]

# Expose the port
EXPOSE 3000

# Run the application
CMD ["./web_server"]