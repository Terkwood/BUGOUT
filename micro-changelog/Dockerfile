FROM rust

RUN rustup default stable  

WORKDIR /var/BUGOUT/micro-changelog

COPY . /var/BUGOUT/micro-changelog/.

RUN cargo install --path .

ENV RUST_LOG info

CMD ["micro_changelog"]
