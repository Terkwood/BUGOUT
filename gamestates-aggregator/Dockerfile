FROM gradle

RUN curl -O https://raw.githubusercontent.com/vishnubob/wait-for-it/master/wait-for-it.sh

RUN chmod 755 wait-for-it.sh

# You must build this outside of docker
# So that development productivity doesn't plummet
COPY "bugout.gamestates.aggregator.jar" .

CMD ["java", "-jar", "bugout.gamestates.aggregator.jar"]
