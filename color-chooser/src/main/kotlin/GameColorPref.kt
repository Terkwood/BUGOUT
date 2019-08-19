import java.util.*

/** Represents an individual client's color
 * preference for a specific game.  Used in
 * multiple contexts:  initially, this can
 * be keyed by client ID as the result of
 * a join against ClientGameReady
 *
 * This can also be an input to the topic
 * which aggregates color prefs for a given
 * game
 */
data class GameColorPref (val clientId: ClientId, val gameId: GameId,
                          val colorPref: ColorPref)