import com.fasterxml.jackson.annotation.JsonTypeInfo
import com.fasterxml.jackson.annotation.JsonTypeInfo.*
import java.util.UUID

typealias ClientIdKey = UUID
typealias GameIdKey = UUID

@JsonTypeInfo(use = Id.NAME, include = As.WRAPPER_OBJECT)
data class ClientId(val underlying: UUID)

@JsonTypeInfo(use = Id.NAME, include = As.WRAPPER_OBJECT)
data class GameId(val underlying: UUID)

@JsonTypeInfo(use = Id.NAME, include = As.WRAPPER_OBJECT)
data class EventId(val underlying: UUID)
