# Multi-stage build for local testing.
# Produces a minimal distroless image with the honeyprompt static binary.
#
# Build:  docker build -t honeyprompt .
# Test:   docker run --rm honeyprompt --version
# Serve:  docker run --rm -v hp-data:/landing -p 8080:8080 honeyprompt serve /landing --domain your-host.example

# Builds natively for the build machine's architecture (x86_64 or aarch64);
# CI builds each on a native runner and merges them into one multi-arch tag.
FROM rust:1.88-slim AS builder
WORKDIR /app
COPY . .
RUN apt-get update && apt-get install -y musl-tools && \
    TARGET="$(uname -m)-unknown-linux-musl" && \
    rustup target add "$TARGET" && \
    RUSTFLAGS="-C target-feature=+crt-static" \
    cargo build --release --target "$TARGET" && \
    strip "target/$TARGET/release/honeyprompt" && \
    cp "target/$TARGET/release/honeyprompt" /honeyprompt

FROM gcr.io/distroless/static-debian12
COPY --from=builder /honeyprompt /usr/local/bin/honeyprompt
# No config is baked in: `serve /landing --domain your-host.example` creates one
# for your domain on first start, then generates the site into the persistent
# /landing volume (see deploy/docker-compose.yml). Without --domain or an
# existing config, `serve` refuses to start rather than guess a callback host.
ENTRYPOINT ["/usr/local/bin/honeyprompt"]
CMD ["serve", "/landing"]
