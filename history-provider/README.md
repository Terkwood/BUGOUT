# History Provider

This microservice provides the game history when requested.  [It prevents white from missing the first turn](https://github.com/Terkwood/BUGOUT/issues/64).

## Mechanism

The gateway must request the history for a given `game_id` via the `bugout-provide-game-history-cmd` channel:

A message formatted as follows will arrive _on a topic whose key is defined to be the game_id_:

```json
{ "game_id": "51943165-7849-4c9a-9e0e-50f16132390d", "request_id": "03243984-c10b-4a86-a231-3b351364dd44",  "timestamp": 1565284695 }
```
