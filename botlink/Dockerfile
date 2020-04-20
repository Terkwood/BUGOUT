FROM rust

WORKDIR /var/BUGOUT/botlink

RUN rustup default stable  

COPY . /var/BUGOUT/botlink/.

RUN cargo install --path .

ENV RUST_LOG info

CMD ["botlink"]
