enum class ColorPref {
    Black,
    White,
    Any
}

fun isAny(c : ColorPref): Boolean = c == ColorPref.Any