.macro  Verify  address
    .byte $0f
    .addr address
.endmacro


.macro  TestStart id
    .byte $ff, id
.endmacro

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

.macro TestAddressContents address, value
    .byte $80
    .addr address
    .byte value
.endmacro

