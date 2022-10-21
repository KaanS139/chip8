use simplelog::{ColorChoice, ConfigBuilder, LevelFilter, TermLogger, TerminalMode};

fn main() {
    test();
    // TermLogger::init(
    //     LevelFilter::Trace,
    //     ConfigBuilder::new()
    //         .add_filter_allow_str("c8common")
    //         .add_filter_allow_str("c8int")
    //         .build(),
    //     TerminalMode::Stderr,
    //     ColorChoice::Always,
    // )
    // .expect("could not set up logging!");
    //
    // let int = c8int::Chip8Interpreter::new();
    //
    // chip8_base::run(int);
}

fn test() {
    let rom: c8common::asm::ROM = c8asm_proc::c8asm!(
        start: nop
            nop,
        begin_loop:
            cls,
            nop,
            jp @begin_loop
    );
    rom.save("roms/test_proc.ch8");
}
