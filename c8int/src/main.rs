#![deny(missing_debug_implementations, unused_must_use)]
#![warn(missing_copy_implementations)]

use c8common::control::ControlledToInterpreter;
use clap::Parser;
use simplelog::{ColorChoice, ConfigBuilder, LevelFilter, TermLogger, TerminalMode};
use std::str::FromStr;

#[derive(Parser, Debug)]
struct Args {
    rom_path: String,
    #[arg(short = 'f', long = "frequency", default_value_t = 512)]
    frequency: u32,
    #[arg(long = "frequency-scale")]
    frequency_scale: Option<f32>,
    #[arg(long = "log", value_parser = <LevelFilter as FromStr>::from_str, default_value_t = LevelFilter::Trace)]
    log_level: LevelFilter,
}

fn main() {
    let Args {
        rom_path,
        frequency,
        frequency_scale: simulated_frequency,
        log_level,
    } = Args::parse();

    TermLogger::init(
        log_level,
        ConfigBuilder::new()
            .add_filter_allow_str("c8common")
            .add_filter_allow_str("c8int")
            .add_filter_allow_str("chip8-base")
            .build(),
        TerminalMode::Stderr,
        ColorChoice::Always,
    )
    .expect("could not set up logging!");

    let int = c8int::Chip8Interpreter::new_from_file(rom_path);
    // let int = c8int::Chip8Interpreter::new_assembled_save("test_rng.ch8", |asm| {
    //     asm
    //         .rng(GeneralRegister::V0, 0xFF)
    //         .label_str("end")
    //         .jump("end")
    // });
    //
    // int.memory().save(std::fs::File::create("roms/test_rng.mem").unwrap());

    chip8_base::run(
        int.to_interpreter()
            .with_frequency(frequency)
            .with_simulated_frequency(simulated_frequency),
    );
}
