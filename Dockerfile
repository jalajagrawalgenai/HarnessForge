FROM rust:1.85-slim AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock* ./
COPY packages/ ./packages/
RUN cargo build --release -p forge-cli

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/forge /usr/local/bin/forge
ENTRYPOINT ["forge"]
CMD ["--help"]
