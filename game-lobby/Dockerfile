FROM gradle

RUN curl -O https://raw.githubusercontent.com/vishnubob/wait-for-it/master/wait-for-it.sh

RUN chmod 755 wait-for-it.sh

COPY . .


#TODO delete all below
COPY bugout.game-lobby.jar /home/gradle/.
CMD [ "tail","-f","/dev/null"]
#TODO delete all above


#TODO restore all below
#RUN sh build.sh

#RUN ./wait-for-it.sh kafka:9092 -s -- sleep 16 
#CMD ["java", "-jar", "bugout.game-lobby.jar"]
