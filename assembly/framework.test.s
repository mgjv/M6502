; Test the test framework itself

.include "test.inc"

    LDX #$ff
    LDA #$00
    BEQ target
    FAIL
    LDX #01

target:

    VRFY    :+
    JMP     :++

:   TestStart   $01
    TestEnd

: HALT
    