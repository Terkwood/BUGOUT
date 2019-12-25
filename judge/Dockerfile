FROM gradle

RUN curl -O https://raw.githubusercontent.com/vishnubob/wait-for-it/master/wait-for-it.sh

RUN chmod 755 wait-for-it.sh

COPY . .
#TODO
#RUN sh build.sh

#CMD ["sh", "run.sh"]
CMD ["tail","-f","/dev/null"]
