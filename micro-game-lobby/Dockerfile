FROM rust

RUN rustup default stable  

WORKDIR /var/BUGOUT/micro-game-lobby

COPY . /var/BUGOUT/micro-game-lobby/.

RUN cargo install --path .

ENV RUST_LOG info

CMD ["micro-game-lobby"]
