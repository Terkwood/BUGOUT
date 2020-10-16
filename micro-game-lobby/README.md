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
