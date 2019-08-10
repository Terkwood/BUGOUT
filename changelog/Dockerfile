FROM gradle

RUN curl -O https://raw.githubusercontent.com/vishnubob/wait-for-it/master/wait-for-it.sh

RUN chmod 755 wait-for-it.sh

### BEGIN PROD
#TODO
#COPY . .
#RUN sh build.sh
### END   PROD

### BEGIN DEV
#TODO
COPY "bugout.changelog.jar" .
### END   DEV

#TODO
#CMD ["sh", "run.sh"]

CMD ["tail","-f","/dev/null"]
