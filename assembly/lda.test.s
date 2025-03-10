.include "test.inc"

test1:
    CLC
    LDA     #$00
    ADC     #$01
    ADC     #$01

    Verify test1_checks
    JMP     test2

test1_checks:
    TestStart  $01
    TestA   $02
    TestCarryClear
    TestEnd
 
 test2:
    LDA     #$ff
    ADC     #$01

    Verify test2_checks
    JMP test3

test2_checks:
    TestStart  $02
    TestA   $00
    TestCarrySet
    TestEnd

test3:
    ADC     #$00    ; Note: This is a test with carry, so it will add 1
    STA     $0030

    Verify test3_checks
    JMP test4

test3_checks:
    TestStart  $03
    TestA   $01
    TestCarryClear
    TestEnd

test4:
    LDA     #$ff
    ADC     #$01
    CLC
    ADC     #$01

    Verify test4_checks
    JMP test5

test4_checks:
    TestStart  $04
    TestA   $01
    TestCarryClear
    TestEnd

test5:
    HALT
