FROM gradle

COPY "run.sh" .

RUN curl -O https://raw.githubusercontent.com/vishnubob/wait-for-it/master/wait-for-it.sh

RUN chmod 755 wait-for-it.sh

# prod only
COPY . .
RUN sh build.sh

# dev only
#COPY "bugout.judge.jar" .

CMD ["sh", "run.sh"]
