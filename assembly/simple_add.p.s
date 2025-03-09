.include "test.s.h"

    CLC
    LDA     #$00
    ADC     #$01
    ADC     #$01

    Verify test_1
 
    LDA     #$ff
    ADC     #$01

    Verify test_2

    ADC     #$00    ; Note: This is a test with carry, so it will add 1
    STA     $0030

    Verify test_3

    LDA     #$ff
    ADC     #$01
    CLC
    ADC     #$01
    
    Verify test_4

    BRK

test_1:
    TestStart  $01
    TestA   $02
    TestCarryClear
    TestEnd

test_2:
    TestStart  $02
    TestA   $00
    TestCarrySet
    TestEnd

test_3:
    TestStart  $03
    TestA   $01
    TestCarryClear
    TestEnd

test_4:
    TestStart  $04
    TestA   $01
    TestCarryClear
    TestEnd
