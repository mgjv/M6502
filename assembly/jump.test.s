; Tests for jumps
; JMP, JSR, RTS

; We won't be testing JMP, because it's so fundamental to everything
; that no test would work anyway if it failed.

.include "test.inc"

; first test
    LDA #$00
    JSR subroutine

    VRFY    :+
    JMP     :++

:   TestStart   $01
    TestA       $ff
    TestAddress r1, $ff
    TestEnd

; second test
:   NOP

    VRFY    :+
    JMP     :++

:   TestStart  $02
    TestEnd

; End of all tests
:   HALT

    ; In case we don't jump far enough
    FAIL
    FAIL
subroutine:
    LDA #$FF
    STA r1
    RTS
    ; In case we keep going here
    FAIL
    FAIL

.data
; Some result variables to prevent needing too many test blocks
    r1: .byte $de
