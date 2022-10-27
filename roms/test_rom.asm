$CHARACTER_SPRITE_LENGTH 5

start: ; Address 0x200
    .assert_addr 0x200
    .name a = 1, b = 2, c = 3
    CALL reset

    JP exit
exit:
    JP exit
reset:
    CLS
    .name select_number = V0
    LD .select_number, 1
    LD F, .select_number
    .name x = V0, y = V1
    LD .x, 2
    LD .y, 4
    DRW .x, .y, $CHARACTER_SPRITE_LENGTH
    RET

data:
    .data 0, 1, 2, 3, 4, 5