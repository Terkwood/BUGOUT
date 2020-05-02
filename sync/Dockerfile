FROM gradle

RUN curl -O https://raw.githubusercontent.com/vishnubob/wait-for-it/master/wait-for-it.sh

RUN chmod 755 wait-for-it.sh

COPY . .

# ..todo
#RUN sh build.sh --no-daemon

# ..todo
#CMD ["sh", "run.sh"]
CMD ["tail", "-f", "/dev/null"]
