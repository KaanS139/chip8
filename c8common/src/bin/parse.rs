use c8common::asm::{conversion::convert, tokenizing::tokenize};

fn main() {
    let contents = std::fs::read_to_string("roms/test_rom.asm").expect("failed to read .asm file!");
    let tokens = tokenize(&contents).unwrap();
    let parts = dbg!(convert(tokens)).unwrap();

    drop(parts);
}
