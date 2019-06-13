FROM gradle

COPY . .

RUN gradle build

CMD ["gradle", "--build-cache", "run"]
