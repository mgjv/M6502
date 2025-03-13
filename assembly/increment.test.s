; Test for increments and decrements
; INX, INY, INC
; DEX, DEY, DEC

.include "test.inc"

; Test increments
    LDX #$10
    INX
    LDY #$20
    INY
    LDA #$a0
    STA $10
    INC $10
    STA s1
    INC s1

    VRFY    :+
    JMP     :++

:   TestStart   $01
    TestX       $11
    TestY       $21
    TestAddress $0010, $a1
    TestAddress s1, $a1
    TestEnd

; Test decrements
:   LDX #$10
    DEX
    LDY #$20
    DEY
    LDA #$a0
    STA $10
    DEC $10
    STA s1
    DEC s1

    VRFY    :+
    JMP     :++

:   TestStart   $01
    TestX       $0f
    TestY       $1f
    TestAddress $0010, $9f
    TestAddress s1, $9f
    TestEnd

; Test remaining indexed address modes for INC and DEC
:   LDA #$10
    STA s1
    STA $10
    LDA #$20
    STA s2
    STA $11
    LDA #$30
    STA s3
    STA $12
    LDX #$01
    INC s1,X
    INC $10,X
    INX
    DEC s1,X
    DEC $10,X

    VRFY    :+
    JMP     :++

:   TestStart  $03
    TestAddress s1, $10
    TestAddress s2, $21
    TestAddress s3, $2f
    TestAddress $0010, $10
    TestAddress $0011, $21
    TestAddress $0012, $2f
    TestEnd

; End of all tests
:   HALT

.data

    s1: .byte $00
    s2: .byte $00
    s3: .byte $00