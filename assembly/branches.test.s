; Test branching instructions

.include "test.inc"

; This uses assumptions verified by the 'flags' test

; Conditional branch tests

; Test BEQ
    LDX #$ff    ; load a canonical value in X and Y
    LDY #$ff
    LDA #$00    ; set the zero flag
    BEQ :+
    FAIL        ; this should be skipped
    FAIL
:   LDX #$03    ; this should be executed.
    LDA #$01    ; clear the zero flag
    BEQ :+
    JMP :++
:   FAIL        ; should not jump here
    FAIL
:   LDY #$04    ; this should be executed.

    VRFY    :+
    JMP     :++

:   TestStart   $01
    TestX       $03
    TestY       $04
    TestEnd

; Test BNE
:   LDX #$ff    ; load a canonical value in X and Y
    LDY #$ff
    LDA #$01    ; clear the zero flag
    BNE :+
    FAIL        ; this should be skipped
    FAIL
:   LDX #$03    ; this should be executed.
    LDA #$00    ; set the zero flag
    BNE :+
    JMP :++
:   FAIL        ; should not jump here
    FAIL
:   LDY #$04    ; this should be executed

    VRFY    :+
    JMP     :++

:   TestStart   $02
    TestX       $03
    TestY       $04
    TestEnd

; Test BCS
:   LDX #$ff    ; load a canonical value in X and Y
    LDY #$ff
    SEC         ; set the carry flag
    BCS :+
    FAIL        ; this should be skipped
    FAIL
:   LDX #$03    ; this should be executed.
    CLC         ; clear the carry flag
    BCS :+
    JMP :++
:   FAIL        ; should not jump here
    FAIL
:   LDY #$04    ; this should be executed.

    VRFY    :+
    JMP     :++

:   TestStart   $03
    TestX       $03
    TestY       $04
    TestEnd

; Test BCC
:   LDX #$ff    ; load a canonical value in X and Y
    LDY #$ff
    CLC         ; clear the carry flag
    BCC :+
    FAIL        ; this should be skipped
    FAIL
:   LDX #$03    ; this should be executed.
    SEC         ; set the carry flag
    BCC :+
    JMP :++
:   FAIL        ; should not jump here
    FAIL
:   LDY #$04    ; this should be executed.

    VRFY    :+
    JMP     :++

:   TestStart   $03
    TestX       $03
    TestY       $04
    TestEnd


; Test BMI
:   LDX #$ff    ; load a canonical value in X and Y
    LDY #$ff
    LDA #$ff    ; set the negative flag
    BMI :+
    FAIL        ; this should be skipped
    FAIL
:   LDX #$03    ; this should be executed.
    LDA #$01    ; clear the negative flag
    BMI :+
    JMP :++
:   FAIL        ; should not jump here
    FAIL
:   LDY #$04    ; this should be executed.

    VRFY    :+
    JMP     :++

:   TestStart   $01
    TestX       $03
    TestY       $04
    TestEnd

; Test BPL
:   LDX #$ff    ; load a canonical value in X and Y
    LDY #$ff
    LDA #$01    ; clear the negative flag
    BPL :+
    FAIL        ; this should be skipped
    FAIL
:   LDX #$03    ; this should be executed.
    LDA #$ff    ; set the negative flag
    BPL :+
    JMP :++
:   FAIL        ; should not jump here
    FAIL
:   LDY #$04    ; this should be executed

    VRFY    :+
    JMP     :++

:   TestStart   $02
    TestX       $03
    TestY       $04
    TestEnd

; Test BVS
:   LDX #$ff    ; load a canonical value in X and Y
    LDY #$ff
    LDA #$7f    ; set the overflow flag
    ADC #$01
    BVS :+
    FAIL        ; this should be skipped
    FAIL
:   LDX #$03    ; this should be executed.
    CLV         ; clear the overflow flag
    BVS :+
    JMP :++
:   FAIL        ; should not jump here
    FAIL
:   LDY #$04    ; this should be executed

    VRFY    :+
    JMP     :++

:   TestStart   $02
    TestX       $03
    TestY       $04
    TestEnd

; Test BVC
:   LDX #$ff    ; load a canonical value in X and Y
    LDY #$ff
    CLV         ; clear the overflow flag
    BVC :+
    FAIL        ; this should be skipped
    FAIL
:   LDX #$03    ; this should be executed.
    LDA #$7f    ; set the overflow flag
    ADC #$01
    BVC :+
    JMP :++
:   FAIL        ; should not jump here
    FAIL
:   LDY #$04    ; this should be executed

    VRFY    :+
    JMP     :++

:   TestStart   $02
    TestX       $03
    TestY       $04
    TestEnd

; End of all tests
:   HALT