; Operators that don't fit any other category

; BIT, NOP

; Note: We're not going to test NOP. No need

.include "test.inc"

; BIT tests
    LDA #$f0
    STA $50
    LDA #$0f
    STA r1
    CLC

    LDA #$ff
    BIT $50
    StStatus s1
    LDA #$0f
    BIT $50
    StStatus s2

    LDA #$ff
    BIT r1
    StStatus s3
    LDA #$10
    BIT r1
    StStatus s4

    LDA #$00
    STA $60
    LDA #$ff
    STA r2

    LDA #$ff
    BIT $60
    StStatus s5
    LDA #$00
    BIT r2
    StStatus s6

    LDA #$01
    STA r3

    LDA #$01
    BIT r3
    StStatus s7
    LDA #$10
    BIT r3
    StStatus s8


    VRFY    :+
    JMP     :++

:   TestStart   $01
    TestAddress s1, %11000000
    TestAddress s2, %11000010
    TestAddress s3, %00000000
    TestAddress s4, %00000010
    TestAddress s5, %00000010
    TestAddress s6, %11000010
    TestAddress s7, %00000000
    TestAddress s8, %00000010
    TestEnd

; Well, NOP is here. Not sure how to test that it doesn't do anything
:   NOP

    VRFY    :+
    JMP     :++

:   TestStart  $02
    TestEnd

; End of all tests
:   HALT

.data
; Some result variables to prevent needing too many test blocks
    r1: .byte $de
    r2: .byte $ad
    r3: .byte $be
    r4: .byte $af

    s1: .byte $de
    s2: .byte $ad
    s3: .byte $be
    s4: .byte $ef
    s5: .byte $de
    s6: .byte $ad
    s7: .byte $be
    s8: .byte $ef