FROM rust

RUN rustup default stable  

WORKDIR /var/BUGOUT/micro-history-provider

COPY . /var/BUGOUT/micro-history-provider/.

RUN cargo install --path .

ENV RUST_LOG info

CMD ["micro-history-provider"]
