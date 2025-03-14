; Bitshift operations
; ASL, LSR, ROL, ROR

.include "test.inc"

; ASL: basic test for value and flags
    LDA #$80 ; clears zero, sets negative
    CLC
    ASL      ; sets zero, clears negative, sets carry

    VRFY    :+
    JMP     :++

:   TestStart   $01
    TestA       $00
    TestCarrySet
    TestZeroSet
    TestNegativeClear
    TestEnd

; ASL: basic test for value and flags
:   LDA #$7f
    LDX #$00 ; sets zero, clears negative
    SEC
    ASL      ; clears zero, clears carry, sets negative - fe

    VRFY    :+
    JMP     :++

:   TestStart  $02
    TestA      $fe
    TestCarryClear
    TestZeroClear
    TestNegativeSet
    TestEnd

; LSR: basic test for value and flags
:   LDA #$01 ; clears zero, clears negative
    LDX #$ff ; clears zero, sets negative
    CLC
    LSR ; sets zero, clears negative 00

    VRFY    :+
    JMP     :++

:   TestStart   $03
    TestA       $00
    TestZeroSet
    TestNegativeClear
    TestCarrySet
    TestEnd

; LSR: basic test for value and flags
:   LDA #$fe
    LDX #$00 ; sets zero, clears negative
    SEC
    LSR      ; clears carry, clears zero, clears negative - 7f

    VRFY    :+
    JMP     :++

:   TestStart  $04
    TestA      $7f
    TestZeroClear
    TestNegativeClear
    TestCarryClear
    TestEnd

; LSR: ensure negative is indeed unset
:   LDA #$fe ; clears zero, sets negative
    SEC
    LSR      ; clears carry, clears zero, clears negative - 7f

    VRFY    :+
    JMP     :++

:   TestStart  $05
    TestA      $7f
    TestZeroClear
    TestNegativeClear
    TestCarryClear
    TestEnd

; ROL: basic test for value and flags
:   LDA #$80 ; clears zero, sets negative
    CLC
    ROL      ; sets zero, clears negative, sets carry

    VRFY    :+
    JMP     :++

:   TestStart   $06
    TestA       $00
    TestCarrySet
    TestZeroSet
    TestNegativeClear
    TestEnd

; ROL: basic test for value and flags
:   LDA #$7f
    LDX #$00 ; sets zero, clears negative
    SEC      ; sets carry
    ROL      ; clears zero, clears carry, sets negative - ff

    VRFY    :+
    JMP     :++

:   TestStart  $07
    TestA      $ff
    TestCarryClear
    TestZeroClear
    TestNegativeSet
    TestEnd

; ROR: basic test for value and flags
:   LDA #$01 ; clears zero, clears negative
    LDX #$ff ; clears zero, sets negative
    CLC
    ROR      ; sets zero, clears negative, sets carry - 00

    VRFY    :+
    JMP     :++

:   TestStart   $08
    TestA       $00
    TestZeroSet
    TestNegativeClear
    TestCarrySet
    TestEnd

; ROR: basic test for value and flags
:   LDA #$fe
    LDX #$00 ; sets zero, clears negative
    SEC
    ROR      ; clears carry, clears zero, sets negative - ff

    VRFY    :+
    JMP     :++

:   TestStart  $09
    TestA      $ff
    TestZeroClear
    TestNegativeSet
    TestCarryClear
    TestEnd

; ASL: some address modes and values
:   LDA #$0f
    ASL
    STA r1
    ; absolute address mode
    LDA #$f0
    STA r2
    ASL r2
    ; zero page
    LDA #%10000001
    STA $10
    ASL $10
    ; indirect absolute
    LDA #$01
    STA a1
    LDA #$03
    STA a2
    LDX #$01
    ASL a1,X

    VRFY    :+
    JMP     :++

:   TestStart  $10
    ; absolute address mode tests
    TestAddress r1, $1e
    TestAddress r2, $e0
    ; zero page
    TestAddress $0010, %00000010
    ; indirect absolute
    TestAddress a1, $01
    TestAddress a2, $06
    TestEnd

; LSR: some address modes and values
:   LDA #$0f
    LSR
    STA r1
    ; absolute address mode
    LDA #$f0
    STA r2
    LSR r2
    ; zero page
    LDA #%10000001
    STA $10
    LSR $10
    ; indirect absolute
    LDA #$80
    STA a1
    LDA #$c0
    STA a2
    LDX #$01
    LSR a1,X

    VRFY    :+
    JMP     :++

:   TestStart  $11
    ; absolute address mode tests
    TestAddress r1, $07
    TestAddress r2, $78
    ; zero page
    TestAddress $0010, %01000000
    ; indirect absolute
    TestAddress a1, $80
    TestAddress a2, $60
    TestEnd

; ROL: some address modes and values
:   LDA #$0f
    CLC
    ROL
    STA r1
    LDA #$0f
    SEC
    ROL
    STA r2
    ; absolute address mode
    LDA #$f0
    STA r3
    CLC
    ROL r3
    LDA #$f0
    STA r4
    SEC
    ROL r4
    ; zero page
    LDA #%10000001
    STA $10
    SEC
    ROL $10
    ; indirect absolute
    LDA #$80
    STA a1
    LDA #$0c
    STA a2
    LDX #$01
    CLC
    ROL a1,X

    VRFY    :+
    JMP     :++

:   TestStart  $12
    ; absolute address mode tests
    TestAddress r1, $1e
    TestAddress r2, $1f
    TestAddress r3, $e0
    TestAddress r4, $e1
    ; zero page
    TestAddress $0010, %00000011
    ; indirect absolute
    TestAddress a1, $80
    TestAddress a2, $18
    TestEnd

; ROR: some address modes and values
:   LDA #$0f
    CLC
    ROR
    STA r1
    LDA #$0f
    SEC
    ROR
    STA r2
    ; absolute address mode
    LDA #$f0
    STA r3
    CLC
    ROR r3
    LDA #$f0
    STA r4
    SEC
    ROR r4
    ; zero page
    LDA #%10000001
    STA $10
    SEC
    ROR $10
    ; indirect absolute
    LDA #$80
    STA a1
    LDA #$0c
    STA a2
    LDX #$01
    CLC
    ROR a1,X

    VRFY    :+
    JMP     :++

:   TestStart  $13
    ; absolute address mode tests
    TestAddress r1, $07
    TestAddress r2, $87
    TestAddress r3, $78
    TestAddress r4, $f8
    ; zero page
    TestAddress $0010, %11000000
    ; indirect absolute
    TestAddress a1, $80
    TestAddress a2, $06
    TestEnd

; End of all tests
:   HALT

.data
    ; Some result variables to prevent needing too many test blocks
    r1: .byte $de
    r2: .byte $ad
    r3: .byte $be
    r4: .byte $ef

    a1: .byte $de
    a2: .byte $ad
    a3: .byte $be
    a4: .byte $ef