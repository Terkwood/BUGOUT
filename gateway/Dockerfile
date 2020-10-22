# syntax=docker/dockerfile:experimental
FROM rust as builder

RUN rustup default stable  

WORKDIR /app-src

COPY Cargo.toml src/ /app-src/

RUN --mount=type=cache,target=/app-src/target \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/usr/local/cargo/registry \
    [ "cargo", "build", "--release" ]

RUN --mount=type=cache,target=/app-src/target \
    ["cp", "/app-src/target/release/gateway", "/usr/local/bin/gateway"]

FROM debian:stable-slim

COPY --from=builder /usr/local/bin/gateway /usr/local/bin/gateway

WORKDIR /run-bugout

# Hack to make sure env is _potentially_ copied (but not required)
# https://stackoverflow.com/questions/31528384/conditional-copy-add-in-dockerfile
COPY Cargo.toml .en[v] /run-bugout/

ENV RUST_LOG info

CMD ["gateway"]
