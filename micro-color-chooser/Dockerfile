FROM rust

RUN rustup default stable  

WORKDIR /var/BUGOUT/micro-color-chooser

COPY . /var/BUGOUT/micro-color-chooser/.

RUN cargo install --path .

ENV RUST_LOG trace

CMD ["micro-color-chooser"]
