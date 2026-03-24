# ---- Build stage ----
FROM rust:latest AS builder

WORKDIR /app

RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    ca-certificates \
 && rm -rf /var/lib/apt/lists/*

# Copy manifests first for better Docker layer caching
COPY Cargo.toml Cargo.lock* ./

# Create a dummy src to cache dependencies
RUN mkdir -p src && echo "fn main() {}" > src/main.rs
RUN cargo build --release

# Now copy the real source and migrations
COPY src ./src
COPY migrations ./migrations

# Touch to invalidate cache for main.rs
RUN touch src/main.rs
RUN cargo build --release

# ---- Runtime stage ----
FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
 && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/ingest /app/ingest
COPY --from=builder /app/migrations /app/migrations

EXPOSE 8081

CMD ["/app/ingest"]
