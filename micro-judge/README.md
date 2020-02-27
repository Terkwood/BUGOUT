# micro-judge

A minified implementation of the [judge](../judge) service, making use of [redis streams](https://redis.io/topics/streams-intro).

## Design

The goal is to stick to the same event structure used in the kafka implementation.  Procedures in this project should be single-threaded, since it's targeting the smallest potential user base.  Since this service is intended to fit alongside several other 'micro' applications, replacing _the entire kafka apparatus_, we prioritize using a small amount of RAM.

## Running Integration Tests

Yes, there is an integration test. Beware!  It will try to put values in localhost's redis instance. 😈

It must be run using a single thread.

```sh
cargo test -- --test-threads=1
```
...or...

```sh
cargo watch -x "test -- --test-threads=1"
```

## More On Redis Streams

- [Antirez: Redis streams as a pure data structure](http://antirez.com/news/128)
- [Paulius: Event sourcing with redis](https://dev.to/pdambrauskas/event-sourcing-with-redis-45ha)
