class CalculatorTests {

    @Test
    fun `1 + 1 = 2`() {
        val calculator = Calculator()
        assertEquals(2, calculator.add(1, 1), "1 + 1 should equal 2")
    }

    @ParameterizedTest(name = "{0} + {1} = {2}")
    @CsvSource(
        "0,    1,   1",
        "1,    2,   3",
        "49,  51, 100",
        "1,  100, 101"
    )
    fun add(first: Int, second: Int, expectedResult: Int) {
        val calculator = Calculator()
        assertEquals(expectedResult, calculator.add(first, second)) {
            "$first + $second should equal $expectedResult"
        }
    }

    @Test
    fun divisionByZeroError() {
        val calculator = Calculator()
        val exception = assertThrows<AssertionError> {
            calculator.div(1, 0)
        }
        assertEquals("Division by Zero", exception.message)
    }
}
