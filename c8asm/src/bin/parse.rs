use c8asm::parsing::parse;
use c8asm::tokenizing::tokenize;

fn main() -> miette::Result<()> {
    miette::set_hook(Box::new(|_| {
        Box::new(
            miette::MietteHandlerOpts::new()
                .terminal_links(true)
                .context_lines(1)
                .tab_width(4)
                .build(),
        )
    }))?;
    let contents = std::fs::read_to_string("roms/test_rom.asm").expect("failed to read .asm file!");
    let tokens = tokenize(&contents)
        .map_err(|error| miette::Error::new(error).with_source_code(contents.clone()))?;
    let parts =
        parse(tokens).map_err(|error| miette::Error::new(error).with_source_code(contents))?;

    drop(dbg!(parts));

    Ok(())
}
