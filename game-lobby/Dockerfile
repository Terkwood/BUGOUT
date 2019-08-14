FROM gradle

RUN curl -O https://raw.githubusercontent.com/vishnubob/wait-for-it/master/wait-for-it.sh

RUN chmod 755 wait-for-it.sh

COPY . .

RUN sh build.sh

RUN ./wait-for-it.sh kafka:9092 -s -- sleep 16
CMD ["java", "-jar", "bugout.game-lobby.jar"]
