import com.fasterxml.jackson.annotation.JsonIgnoreProperties

/**
 * An event signaling that a move has been made.
 */
@JsonIgnoreProperties(ignoreUnknown = true)
data class MoveAccepted(
    val gameId: GameId,
    val coord: Coord?
)

data class Coord(val x: Int, val y: Int)
