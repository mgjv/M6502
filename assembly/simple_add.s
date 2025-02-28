LDA #$ff
ADC #$02     ; expected content of A: #01, carry flag set()
STA $0402
BRK