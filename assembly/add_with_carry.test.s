; Tests for ADC and SBC 

; Also see some tests for flags in 'flags' test

; TODO This needs many more tests

.include "test.inc"

; Basic ADC flag setting
    CLC
    LDA #$00
    ADC #$01
    ADC #$01

    VRFY    :+
    JMP     :++

:   TestStart   $01
    TestA       $02
    TestCarryClear
    TestOverflowClear
    TestZeroClear
    TestNegativeClear
    TestEnd
 
; ADC Test that we roll over, and that rthe correct flags are set
:   CLC
    LDA #$ff
    ADC #$01

    VRFY    :+
    JMP     :++

:   TestStart   $02
    TestA       $00
    TestCarrySet
    TestOverflowClear
    TestZeroSet
    TestNegativeClear
    TestEnd

; ADC Check that carry flag is properly used, and reset
:   SEC
    ADC #$00

    VRFY    :+
    JMP     :++

:   TestStart   $03
    TestA       $01
    TestCarryClear
    TestEnd

; ADC Test that we can clear the carry flag
:   CLC
    LDA #$ff
    ADC #$01
    CLC
    ADC #$01

    VRFY    :+
    JMP     :++

:   TestStart   $04
    TestA       $01
    TestCarryClear
    TestEnd

; SBC: 1 - 1 = 0; C set
:   SEC
    LDA #$01
    SBC #$01

    VRFY    :+
    JMP     :++

:   TestStart   $10
    TestA       $00
    TestZeroSet
    TestCarrySet
    TestOverflowClear
    TestNegativeClear
    TestEnd

; SBC 1 - 2 = -1/ff; C clear
:   SEC
    LDA #$01
    SBC #$02

    VRFY    :+
    JMP     :++

:   TestStart   $11
    TestA       $ff
    TestZeroClear
    TestCarryClear
    TestOverflowClear
    TestNegativeSet
    TestEnd

; SBC -128 - 1 = -129; V set, C clear
:   SEC
    LDA #$80
    SBC #$01

    VRFY    :+
    JMP     :++

:   TestStart   $12
    TestA       $7f
    TestZeroClear
    TestCarrySet
    TestOverflowSet
    TestNegativeClear
    TestEnd


; Check that carry flag is properly used, and reset
:   CLC
    LDA #$02
    SBC #$00

    VRFY    :+
    JMP     :++

:   TestStart   $13
    TestA       $01
    TestCarrySet
    TestEnd


; End of all tests
:   HALT
