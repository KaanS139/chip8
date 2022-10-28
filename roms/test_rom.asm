$CHARACTER_SPRITE_LENGTH 5

$KEY_X 0x0
$KEY_1 0x1
$KEY_2 0x2
$KEY_3 0x3
$KEY_Q 0x4
$KEY_W 0x5
$KEY_E 0x6
$KEY_A 0x7
$KEY_S 0x8
$KEY_D 0x9
$KEY_Z 0xA
$KEY_C 0xB
$KEY_4 0xC
$KEY_R 0xD
$KEY_F 0xE
$KEY_V 0xF

.assert_addr 0x200
CALL entry
exit:
    JP exit

static:
    image_1:  .data 0b00000011
              .data 0b00000111
              .data 0b00000111
              .data 0b00000111
              .data 0b00000011
              .data 0b01110000
              .data 0b11111000
              .data 0b11111000
              .data 0b11111000
              .data 0b01110000
              .data 0b00000011
              .data 0b00000111
              .data 0b00000111
              .data 0b00000111
              .data 0b00000011
    image_2:  .data 0b10000000
              .data 0b11000000
              .data 0b11000000
              .data 0b11000000
              .data 0b10000000
              .data 0b00000000
              .data 0b00000000
              .data 0b00000000
              .data 0b00000000
              .data 0b00000000
              .data 0b10000000
              .data 0b11000000
              .data 0b11000000
              .data 0b11000000
              .data 0b10000000
    image_3:  .data 0b01110
              .data 0b11111
              .data 0b11111
              .data 0b11111
              .data 0b01110
    image_uw: .data 0b11000011
              .data 0b11000011
              .data 0b11000011
              .data 0b11000011
              .data 0b11000011
              .data 0b11000011
              .data 0b11100111
              .data 0b11111111
              .data 0b01111110
    image_w1: .data 0b11100011
              .data 0b01100011
              .data 0b01110011
              .data 0b01110111
              .data 0b00110110
              .data 0b00111110
              .data 0b00111100
              .data 0b00011000
              .data 0b00011000
    image_w2: .data 0b1000111
              .data 0b1000110
              .data 0b1001110
              .data 0b1101110
              .data 0b1101100
              .data 0b1111100
              .data 0b0111100
              .data 0b0011000
              .data 0b0011000
    image_c:  .data 0b00111110
              .data 0b01111111
              .data 0b11100011
              .data 0b11000000
              .data 0b11000000
              .data 0b11000000
              .data 0b11100011
              .data 0b01111111
              .data 0b00111110
    image_s:  .data 0b0111110
              .data 0b1111111
              .data 0b1100011
              .data 0b1111000
              .data 0b0111110
              .data 0b0001111
              .data 0b1100011
              .data 0b1111111
              .data 0b0111110

.name is_on_screen = VC
entry:
    .name key_storage = VE
    CALL clear
    loop:
    CALL draw_uwcs_logo
    LD VD, 60
    LD DT, VD
    wait_loop:
    LD .key_storage, $KEY_C
    SKNP .key_storage
    JP final
    LD VD, DT
    SE VD, 0
    JP wait_loop

    JP loop

    final:
    SE .is_on_screen, 1
    CALL draw_uwcs_logo
    RET

draw_uwcs_logo:
    .name sprite_x = VA, sprite_y = VB

    LD .sprite_x, 3
    LD .sprite_y, 6
    LD I, image_1
    DRW .sprite_x, .sprite_y, 15

    LD I, image_2
    ADD .sprite_x, 8
    DRW .sprite_x, .sprite_y, 15

    LD I, image_3
    LD .sprite_x, 10
    LD .sprite_y, 21
    DRW .sprite_x, .sprite_y, 5

    LD I, image_uw
    LD .sprite_x, 20
    LD .sprite_y, 11
    DRW .sprite_x, .sprite_y, 9

    LD I, image_w1
    LD .sprite_x, 29
    LD .sprite_y, 11
    DRW .sprite_x, .sprite_y, 9

    LD I, image_w2
    LD .sprite_x, 36
    LD .sprite_y, 11
    DRW .sprite_x, .sprite_y, 9

    LD I, image_c
    LD .sprite_x, 45
    LD .sprite_y, 11
    DRW .sprite_x, .sprite_y, 9

    LD I, image_s
    LD .sprite_x, 54
    LD .sprite_y, 11
    DRW .sprite_x, .sprite_y, 9

    LD VE, 1
    XOR .is_on_screen, VE
    RET

clear:
    CLS
    RET

working_memory:
