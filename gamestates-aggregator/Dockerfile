FROM gradle

# You must build this outside of docker
# So that development productivity doesn't plummet
COPY "bugout.gamestates.aggregator.jar" .

CMD ["java", "-jar", "bugout.gamestates.aggregator.jar"]
