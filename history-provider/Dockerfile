FROM gradle

RUN curl -O https://raw.githubusercontent.com/vishnubob/wait-for-it/master/wait-for-it.sh

RUN chmod 755 wait-for-it.sh

### BEGIN PROD
#COPY . .
#RUN sh build.sh
### END PROD

### BEGIN DEV
COPY "run.sh" .
COPY "bugout.historyprovider.jar" .
### END DEV

#TODO
CMD ["tail","-f","/dev/null"]

#TODO
#CMD [ "sh", "run.sh"]