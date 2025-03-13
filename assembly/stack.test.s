; Test stack operation and the stack instructions
; PHA, PLA, PHP, PLP
; TSX, TXS

.include "test.inc"

; Try to preserve current stack pointer. This assumes TXS and TSX work
    TSX
    STX sp_saved

; TODO Decide whether we want to test initial stack position. We probably 
;      shouldn't care, as it's up to the ROM initialisation routines

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
    ; clear all flags we can clear
    LDA #01
    ADC #01 ; This should have cleared Carry, Zero, Overflow and Negative
    CLI
    CLD
    PLP         ; Pull the all-bits-set from stack and set status

    VRFY    :+
    JMP     :++

:   TestStart   $10
    TestCarrySet
    TestNegativeSet
    TestZeroSet
    TestOverflowSet
    TestBreakClear      ; The break flag should have been cleared
    TestInterruptSet
    TestEnd


; Test PHP
:   LDA #%11111111
    PHA
    PLP             ; All flags, except BRK should now be set (see previous test)
    PHP             ; So, push them again
    PLA             ; and pull them into A

    VRFY    :+
    JMP     :++

:   TestStart   $10
    ; TODO implement some way to test
    ; The problem is that we cannot kn ow what the status of the ignored bit is
    ; and we don't (yet) have an operation to mask bits, or extract bits
    TestEnd


; End of all tests
; Restore the saved stack pointer
:   LDX sp_saved
    TXS
    HALT

.data
    sp_saved:   .byte $aa