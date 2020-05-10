FROM rust

RUN rustup default stable  

WORKDIR /var/BUGOUT/micro-judge

COPY . /var/BUGOUT/micro-judge/.

RUN cargo install --path .

ENV RUST_LOG info

CMD ["micro_judge"]
