# micro-game-lobby

A small implementation of game-lobby features, with support for:

- finding a public game on a 19x19 board
- creating and joining a private game by link, with multiple board sizes
- sensitivity to session disconnection

## Docker Buildkit support

This service is experimenting with buildkit support to
reduce docker build times:

- https://www.docker.com/blog/faster-builds-in-compose-thanks-to-buildkit-support/
- https://stackoverflow.com/a/59633394

You need to use something like [compose.sh](../compose.sh) to build this in docker compose:

```sh
COMPOSE_DOCKER_CLI_BUILD=1 DOCKER_BUILDKIT=1 docker-compose build micro-game-lobby
```

And you must run `docker-compose up` using buildkit support:

```sh
COMPOSE_DOCKER_CLI_BUILD=1 DOCKER_BUILDKIT=1 docker-compose up
```
