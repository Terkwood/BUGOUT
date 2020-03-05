FROM rust

WORKDIR /var/BUGOUT/micro-changelog

RUN rustup default stable  

COPY . /var/BUGOUT/micro-changelog/.

RUN cargo install --path .

CMD ["micro_changelog"]
