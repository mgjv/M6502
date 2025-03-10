; Test 6502 address modes

.include "test.inc"

test1:
    LDX #$de        ; Immediate addressing mode
    STX $10         ; Zero page addressing mode
    STX $0100       ; Absolute addressing mode
    LDY #$04
    STX $10,Y       ; Zero page,Y addressing mode

    Verify test1_checks
    JMP     test2

test1_checks:
    TestStart   $01
    TestX       $de
    TestY       $04
    TestAddress $0100, $de
    TestAddress $0010, $de
    TestAddress $0014, $de
    TestEnd

test2:
    LDY #$ab        ; Immediate addressing mode
    STY $70         ; Zero page addressing mode
    STY $0200       ; Absolute addressing mode
    LDX #$03
    STY $60,X       ; Zero page,X addressing mode

    Verify test2_checks
    JMP     test3

test2_checks:
    TestStart   $02
    TestY       $ab
    TestX       $03
    TestAddress $0200, $ab
    TestAddress $0070, $ab
    TestAddress $0063, $ab
    TestEnd


test3:
    LDA #$7a        ; Immediate addressing mode
    STA $70         ; Zero page addressing mode
    STA $0300       ; Absolute addressing mode
    LDX #$02
    LDY #$04
    STA $60,X       ; Zero page,X addressing mode
    STA $60,Y       ; Zero page,Y addressing mode
    STA $0310, X    ; Absolute,X addressing mode
    STA $0310, Y    ; Absolute,X addressing mode
 
    Verify test3_checks
    JMP     test4

test3_checks:
    TestStart   $03
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

test4:
    HALT