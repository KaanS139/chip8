use proc_macro::{Delimiter, Group, Ident, Literal, Punct, Spacing, Span, TokenStream, TokenTree};
use tap::prelude::*;

#[proc_macro]
pub fn c8asm(_item: TokenStream) -> TokenStream {
    let mut add_instructions = TokenStream::new();
    add_instructions.extend(Some(TokenTree::Literal(Literal::string("Who what how?"))));

    // TODO: Use the parser to generate the instructions, and then map those to tokens here

    let mut inner_stream = TokenStream::new();
    inner_stream.extend(vec![
        basic_ident("use"),
        basic_ident("c8common"),
        path_separator(),
        basic_ident("asm"),
        semicolon(),
        basic_ident("let"),
        basic_ident("mut"),
        basic_ident("assembler"),
        equals(),
        basic_ident("asm"),
        path_separator(),
        basic_ident("Assembler"),
        path_separator(),
        basic_ident("new"),
        arguments(|_| {}),
        semicolon(),
    ]);

    inner_stream.extend(add_instructions);

    inner_stream.extend(Some(semicolon()));

    inner_stream.extend(vec![
        basic_ident("assembler"),
        dot(),
        basic_ident("assemble"),
        arguments(|_| {}),
    ]);

    TokenTree::Group(Group::new(Delimiter::Brace, inner_stream)).into()
}

fn basic_ident(ident: &str) -> TokenTree {
    Ident::new(ident, Span::mixed_site()).into()
}

fn semicolon() -> TokenTree {
    Punct::new(';', Spacing::Alone).into()
}

fn dot() -> TokenTree {
    Punct::new('.', Spacing::Alone).into()
}

fn path_separator() -> TokenTree {
    Group::new(
        Delimiter::None,
        TokenStream::new().tap_mut(|ts| {
            ts.extend([
                Punct::new(':', Spacing::Joint).conv::<TokenTree>(),
                Punct::new(':', Spacing::Joint).conv::<TokenTree>(),
            ])
        }),
    )
    .into()
}

fn equals() -> TokenTree {
    Punct::new('=', Spacing::Alone).into()
}

fn arguments(with: impl FnOnce(&mut TokenStream)) -> TokenTree {
    Group::new(Delimiter::Parenthesis, TokenStream::new().tap_mut(with)).into()
}

// fn test() {
//     let rom: c8common::asm::ROM = {
//         use c8common::asm;
//         let mut assembler = asm::Assembler::new();
//
//         assembler.assemble()
//     };
//
//     rom.save("roms/test_proc.ch8");
// }
