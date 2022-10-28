$CHARACTER_SPRITE_LENGTH 5

.name sprite_x = VA, sprite_y = VB

.assert_addr 0x200
CALL entry
exit:
    JP exit

static:

entry:
    CALL clear
    LD I, working_memory
    .name to_bcd = V0
    LD .to_bcd, 123
    LD B, .to_bcd
    .name h = V0, t = V1, u = V2
    LD V2, I

    LD .sprite_x, 5
    LD .sprite_y, 5
    LD F, .h
    DRW .sprite_x, .sprite_y, $CHARACTER_SPRITE_LENGTH
    ADD .sprite_x, 6
    LD F, .t
    DRW .sprite_x, .sprite_y, $CHARACTER_SPRITE_LENGTH
    ADD .sprite_x, 6
    LD F, .u
    DRW .sprite_x, .sprite_y, $CHARACTER_SPRITE_LENGTH

    RET

clear:
    CLS
    RET

working_memory:
