# Build stage
FROM rust:latest as builder
WORKDIR /usr/src/eagle
COPY . .
RUN cargo build --release -p backend

# Runtime stage
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /usr/src/eagle/target/release/backend /app/backend
COPY --from=builder /usr/src/eagle/target/release/seed /app/seed
COPY backend/Rocket.toml /app/Rocket.toml
ENV ROCKET_CONFIG=/app/Rocket.toml
EXPOSE 8000
CMD ["/app/backend"]
