# BUGOUT gateway

rust/rocket web server which authorizes BUGOUT requests and forwards them to an internal kafka cluster serving the [judge](../judge/README.md).

## Potential JWT auth

Prototype efforts lack any authorization or authentication ability whatsoever.

In the event that we want to authenticate players, use[jsonwebtoken](https://github.com/Keats/jsonwebtoken) or [frank_jwt](https://github.com/GildedHonour/frank_jwt).
