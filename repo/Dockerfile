###############################
# 1. Build Stage
FROM rust:slim-bookworm as builder

# Install minimal build dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config libssl-dev && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/eagle

# Cache dependencies to speed up builds
COPY Cargo.toml Cargo.lock ./
RUN mkdir -p backend/src && echo "fn main() {}" > backend/src/main.rs && cargo build --release -p backend || true

# Copy the rest of the source code
COPY . .

# Build the backend binary
RUN cargo build --release -p backend

###############################
# 2. Test Runner Stage
FROM mcr.microsoft.com/playwright:v1.51.1-noble as test-runner

ENV DEBIAN_FRONTEND=noninteractive
ENV CARGO_HOME=/usr/local/cargo
ENV RUSTUP_HOME=/usr/local/rustup
ENV NODE_PATH=/opt/test-deps/node_modules
ENV PATH=/usr/local/cargo/bin:/root/.cargo/bin:${PATH}

RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential \
    ca-certificates \
    curl \
    libssl-dev \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*

RUN npm install --prefix /opt/test-deps playwright@1.51.1
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --profile minimal --default-toolchain stable
RUN rustup target add wasm32-unknown-unknown

WORKDIR /workspace

###############################
# 3. Runtime Stage
FROM debian:bookworm-slim

# Install only necessary runtime libraries
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy binaries from builder
COPY --from=builder /usr/src/eagle/target/release/backend /app/backend
COPY --from=builder /usr/src/eagle/target/release/seed /app/seed
COPY backend/Rocket.toml /app/Rocket.toml

ENV ROCKET_CONFIG=/app/Rocket.toml
EXPOSE 8001

CMD ["/app/backend"]
