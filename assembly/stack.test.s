; Test stack operation and the stack instructions
; PHA, PLA, PHP, PLP
; TSX, TXS

.include "test.inc"

; Try to preserve current stack pointer. This assumes TXS and TSX work
    TSX
    STX sp_saved

; basic stack test #1: PHA
; NOTE: Ensure the first two tests remain together
    LDA #$ee
    PHA
    LDA #$dd
    PHA
    LDA #$11
    PHA

    VRFY    :+
    JMP     :++

:   TestStart   $01
    TestStack   $01, $11
    TestStack   $02, $dd
    TestStack   $03, $ee
    TestEnd

; basic stack test #2: PLA
; NOTE: Ensure the first two tests remain together
:   LDA #$00
    PLA         ; We should now have $11 in A
    PLA         ; We should now have $dd in A

    VRFY    :+
    ; remove the last test element from the stack
    PLA
    JMP     :++

:   TestStart   $02
    TestA       $dd
    TestEnd

; Test TXS, TSX
:   LDX #$7f    ; unlikely we'll be overwriting anything here, assuming stack started at $ff
    TXS
    LDX #$00
    TSX

    VRFY    :+
    JMP     :++

:   TestStart   $10
    TestStackPointer $7f
    TestX $7f
    TestEnd

; Test PLP
:   LDA #%11111111
    PHA
    ; clear all status flags we can clear
    LDA #01
    ADC #01 ; This should have cleared Carry, Zero, Overflow and Negative
    CLI
    CLD
    PLP ; Pull the all-bits-set from stack and set status

    VRFY    :+
    JMP     :++

:   TestStart   $10
    TestCarrySet
    TestNegativeSet
    TestZeroSet
    TestOverflowSet
    TestInterruptSet
    ; The value of the BRK flag is indeterminate
    TestEnd

; Test PHP 1
:   LDA #%11111111
    PHA
    PLP             ; All flags, except BRK should now be set (see previous test)
    StStatus a1, %11001111 ; Store A but mask out bits 4 and 5
    PHP             ; So, push them again
    PLA             ; and pull them into A
    AND #%11001111  ; mask out bits 4 and 5

    VRFY    :+
    JMP     :++

:   TestStart   $20
    TestAddress a1, %11001111 ; all but ignored and brk should be set
    TestA %11001111
    TestEnd

; Test PHP 2
:   LDA #$00
    PHA
    PLP             ; All flags should now be clear
    StStatus a1, %11001111 ; Store A but mask out bits 4 and 5
    PHP             ; Push them again
    PLA             ; and pull them into A
    AND #%11001111  ; mask out bits 4 and 5

    VRFY    :+
    JMP     :++

:   TestStart   $21
    TestAddress a1, %00 ; all bits should be clear
    TestA %00
    TestEnd

; Test PHP 3
:   LDA #$91
    PHA
    PLP
    StStatus a1, %11001111 ; Store A but mask out bits 4 and 5
    PHP             ; Push them again
    PLA             ; and pull them into A
    AND #%11001111  ; mask out bits 4 and 5

    VRFY    :+
    JMP     :++

:   TestStart   $22
    TestAddress a1, $81
    TestA $81
    TestEnd


; End of all tests
; Restore the saved stack pointer
:   LDX sp_saved
    TXS
    HALT

.data
    sp_saved:   .byte $aa

    ; Some result variables to prevent needing too many test blocks
    r1: .byte $de
    r2: .byte $ad
    r3: .byte $be
    r4: .byte $af

    a1: .byte $de
    a2: .byte $ad
    a3: .byte $be
    a4: .byte $ef