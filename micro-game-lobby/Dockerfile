# syntax=docker/dockerfile:experimental
FROM rust

RUN rustup default stable  

WORKDIR /var/BUGOUT/micro-game-lobby

COPY Cargo.toml /var/BUGOUT/micro-game-lobby/.
COPY src/ /var/BUGOUT/micro-game-lobby/src
COPY lobby-model/ /var/BUGOUT/micro-game-lobby/lobby-model

RUN --mount=type=cache,target=/usr/local/cargo,from=rust,source=/usr/local/cargo \
    --mount=type=cache,target=target \
    [ "cargo", "install", "--path", "." ]

ENV RUST_LOG trace

CMD ["micro-game-lobby"]
