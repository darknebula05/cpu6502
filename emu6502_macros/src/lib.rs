use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput};

#[proc_macro_derive(AddressingMode)]
pub fn derive_addressing_mode(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let out = debug_enum(&input.data);
    quote!().into()
}

fn debug_enum(data: &Data) -> TokenStream2 {
    match *data {
        Data::Struct(ref data) => {
            println!("{:?}", data);
            quote!()
        }
        Data::Enum(ref data) => {
            println!("{data:?}");
            quote!()
        }
        Data::Union(_) => todo!(),
    }
}
