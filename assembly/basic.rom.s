; A basic ROM for the 6502, consisting mainly of NMI/BRK/IRQ vectors
; TODO: Add interrupt handlers
    .setcpu "6502"
    .segment "OS"

reset:
    LDX #$ff
    TXS
    BRK

nmi:
    NOP   
irq:
    ; Save the state of the registers
    PHA
    TXA
    PHA
    TYA
    PHA

    ; restore the state of the registers
    PLA
    TAY
    PLA
    TAX
    PLA

    ; return
    RTI 

    .segment "VECTORS"

    .word nmi 
    .word reset
    .word irq
