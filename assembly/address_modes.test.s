; Test 6502 address modes

.include "test.inc"

; test we can use a label as address
    LDA #$ab
    STA variable
    LDX #$00
    LDX variable

    VRFY  :+
    JMP     :++

:   TestStart   $ff
    TestAddress variable, $ab
    TestX       $ab
    TestEnd

; Test specific address modes
:   LDX #$de        ; Immediate addressing mode
    STX $10         ; Zero page addressing mode
    STX $0100       ; Absolute addressing mode
    LDY #$04
    STX $10,Y       ; Zero page,Y addressing mode

    VRFY  :+
    JMP     :++

:   TestStart   $01
    TestX       $de
    TestY       $04
    TestAddress $0100, $de
    TestAddress $0010, $de
    TestAddress $0014, $de
    TestEnd

; test specific address modes
:   LDY #$ab        ; Immediate addressing mode
    STY $70         ; Zero page addressing mode
    STY $0200       ; Absolute addressing mode
    LDX #$03
    STY $60,X       ; Zero page,X addressing mode

    VRFY  :+
    JMP     :++

:   TestStart   $02
    TestY       $ab
    TestX       $03
    TestAddress $0200, $ab
    TestAddress $0070, $ab
    TestAddress $0063, $ab
    TestEnd

; test specific address modes
:   LDA #$7a        ; Immediate addressing mode
    STA $70         ; Zero page addressing mode
    STA $0300       ; Absolute addressing mode
    LDX #$02
    LDY #$04
    STA $60,X       ; Zero page,X addressing mode
    STA $60,Y       ; Zero page,Y addressing mode
    STA $0310, X    ; Absolute,X addressing mode
    STA $0310, Y    ; Absolute,X addressing mode
 
    VRFY  :+
    JMP     :++

:   TestStart   $03
    TestA       $7a
    TestX       $02
    TestY       $04
    TestAddress $0300, $7a
    TestAddress $0070, $7a
    TestAddress $0062, $7a
    TestAddress $0064, $7a
    TestAddress $0312, $7a
    TestAddress $0314, $7a
    TestEnd

; Test relative mode, conditional branching only
:   LDA #$ff
    LDX #$ff
    LDY #$ff
    CLC
    BCC loc2
    FAIL
loc1:
    LDX #$02
    CLC
    BCC loc3
    FAIL
loc2:
    LDA #$01
    CLC
    BCC loc1
    FAIL
loc3:
    LDY #$03

    VRFY    :+
    JMP     :++

:   TestStart $04
    TestA $01
    TestX $02
    TestY $03
    TestEnd


; Indirect addressing ZeroPage,X pre-index
:   LDA #$20    ; low byte
    STA $13
    LDA #$40    ; high byte
    STA $14
    LDX #$03
    LDA #$ee
    STA ($10,X)

; Indirect addressing Zeropage,Y post-index
    LDA #$60    ; low byte
    STA $20
    LDA #$40    ; high byte
    STA $21
    LDY #$03
    LDA #$cc
    STA ($20),Y

    VRFY    :+
    JMP     :++

:   TestStart   $04
    TestAddress $4020, $ee
    TestAddress $4063, $cc
    TestEnd


; End of all tests
:   HALT

.data
variable: .byte $00
