; Test the transfer instructions
; LDA, LDX, LDY
; STA, STX, STY
; TAX, TAY, TXA, TYA
; TSX, TXS

.include "test.inc"

; load immediate (LDA, LDX, LDY)
    LDA #$01
    LDX #$02
    LDY #$03

    VRFY    :+
    JMP     :++

:   TestStart   $01
    TestA       $01
    TestX       $02
    TestY       $03
    TestEnd

; Various storage tests (STA, STX, STY)
:   LDA #$40
    LDX #$41
    LDY #$42
; Store in zero page
    STA $a0
    STX $a1
    STY $a2
; Store in absolute address
    STA $3010
    STX $3011
    STY $3012

    VRFY    :+
    JMP     :++

:   TestStart   $02
    TestAddress $00a0, $40
    TestAddress $00a1, $41
    TestAddress $00a2, $42
    TestAddress $3010, $40
    TestAddress $3011, $41
    TestAddress $3012, $42
    TestEnd


; Indexed addressing (STA, STX, STY)
:   LDA #$80
    LDX #$00
    STA $3020,X
    INX
    STA $3020,X
    INX
    STA $90,X
    INX
    STA $90,X

    LDX #$00
    LDY #$8d
    STY $95,X
    INX
    STY $95,X

    LDY #$00
    LDX #$33
    STX $a0,Y
    INY
    STX $a0,Y

    VRFY    :+
    JMP     :++

:   TestStart   $03
    TestAddress $3020, $80
    TestAddress $3021, $80
    TestAddress $0092, $80
    TestAddress $0093, $80
    TestAddress $0095, $8d
    TestAddress $0096, $8d
    TestAddress $00a0, $33
    TestAddress $00a1, $33
    TestEnd

; Indirect addressing ZeroPage,X pre-index
:   LDA #$20    ; low byte
    STA $0013
    LDA #$40    ; high byte
    STA $0014
    LDX #$03
    LDA #$ee
    STA ($10,X)

; Indirect addressing Zeropage,Y post-index
    LDA #$60    ; low byte
    STA $0020
    LDA #$40    ; high byte
    STA $0021
    LDY #$03
    LDA #$cc
    STA ($20),Y

    VRFY    :+
    JMP     :++

:   TestStart   $04
    TestAddress $4020, $ee
    TestAddress $4063, $cc
    TestEnd

; TAX, TAY, TXA, TYA
:   LDX #$66
    TXA
    STA $10
    LDY #$77
    TYA
    STA $11
    LDA #$aa
    TAX
    LDA #$bb
    TAY

    VRFY    :+
    JMP     :++

:   TestStart   $05
    TestAddress $0010, $66
    TestAddress $0011, $77
    TestX       $aa
    TestY       $bb
    TestEnd

; TODO Work out tests for this. May need CMP to work
; TSX, TXS
:   TSX

    VRFY    :+
    JMP     :++

:   TestStart   $10
    ; How do I test that the stack pointer was actually copied to X?
    TestEnd


; End of all tests
:   HALT