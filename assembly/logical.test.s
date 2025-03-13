; Logical operations
; AND, EOR, ORA

.include "test.inc"

; AND: basic test for value and flags
    LDA #$80 ; clears zero, sets negative
    AND #$00 ; sets zero, clears negative - 00

    VRFY    :+
    JMP     :++

:   TestStart   $01
    TestA       $00
    TestZeroSet
    TestNegativeClear
    TestEnd

; AND: basic test for value and flags
:   LDA #$ff
    LDX #$00 ; sets zero, clears negative
    AND #$ff ; clears zero, sets negative - ff

    VRFY    :+
    JMP     :++

:   TestStart  $02
    TestA      $ff
    TestZeroClear
    TestNegativeSet
    TestEnd


; EOR: basic test for value and flags
:   LDA #$ff ; clears zero, sets negative
    EOR #$ff ; sets zero, clears negative 00

    VRFY    :+
    JMP     :++

:   TestStart   $03
    TestA       $00
    TestZeroSet
    TestNegativeClear
    TestEnd

; EOR: basic test for value and flags
:   LDA #$00 ; sets zero, clears negative
    EOR #$ff ; clears zero, sets negative - ff

    VRFY    :+
    JMP     :++

:   TestStart  $04
    TestA      $ff
    TestZeroClear
    TestNegativeSet
    TestEnd


; ORA: basic test for value and flags
:   LDA #$00 ; sets zero, clears negative
    LDX #$ff ; clears zero, sets negative
    ORA #$00 ; sets zero, clears negative 00

    VRFY    :+
    JMP     :++

:   TestStart   $05
    TestA       $00
    TestZeroSet
    TestNegativeClear
    TestEnd

; ORA: basic test for value and flags
:   LDA #$00 ; sets zero, clears negative
    ORA #$ff ; clears zero, sets negative - ff

    VRFY    :+
    JMP     :++

:   TestStart  $06
    TestA      $ff
    TestZeroClear
    TestNegativeSet
    TestEnd

; AND: some address modes and values
:   LDA #$0f
    AND #$ff
    STA r1
    ; absolute address mode
    LDA #$f0
    STA r2
    LDA #$0f
    AND r2
    STA r3
    ; zero page
    LDA #%10000001
    STA $10
    LDA #%10001001
    AND $10
    STA r4
    ; indirect absolute
    LDA #$01
    STA a1
    LDA #$03
    STA a2
    LDA #$ff
    LDX #$01
    AND a1,X ; Should be $03 now
    STA a1

    VRFY    :+
    JMP     :++

:   TestStart  $10
    ; absolute address mode tests
    TestAddress r1, $0f
    TestAddress r2, $f0
    TestAddress r3, $00
    ; zero page
    TestAddress r4, %10000001
    ; indirect absolute
    TestAddress a1, $03
    TestAddress a2, $03
    TestEnd

; EOR: some address modes and values
:   LDA #$0f
    EOR #$ff
    STA r1
    ; absolute address mode
    LDA #$f0
    STA r2
    LDA #$0f
    EOR r2
    STA r3
    ; zero page
    LDA #%10000001
    STA $10
    LDA #%10001001
    EOR $10
    STA r4
    ; indirect absolute
    LDA #$01
    STA a1
    LDA #$03
    STA a2
    LDA #$ff
    LDX #$01
    EOR a1,X
    STA a1

    VRFY    :+
    JMP     :++

:   TestStart  $11
    ; absolute address mode tests
    TestAddress r1, $f0
    TestAddress r2, $f0
    TestAddress r3, $ff
    ; zero page
    TestAddress r4, %00001000
    ; indirect absolute
    TestAddress a1, $fc
    TestAddress a2, $03
    TestEnd

; ORA: some address modes and values
:   LDA #$0f
    ORA #$7a
    STA r1
    ; absolute address mode
    LDA #$f0
    STA r2
    LDA #$0f
    ORA r2
    STA r3
    ; zero page
    LDA #%10000001
    STA $10
    LDA #%10001001
    ORA $10
    STA r4
    ; indirect absolute
    LDA #$01
    STA a1
    LDA #$03
    STA a2
    LDA #$30
    LDX #$01
    ORA a1,X
    STA a1

    VRFY    :+
    JMP     :++

:   TestStart  $12
    ; absolute address mode tests
    TestAddress r1, $7f
    TestAddress r2, $f0
    TestAddress r3, $ff
    ; zero page
    TestAddress r4, %10001001
    ; indirect absolute
    TestAddress a1, $33
    TestAddress a2, $03
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