use c8common::asm::parsing;

fn main() {
    let contents = std::fs::read_to_string("roms/test_rom.asm").expect("failed to read .asm file!");
    let tokens = parsing::tokenize(&contents).unwrap();
    dbg!(tokens);
}
