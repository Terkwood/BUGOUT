# syntax=docker/dockerfile:experimental
FROM rust as builder

RUN rustup default stable  

WORKDIR /app-src

COPY . /app-src/.

RUN --mount=type=cache,target=/app-src/target \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/usr/local/cargo/registry \
    [ "cargo", "build", "--release" ]

RUN --mount=type=cache,target=/app-src/target \
    ["cp", "/app-src/target/release/undo", "/usr/local/bin/undo"]

FROM debian:stable-slim

RUN apt-get update

RUN apt install -y libssl-dev

COPY --from=builder /usr/local/bin/undo /usr/local/bin/undo

WORKDIR /BUGOUT

ENV RUST_LOG info

CMD ["undo"]
