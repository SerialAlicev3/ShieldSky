# ── Build stage ─────────────────────────────────────────────────────────────
FROM rust:bookworm AS builder

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        libssl-dev \
        pkg-config \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY rust/ ./rust/

WORKDIR /app/rust
RUN cargo build --release -p sss-api

# ── Runtime stage ────────────────────────────────────────────────────────────
FROM debian:bookworm-slim AS runtime

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        ca-certificates \
        libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/rust/target/release/sss-api ./sss-api

RUN mkdir -p data

ENV SSS_STORAGE_PATH=/app/data/sss-api.sqlite

EXPOSE 8088

CMD ["./sss-api"]
