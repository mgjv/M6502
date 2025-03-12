; Test the test framework itself

.include "test.inc"

; basic test
    LDX #$ff
    LDA #$00

    VRFY    :+
    JMP     :++

:   TestStart   $01
    TestEnd

; End of all tests
: HALT
    