; Tests for ADC and SBC 

.include "test.inc"

; Basic ADC flag setting
    CLC
    LDA     #$00
    ADC     #$01
    ADC     #$01

    VRFY  :+
    JMP     :++

:   TestStart   $01
    TestA       $02
    TestCarryClear
    TestOverflowClear
    TestZeroClear
    TestNegativeClear
    TestEnd
 
; Test that we roll over, and that rthe correct flags are set
:   LDA     #$ff
    ADC     #$01

    VRFY  :+
    JMP     :++

:   TestStart   $02
    TestA       $00
    TestCarrySet
    TestOverflowClear
    TestZeroSet
    TestNegativeClear
    TestEnd

; Check that carry flag is properly used
:   SEC
    ADC     #$00

    VRFY  :+
    JMP     :++

:   TestStart   $03
    TestA       $01
    TestCarryClear
    TestEnd

; Test that we can clear the carry flag
:   LDA     #$ff
    ADC     #$01
    CLC
    ADC     #$01

    VRFY  :+
    JMP     :++

:   TestStart   $04
    TestA       $01
    TestCarryClear
    TestEnd

; End of all tests
:   HALT
