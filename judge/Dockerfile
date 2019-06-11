FROM gradle

# initialize demon (slow step)
RUN gradle tasks 

COPY . .

CMD ["gradle", "run"]
