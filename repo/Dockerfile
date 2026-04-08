# Multi-stage build for minimal runtime image
# Stage 1: Build
FROM rust:1.88-bookworm AS builder

RUN apt-get update && apt-get install -y libpq-dev && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY Cargo.toml Cargo.lock* ./
# Pre-build dependencies for layer caching
RUN mkdir -p src && echo "fn main() {}" > src/main.rs && cargo build --release 2>/dev/null || true
RUN rm -rf src

COPY . .
# Touch main.rs so cargo knows to rebuild
RUN touch src/main.rs
RUN cargo build --release

# Stage 2: Runtime
FROM debian:bookworm-slim

RUN apt-get update && \
    apt-get install -y --no-install-recommends libpq5 ca-certificates curl && \
    rm -rf /var/lib/apt/lists/*

RUN useradd -r -s /bin/false retailops
WORKDIR /app

COPY --from=builder /app/target/release/retailops /app/retailops
COPY --from=builder /app/migrations /app/migrations

RUN mkdir -p /app/data/uploads && chown -R retailops:retailops /app

USER retailops

ENV RUST_LOG=info
ENV BIND_ADDR=0.0.0.0:8080
ENV FILE_STORAGE_PATH=/app/data/uploads

EXPOSE 8080

HEALTHCHECK --interval=30s --timeout=5s --start-period=30s --retries=3 \
    CMD curl -f http://localhost:8080/api/v1/health || exit 1

CMD ["./retailops"]
