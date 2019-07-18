FROM gradle

COPY "run.sh" .

RUN curl -O https://raw.githubusercontent.com/vishnubob/wait-for-it/master/wait-for-it.sh

RUN chmod 755 wait-for-it.sh

### BEGIN PROD
COPY . .
RUN sh build.sh
### END   PROD

### BEGIN DEV
#COPY "bugout.judge.jar" .
### END   DEV

CMD ["sh", "run.sh"]
