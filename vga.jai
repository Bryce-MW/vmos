ROWS :: 25;
COLS :: 80;
row := 0;
col := 0;
vga_buf: *u16: xx 0xb8000;

colors :: u8.[0x0e, 0x0f, 0x0e];

clear_vga :: () {
    row = 0;
    col = 0;
    clear(vga_buf, ROWS*COLS);
}

#program_export "write_impl" write :: (fd: s32, buf: *u8, len: u64) -> s64 #c_call {
    ret := cast(s64)len;
    while len != 0 {
        len -= 1;

        char := cast(u16) <<buf;
        if char == #char "\n" {
            col = 0;
            row += 1;
            if row == ROWS {
                row -= 1;
                copy(vga_buf + COLS, vga_buf, COLS * (ROWS - 1));
                clear(vga_buf + COLS * (ROWS - 1), COLS);
            }
        } else {
            if char < 0x20 || char > 0x7E {
                char = 0xA8;
            }
            char |= (cast(u16) colors[fd]) << 8;
            <<(vga_buf + col + row * COLS) = char;
            col += 1;
            if col == COLS {
                col = 0;
                row += 1;
            }
            if row == ROWS {
                copy(vga_buf + COLS, vga_buf, COLS * (ROWS - 1));
                clear(vga_buf + COLS * (ROWS - 1), COLS);
            }
        }

        buf += 1;
    }
    return ret;
} @StubImpl
