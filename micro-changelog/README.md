# micro_changelog

This service is responsible for reading in moves accepted by 
the judge service, recording them, and issuing a final `MoveMade`
event.  This `MoveMade` event ultimately signals the UI and player
that their move was accepted.

This implementation is designed to run in memory-constrained environments.
