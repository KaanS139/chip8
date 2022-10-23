use c8common::control::ControlledToInterpreter;
use simplelog::{ColorChoice, ConfigBuilder, LevelFilter, TermLogger, TerminalMode};

fn main() {
    // test();
    TermLogger::init(
        LevelFilter::Trace,
        ConfigBuilder::new()
            .add_filter_allow_str("c8common")
            .add_filter_allow_str("c8int")
            .add_filter_allow_str("chip8-base")
            .build(),
        TerminalMode::Stderr,
        ColorChoice::Always,
    )
    .expect("could not set up logging!");

    let int = c8int::Chip8Interpreter::new_from_file("roms/Heart Monitor.ch8");
    // let int = c8int::Chip8Interpreter::new_assembled_save("test_rng.ch8", |asm| {
    //     asm
    //         .rng(GeneralRegister::V0, 0xFF)
    //         .label_str("end")
    //         .jump("end")
    // });
    //
    // int.memory().save(std::fs::File::create("roms/test_rng.mem").unwrap());

    chip8_base::run(int.to_interpreter().with_frequency(1024));
}
