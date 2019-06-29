FROM rust

WORKDIR /var/
RUN git clone https://github.com/Terkwood/BUGOUT
WORKDIR /var/BUGOUT/gateway
RUN git submodule update --init

RUN apt-get update -y 
RUN apt-get install -y clang

RUN git fetch && git pull
RUN git checkout feature/gateway-kafka

RUN cargo install --path .

COPY "run.sh" .

RUN curl -O https://raw.githubusercontent.com/vishnubob/wait-for-it/master/wait-for-it.sh

RUN chmod 755 wait-for-it.sh

CMD ["sh", "run.sh"]
