FROM rust

WORKDIR /var/BUGOUT/botlink

ENV RUST_LOG info

RUN rustup default stable  

COPY . /var/BUGOUT/botlink/.

RUN cargo install --path .

CMD ["botlink"]
