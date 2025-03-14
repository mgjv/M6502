; Test the framework itself

.include "test.inc"

; Test StStatus
; TODO probably should test that status register is (almost) unaffected
    LDA #$7f
    ADC #$01 ; This should set V and N and clear Z (result $80 = -128)
    STA a1
    SEC
    SED
    SEI
    StStatus r1
    StStatus r2, %01000001 ; Test we can pass our own mask

    LDA #$00 ; set Z, clear N. C and V should still be set
    StStatus r3
    CLV
    StStatus r4

    VRFY    :+
    JMP     :++

:   TestStart   $01
    TestAddress a1, $80 ; ensure macro has not affected A
    TestAddress r1, %11000001
    TestAddress r2, %01000001
    TestAddress r3, %01000011
    TestAddress r4, %00000011
    TestEnd

; Test memory clearing
:   LDA #$ff ; put a marker value in the test locations
    STA r1
    STA a4
    ClearMemory r1, 8

    VRFY    :+
    
    ClearMemory r2, 6, $ae

    VRFY    :++
    JMP     :+++

:   TestStart  $02
    TestAddress r1, $00
    TestAddress a4, $00
    TestEnd

:   TestStart  $03
    TestAddress r1, $00
    TestAddress r2, $ae    
    TestAddress a3, $ae
    TestAddress a4, $00
    TestEnd

; End of all tests
:   HALT

.data
; Some result variables to prevent needing too many test blocks
    r1: .byte $de
    r2: .byte $ad
    r3: .byte $be
    r4: .byte $af

    a1: .byte $de
    a2: .byte $ad
    a3: .byte $be
    a4: .byte $ef