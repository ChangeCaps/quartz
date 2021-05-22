mod inspect;
mod reflect;

use proc_macro::TokenStream;

#[proc_macro_derive(Reflect, attributes(reflect))]
pub fn derive_reflect(input: TokenStream) -> TokenStream {
    reflect::derive_reflect(input)
}

#[proc_macro_derive(Inspect, attributes(inspect))]
pub fn derive_inspect(input: TokenStream) -> TokenStream {
    inspect::derive_inspect(input)
}
