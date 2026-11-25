# Multi-stage build for local testing.
# Produces a minimal distroless image with the honeyprompt static binary.
#
# Build:  docker build -t honeyprompt .
# Test:   docker run --rm honeyprompt --version
# Serve:  docker run --rm -v hp-data:/landing -p 8080:8080 honeyprompt serve /landing --domain your-host.example

FROM rust:1.88-slim AS builder
WORKDIR /app
COPY . .
RUN apt-get update && apt-get install -y musl-tools && \
    rustup target add x86_64-unknown-linux-musl && \
    RUSTFLAGS="-C target-feature=+crt-static" \
    cargo build --release --target x86_64-unknown-linux-musl && \
    strip target/x86_64-unknown-linux-musl/release/honeyprompt

FROM gcr.io/distroless/static-debian12
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/honeyprompt /usr/local/bin/honeyprompt
# No config is baked in: `serve /landing --domain your-host.example` creates one
# for your domain on first start, then generates the site into the persistent
# /landing volume (see deploy/docker-compose.yml). Without --domain or an
# existing config, `serve` refuses to start rather than guess a callback host.
ENTRYPOINT ["/usr/local/bin/honeyprompt"]
CMD ["serve", "/landing"]
