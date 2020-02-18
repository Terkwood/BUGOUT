FROM rust

WORKDIR /var/BUGOUT/micro-judge

RUN rustup default stable  

COPY . /var/BUGOUT/micro-judge/.

#RUN cargo install --path .

#TODO
#CMD ["micro-judge"]
CMD ["tail", "-f", "/dev/null"]
