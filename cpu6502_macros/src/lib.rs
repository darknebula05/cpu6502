use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{quote, quote_spanned, ToTokens};
use syn::{
    parenthesized,
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
    Ident, LitInt, Result, Token,
};

struct Mode {
    name: Ident,
    opcode: LitInt,
    cycles: LitInt,
    span: Span,
    additional: Punctuated<Ident, Token![+]>,
}

impl Parse for Mode {
    fn parse(input: ParseStream) -> Result<Self> {
        let span = input.span();
        let content;
        parenthesized!(content in input);
        let name = content.parse()?;
        content.parse::<Token![,]>()?;
        let opcode = content.parse()?;
        content.parse::<Token![,]>()?;
        let cycles = content.parse()?;
        content.parse::<Option<Token![+]>>()?;
        let additional = content.parse_terminated(Ident::parse, Token![+])?;
        Ok(Self {
            name,
            opcode,
            cycles,
            span,
            additional,
        })
    }
}

struct Args {
    instructions: Punctuated<Instruction, Token![;]>,
}

impl Parse for Args {
    fn parse(input: ParseStream) -> Result<Self> {
        let vars = Punctuated::<Instruction, Token![;]>::parse_terminated(input)?;
        Ok(Args { instructions: vars })
    }
}

struct Instruction {
    name: Ident,
    modes: Vec<Mode>,
    span: Span,
}

impl Parse for Instruction {
    fn parse(input: ParseStream) -> Result<Self> {
        let name = input.parse::<Ident>()?;
        input.parse::<Token![:]>()?;
        let modes = Punctuated::<Mode, Token![,]>::parse_separated_nonempty(input)?
            .into_iter()
            .collect();
        Ok(Instruction {
            name,
            modes,
            span: input.span(),
        })
    }
}

#[proc_macro]
pub fn instructions(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as Args);
    let mut expanded = vec![];
    for inst in input.instructions {
        expanded.extend(generate_instruction(inst))
    }
    let quote = quote! {
        {
            let mut array: [(&str, fn(&mut Cpu6502) -> u8, &str); 256]  = [("", |a: &mut Cpu6502| 0, ""); 256];
            #(#expanded)*
            array
        }
    };
    println!("{}", quote);
    quote.into()
}

fn generate_instruction(instruction: Instruction) -> TokenStream2 {
    let name = &instruction.name;
    let name_str =
        Ident::new(&name.to_string().to_uppercase(), instruction.name.span()).to_string();
    let mut modes = vec![];
    for mode in instruction.modes {
        let addressing = mode.name;
        // let addressing_str = addressing.to_string();
        let opcode = mode.opcode;
        let additional = mode.additional.to_token_stream().to_string();
        let cycles = if additional.len() > 0 {
            mode.cycles.to_string() + "+" + &additional
        } else {
            mode.cycles.to_string() + &additional
        };
        let span = mode.span;
        modes.extend(quote_spanned!(span=>
            array[#opcode] = (#name_str, Self::#addressing, #cycles);
        ));
    }
    quote_spanned!(instruction.span=>
        #(#modes)*
    )
}
