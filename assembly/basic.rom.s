; A basic ROM for the 6502, consisting mainly of NMI/BRK/IRQ vectors
; TODO: Add interrupt handlers
    .setcpu "6502"
    .segment "OS"

reset:
    LDX #$ff
    TXS

nmi:
    BRK

irq:
    BRK

    .segment "VECTORS"

    .word nmi 
    .word reset
    .word irq
