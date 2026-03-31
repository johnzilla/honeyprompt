# Multi-stage build for local testing.
# Produces a minimal distroless image with the honeyprompt static binary.
#
# Build:  docker build -t honeyprompt .
# Test:   docker run --rm honeyprompt --version
# Serve:  docker run --rm -v $(pwd)/landing:/data -p 8080:8080 honeyprompt serve /data

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
COPY landing/ /landing/
ENTRYPOINT ["/usr/local/bin/honeyprompt"]
CMD ["serve", "/landing"]
