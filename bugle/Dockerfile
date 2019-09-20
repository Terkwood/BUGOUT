FROM rust

WORKDIR /var/BUGOUT/bugle

RUN rustup default nightly

RUN COPY . /var/BUGOUT/bugle/.

RUN cargo install --path .

CMD ["bugle"]
