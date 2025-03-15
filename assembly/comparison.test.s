; Comparison operation tests
; CMP, CPX, CPY
;
; See http://www.6502.org/tutorials/compare_instructions.html
; for comprehensive explanation of logic

.include "test.inc"

NZCmask .set %10000011

; TODO Address mode tests for at least one of these

; CMP: A > memory, pos result -> C set
    LDA #$01
    STA r1
    LDA #$02
    CMP r1
    STA a1                  ; store accumulator for check
    StStatus s1, NZCmask

; CMP: A > memory, neg result -> N and C set
    LDA #$ff
    CMP r1                  ; r1 should still have $01 in it
    StStatus s2, NZCmask

; CMP: A < memory pos result -> none set
    LDA #$ff
    STA r2
    LDA #$01
    CMP r2
    StStatus s3, NZCmask

; CMP: A < memory neg result -> N set
    LDA #$02
    STA r3
    LDA #$01
    CMP r3
    StStatus s4, NZCmask

; CMP: A == memory -> Z, C set
    LDA #$ff
    STA r4
    CMP r4
    StStatus s5, NZCmask

    VRFY    :+
    JMP     :++

:   TestStart   $01
    TestAddress s1, $01 ; C set
    TestAddress a1, $02 ; should be unaffected
    TestAddress r1, $01 ; should be unaffected
    TestAddress s2, $81 ; N and C set
    TestAddress s3, $00 ; none of the flags should be set
    TestAddress s4, $80 ; Just N set
    TestAddress s5, $03 ; Z, C set
    TestEnd

; CPX: X > memory, pos result -> C set
:   LDX #$01
    STX r1
    LDX #$02
    CPX r1
    STX a1                  ; store X for check
    StStatus s1, NZCmask

; CPX: X > memory, neg result -> N and C set
    LDX #$ff
    CPX r1                  ; r1 should still have $01 in it
    StStatus s2, NZCmask

; CPX: X < memory pos result -> none set
    LDX #$ff
    STX r2
    LDX #$01
    CPX r2
    StStatus s3, NZCmask

; CPX: X < memory neg result -> N set
    LDX #$02
    STX r3
    LDX #$01
    CPX r3
    StStatus s4, NZCmask

; CPX: X == memory -> Z, C set
    LDX #$ff
    STX r4
    CPX r4
    StStatus s5, NZCmask

    VRFY    :+
    JMP     :++

:   TestStart   $01
    TestAddress s1, $01 ; C set
    TestAddress a1, $02 ; should be unaffected
    TestAddress r1, $01 ; should be unaffected
    TestAddress s2, $81 ; N and C set
    TestAddress s3, $00 ; none of the flags should be set
    TestAddress s4, $80 ; Just N set
    TestAddress s5, $03 ; Z, C set
    TestEnd


; CPY: Y > memory, pos result -> C set
:   LDY #$01
    STY r1
    LDY #$02
    CPY r1
    STY a1                  ; store X for check
    StStatus s1, NZCmask

; CPY: Y > memory, neg result -> N and C set
    LDY #$ff
    CPY r1                  ; r1 should still have $01 in it
    StStatus s2, NZCmask

; CPY: Y < memory pos result -> none set
    LDY #$ff
    STY r2
    LDY #$01
    CPY r2
    StStatus s3, NZCmask

; CPY: Y < memory neg result -> N set
    LDY #$02
    STY r3
    LDY #$01
    CPY r3
    StStatus s4, NZCmask

; CPY: Y == memory -> Z, C set
    LDY #$ff
    STY r4
    CPY r4
    StStatus s5, NZCmask

    VRFY    :+
    JMP     :++

:   TestStart   $01
    TestAddress s1, $01 ; C set
    TestAddress a1, $02 ; should be unaffected
    TestAddress r1, $01 ; should be unaffected
    TestAddress s2, $81 ; N and C set
    TestAddress s3, $00 ; none of the flags should be set
    TestAddress s4, $80 ; Just N set
    TestAddress s5, $03 ; Z, C set
    TestEnd

; End of all tests
:   HALT

.data
    ; Some result variables to provide work memory
    r1: .byte $de
    r2: .byte $ad
    r3: .byte $be
    r4: .byte $af
    ; Some variables to store stack status in
    s1: .byte $de
    s2: .byte $ad
    s3: .byte $be
    s4: .byte $ef
    s5: .byte $ee
    ; Some variables to store accumulator in
    a1: .byte $de
    a2: .byte $ad
    a3: .byte $be
    a4: .byte $ef