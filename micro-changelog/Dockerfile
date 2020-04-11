FROM rust

WORKDIR /var/BUGOUT/micro-changelog

RUN rustup default stable  

COPY . /var/BUGOUT/micro-changelog/.

#TODO
#RUN cargo install --path .

ENV RUST_LOG info

#TODO
#CMD ["micro_changelog"]
CMD [ "tail","-f","/dev/null"]
