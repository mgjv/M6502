; This is a pseudo-instruction that will cause the computer to shut down
.macro HALT
    .byte $fc
.endmacro

.macro  Verify  address
    .byte $fb
    .addr address
.endmacro

.macro  TestStart id
    .byte $c0, id
.endmacro

; Use the most likely "wrong" byte value for the end marker
.macro  TestEnd
    .byte $00
.endmacro

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

.macro TestAddress address, value
    .byte $80
    .addr address
    .byte value
.endmacro

; This overwrites X.
; FIXME Also, this does not work at the moment
; FIXME https://forums.atariage.com/topic/305708-saving-registers-in-a-subroutine/
.macro ClearMemory address, size
    ;PHX
;     PHA
; :   LDX size
;     LDA #$00
;     STA address, X
;     DEX
;     BEQ :+
;     JMP :-
; :   PLA
    ;PLX
.endmacro