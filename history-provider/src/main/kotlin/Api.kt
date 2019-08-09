data class ProvideHistoryCommand(val gameId: GameId, val reqId: ReqId, val timestamp: Int)

data class HistoryProvidedEvent(val gameId: GameId, val replyTo: ReqId, val
eventId: EventId, val history: History)
