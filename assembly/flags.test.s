; Test setting of flags

.include "test.inc"

; TODO Add tests for carry flag

; Ensure LDA sets zero flag
    LDA #$00

    VRFY    :+
    JMP     :++

:   TestStart   $01
    TestZeroSet
    TestEnd

; Ensure LDA clears zero flag
:   LDA #$01

    VRFY    :+
    JMP     :++

:   TestStart $02
    TestZeroClear
    TestEnd

; Ensure LDA sets negative flag
:   LDA #$FF

    VRFY    :+
    JMP     :++

:   TestStart   $03
    TestNegativeSet
    TestEnd

; Ensure LDA clears negative flag
:   LDA #$01

    VRFY    :+
    JMP     :++

:   TestStart   $04
    TestNegativeClear
    TestEnd

; Ensure Overflow flag gets set correctly

; 1 + 1 = 2
:   CLC
    LDA #$01
    ADC #$01

    VRFY    :+
    JMP     :++

:   TestStart   $10
    TestOverflowClear
    TestEnd

; 1 + -1 = 0; 1 + 255 = 0
:   CLC 
    LDA #$01
    ADC #$ff

    VRFY    :+
    JMP     :++

:   TestStart   $11
    TestOverflowClear
    TestEnd

; 127 + 1 = 128
:   CLC
    LDA #$7f
    ADC #$01

    VRFY    :+
    JMP     :++

:   TestStart   $12
    TestOverflowSet
    TestEnd

; -128 + -1 = -129
:   CLC
    LDA #$80
    ADC #$ff

    VRFY    :+
    JMP     :++

:   TestStart   $13
    TestOverflowSet
    TestEnd

; Check we can use CLV
:   CLC
    LDA #$7f
    ADC #$01    ; Overflow is set here
    CLV

    VRFY    :+
    JMP     :++

:   TestStart   $14
    TestOverflowClear
    TestEnd


; FIXME Properly implement SBC
; 0 - 1 = -1, V clear
:   SEC
    LDA #$00
    SBC #$01

    VRFY    :+
    JMP     :++

:   TestStart   $15
    TestOverflowClear
    TestEnd

; -128 - 1 = -129, V set
:   SEC      
    LDA #$80
    SBC #$01

    VRFY    :+
    JMP     :++

:   TestStart   $16
    TestOverflowSet
    TestEnd

; 127 - -1 = 128, V set
:   SEC      
    LDA #$7F
    SBC #$FF

    VRFY    :+
    JMP     :++

:   TestStart   $17
    TestOverflowSet
    TestEnd

; End of all tests
:   HALT