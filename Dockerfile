# Build stage
FROM rust:latest as builder
WORKDIR /usr/src/currency
COPY . .
COPY .env .env
RUN cargo build --release && cargo test --release

# Final stage
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y openssl ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/src/currency/target/release/currency /usr/local/bin/currency
COPY --from=builder /usr/src/currency/.env .env
CMD ["currency"]