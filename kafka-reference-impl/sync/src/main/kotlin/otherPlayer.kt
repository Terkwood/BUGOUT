fun otherPlayer(player: Player): Player = when (player) {
    Player.BLACK -> Player.WHITE
    Player.WHITE -> Player.BLACK
}
