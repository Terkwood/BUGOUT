FROM rust

WORKDIR /var/
RUN git clone https://github.com/Terkwood/BUGOUT
WORKDIR /var/BUGOUT/gateway
RUN git submodule update --init

RUN apt-get update -y 
RUN apt-get install -y clang

RUN curl -O https://raw.githubusercontent.com/vishnubob/wait-for-it/master/wait-for-it.sh

RUN chmod 755 wait-for-it.sh

RUN echo "Greetings"
RUN git fetch
RUN git checkout feature/gateway-kafka-producer
RUN git rev-parse HEAD

RUN cargo install --path .

CMD ["sh", "run.sh"]
