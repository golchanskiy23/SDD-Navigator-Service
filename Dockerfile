FROM rust:1.89-slim AS builder

RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY Cargo.toml Cargo.lock ./

RUN mkdir src && echo "fn main() {}" > src/main.rs

RUN cargo build --release && rm -rf src

COPY src ./src

RUN rm -f target/release/sdd-navigator-service && \
    rm -f target/release/deps/sdd_navigator_service* && \
    touch src/main.rs && \
    cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

RUN useradd -r -s /bin/false sdd-navigator

WORKDIR /app

COPY --from=builder /app/target/release/sdd-navigator-service /usr/local/bin/

RUN chown -R sdd-navigator:sdd-navigator /app

USER sdd-navigator

EXPOSE 3000

HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:3000/health || exit 1

CMD ["sdd-navigator-service"]
