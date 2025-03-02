LDA #$ff
ADC #$03     ; expected content of A: #02, carry flag set()
STA $0030
BRK