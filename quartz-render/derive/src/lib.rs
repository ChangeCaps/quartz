mod uniform;

use proc_macro::TokenStream;
use proc_macro_crate::{crate_name, FoundCrate, Error};

fn crate_path() -> proc_macro2::TokenStream {
    if let Ok(found) = crate_name("quartz-render") {
        match found {
            FoundCrate::Itself => quote::quote!(crate),
            FoundCrate::Name(name) => {
                let ident = syn::Ident::new(&name, proc_macro2::Span::call_site());
                quote::quote!(#ident)
            }
        }
    } else {
        match crate_name("quartz-engine").unwrap() {
            FoundCrate::Itself => quote::quote!(crate),
            FoundCrate::Name(name) => {
                let ident = syn::Ident::new(&name, proc_macro2::Span::call_site());
                quote::quote!(#ident::render)
            }
        }
    }
}

#[proc_macro_derive(Uniform, attributes(uniform))]
pub fn derive_uniform_object(input: TokenStream) -> TokenStream {
    uniform::derive_uniform(input)
}