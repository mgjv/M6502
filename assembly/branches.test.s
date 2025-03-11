; Test branching instructions

.include "test.inc"

.code
; Ensure LDA sets zero flag
    LDA #$00

    VRFY  :+
    JMP     :++

:   TestStart   $01
    TestZeroSet
    TestEnd

.data
; Ensure LDA clears zero flag
:   LDA #$fe

    VRFY  :+
    JMP     :++

:   TestStart $02
    TestZeroClear
    TestEnd

.code
; Test BEQ on flag set
:   LDX #$ff    ; load a canonical value in X
    LDA #$00    ; set the zero flag
    BEQ :+
    FAIL        ; this should be skipped
    FAIL
    FAIL
:   LDX #$03    ; this should be executed. If not, 

    VRFY    :+
    JMP     :++

.data
:   TestStart   $03
    TestX       $03
    TestEnd

; End of all tests
:   HALT