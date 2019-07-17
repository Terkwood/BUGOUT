FROM gradle

COPY "run.sh" .

RUN curl -O https://raw.githubusercontent.com/vishnubob/wait-for-it/master/wait-for-it.sh

RUN chmod 755 wait-for-it.sh

# You must build this outside of docker
# So that development productivity doesn't plummet
COPY "bugout.changelog.jar" .

CMD ["sh", "run.sh"]
