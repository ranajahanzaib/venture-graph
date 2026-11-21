FROM rust:latest AS api-builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
COPY migrations ./migrations
COPY docs/openapi.json ./docs/openapi.json
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    cargo build --release -p venturegraph-api --bins && \
    cp /app/target/release/venturegraph-api /app/venturegraph-api && \
    cp /app/target/release/venturegraph-import /app/venturegraph-import
FROM debian:stable-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates libpq5 && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=api-builder /app/venturegraph-api /usr/local/bin/venturegraph-api
COPY --from=api-builder /app/venturegraph-import /usr/local/bin/venturegraph-import
COPY migrations ./migrations
USER 10001:10001
ENV API_ADDR=0.0.0.0:8080
EXPOSE 8080
ENTRYPOINT ["venturegraph-api"]
