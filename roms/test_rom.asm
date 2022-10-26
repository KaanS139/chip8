$CHARACTER_SPRITE_LENGTH 5
start: ; Address 0x200
    CALL reset

    JP exit
exit:
    JP exit
reset:
    CLS
    .set select_number = V0@
    LD .select_number, 1
    LD F, .select_number
    .set x, y = V0, V1
    LD .x, 2
    LD .y, 4
    DRW .x, .y, $CHARACTER_SPRITE_LENGTH
    RET
