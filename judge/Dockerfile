FROM gradle

# You must build this outside of docker
# So that development productivity doesn't plummet
COPY "bugout.judge.jar" .

CMD ["java", "-jar", "bugout.judge.jar"]
