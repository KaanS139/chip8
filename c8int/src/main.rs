use c8int;
use chip8_base;
use simplelog::{ColorChoice, ConfigBuilder, LevelFilter, TermLogger, TerminalMode};

fn main() {
    TermLogger::init(
        LevelFilter::Trace,
        ConfigBuilder::new()
            .add_filter_allow_str("c8common")
            .add_filter_allow_str("c8int")
            .build(),
        TerminalMode::Stderr,
        ColorChoice::Always,
    )
    .expect("could not set up logging!");

    let int = c8int::Chip8Interpreter::new();

    chip8_base::run(int);
}
