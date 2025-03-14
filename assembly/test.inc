; Macros to assist in testing. There are pseudo-ionstructions that 
; need to be handled by the CPU in test mode only (see cpu.rs),
; and there are specific test operators that are only active in VRFY mode

; This is a pseudo instruction that will start a verification test l;ocated at the 
; given address/label
.macro  VRFY  address
    .byte $fa
    .addr address
.endmacro

; Fail the test. Use this to identify places in the code that should not be 
; reached
.macro FAIL
    .byte $fb
.endmacro

; This is a pseudo-instruction that will cause the computer to shut down
.macro HALT
    .byte $fc
.endmacro

; The following instructions are only valid during a VRFY started loop
; which ends when TestEnd is encountered

; Identify the start of a test. Tests fail if this is not present
; This is deliberately an unimplemented c6502 value, so we get an error if
; the program_counter ends up on one of these
.macro  TestStart id
    .byte $c2, id
.endmacro

; Use the most likely "wrong" byte value for the end marker
.macro  TestEnd
    .byte $00
.endmacro

; Test the contents of registers
.macro  TestA   e
    .byte $01, e
.endmacro
.macro  TestX   e
    .byte $02, e
.endmacro
.macro  TestY   e
    .byte $03, e
.endmacro
.macro TestSP   e
    .byte $08, e
.endmacro

; Test the status flags
.macro  TestCarrySet
    .byte $30
.endmacro
.macro  TestCarryClear
    .byte $31
.endmacro
.macro  TestZeroSet
    .byte $32
.endmacro
.macro  TestZeroClear
    .byte $33
.endmacro
.macro  TestNegativeSet
    .byte $34
.endmacro
.macro  TestNegativeClear
    .byte $35
.endmacro
.macro  TestOverflowSet
    .byte $36
.endmacro
.macro  TestOverflowClear
    .byte $37
.endmacro
.macro  TestDecimalSet
    .byte $38
.endmacro
.macro  TestDecimalClear
    .byte $39
.endmacro
.macro  TestInterruptSet
    .byte $3a
.endmacro
.macro  TestInterruptClear
    .byte $3b
.endmacro
.macro  TestBreakSet
    .byte $3c
.endmacro
.macro  TestBreakClear
    .byte $3d
.endmacro

.macro TestAddress address, value
    .byte $80
    .addr address
    .byte value
.endmacro

; position 0x01 is the 'top' of the stack
.macro TestStack position, value
    .byte $88, position, value
.endmacro

; Test that the stack pointer has a particular value.
; Only used to test TXS, TSX
.macro TestStackPointer value
    .byte $89, value
.endmacro


; UTILITY MACROS

; Store status
.macro StStatus address, mask
    PHP ; push the status register
    PHA ; save current value of A
    PHP ; push the status register again to copy into A
    PLA ; pull the status register into A
    .ifnblank mask
    AND #mask ; mask the bits we want
    .else
    AND #%11000011 ; mask all but N V Z and C
    .endif
    STA address ; and, finally, store it

    PLA ; restore original value of A
    PLP ; restore the status register
.endmacro

; This overwrites X.
; FIXME Also, this does not work at the moment
; FIXME https://forums.atariage.com/topic/305708-saving-registers-in-a-subroutine/
.macro ClearMemory address, size, value
    .local loop
    .local end

    PHA ; save current value of A
    TXA
    PHA ; Save current value of X

    .ifnblank value
    LDA #value
    .else
    LDA #$00
    .endif
    LDX #$00       ; start at 0
loop:   
    STA address,X
    INX
    CPX #size
    BEQ end         ; bail out before size
    JMP loop

end:   
    PLA
    TAX ; restore original X
    PLA ; restore original A
    
.endmacro