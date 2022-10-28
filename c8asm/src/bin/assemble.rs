use std::error::Error;
use clap::Parser;
use log::{info, LevelFilter};
use simplelog::{ColorChoice, ConfigBuilder, TerminalMode, TermLogger};
use c8asm::compilation::compile;
use c8asm::instruction_sets::Chip8InstructionSet;
use c8asm::parsing::parse;
use c8asm::tokenizing::tokenize;
use std::str::FromStr;

#[derive(Parser, Debug)]
struct Args {
    asm_path: String,
    out_path: String,
    #[arg(long = "log", value_parser = <LevelFilter as FromStr>::from_str, default_value_t = LevelFilter::Info)]
    log_level: LevelFilter,
}

fn main() -> Result<(), Box<dyn Error>> {
    let Args { asm_path, out_path, log_level } = Args::parse();

    TermLogger::init(
        log_level,
        ConfigBuilder::new()
            .add_filter_allow_str("c8asm")
            .add_filter_allow_str("assemble")
            .add_filter_allow_str("c8common")
            .add_filter_allow_str("c8hooks")
            .add_filter_allow_str("c8int")
            .add_filter_allow_str("c8runner")
            .build(),
        TerminalMode::Stderr,
        ColorChoice::Always,
    ).expect("could not set up logging!");

    miette::set_hook(Box::new(|_| {
        Box::new(
            miette::MietteHandlerOpts::new()
                .terminal_links(true)
                .context_lines(1)
                .tab_width(4)
                .build(),
        )
    }))?;

    let contents = std::fs::read_to_string(asm_path)?;
    info!("Read file contents");
    let tokens = tokenize(&contents).map_err(|error| miette::Error::new(error).with_source_code(contents.clone()))?;
    info!("Tokenized");
    let parts = parse(tokens).map_err(|error| miette::Error::new(error).with_source_code(contents.clone()))?;
    info!("Parsed");
    let rom = compile::<Chip8InstructionSet>(parts).map_err(|error| miette::Error::new(error).with_source_code(contents))?;
    info!("Compiled");
    rom.save(out_path)?;
    info!("Saved, OK");
    Ok(())
}
