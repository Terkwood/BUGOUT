FROM rust

RUN rustup default stable  

WORKDIR /var/BUGOUT/micro-sync

COPY . /var/BUGOUT/micro-sync/.

RUN cargo install --path .

ENV RUST_LOG info

CMD ["micro-sync"]
