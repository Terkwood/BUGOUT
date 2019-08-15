FROM confluentinc/cp-kafkacat

RUN apt-get update

RUN apt-get -y install curl

RUN curl -O https://raw.githubusercontent.com/vishnubob/wait-for-it/master/wait-for-it.sh

RUN chmod 755 wait-for-it.sh

COPY . .

CMD ["sh", "feed.sh"]