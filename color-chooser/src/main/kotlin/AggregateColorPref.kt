/**
 * Each game must see two of these messages in order
 * to trigger a color choice.  Keyed by game ID
 */
data class AggregateColorPref(val gameId: GameId,
                              val clientId: ClientId,
                              val colorPref: ColorPref)
