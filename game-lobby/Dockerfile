FROM gradle

RUN curl -O https://raw.githubusercontent.com/vishnubob/wait-for-it/master/wait-for-it.sh

RUN chmod 755 wait-for-it.sh

COPY . .

#TODO
CMD ["tail", "-f", "/dev/null"]

#TODO
#RUN sh build.sh

#TODO
#CMD ["sh", "run.sh"]
