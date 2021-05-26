use proc_macro2::{Span, TokenStream};
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{parse_macro_input, parse_quote, Data, DataStruct, DeriveInput, Fields};

pub fn derive_uniform(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let crate_path = crate::crate_path();

    let ident = input.ident;

    let generics = input.generics;
    let size = size(&crate_path, &input.data);
    let data = data(&crate_path, &input.data);
    let (impl_generics, type_generics, where_clause) = generics.split_for_impl();

    let expanded = quote! {
        impl #impl_generics #crate_path::render::uniform::Uniform
            for #ident #type_generics #where_clause
        {
            fn alignment() -> u64 {
                16
            }

            fn size() -> u64 {
                #size
            }

            fn data(&self) -> Vec<u8> {
                #data
            }
        }
    };

    proc_macro::TokenStream::from(expanded)
}

fn size(crate_path: &TokenStream, data: &Data) -> TokenStream {
    match data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => {
                let recurse = fields.named.iter().map(|f| {
                    let ty = &f.ty;

                    quote_spanned! {f.ident.as_ref().unwrap().span()=>
                        #crate_path::render::uniform::aligned_size(
                            <#ty as #crate_path::render::uniform::Uniform>::size(),
                            <#ty as #crate_path::render::uniform::Uniform>::alignment()
                        )
                    }
                });

                quote! {
                    0 #( + #recurse )*
                }
            }
            _ => unimplemented!(),
        },
        _ => unimplemented!(),
    }
}

fn data(crate_path: &TokenStream, data: &Data) -> TokenStream {
    match data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => {
                let recurse = fields.named.iter().map(|f| {
                    let ident = &f.ident.as_ref().unwrap();
                    let ty = &f.ty;

                    quote_spanned! {ident.span()=>
                        #crate_path::render::uniform::append_aligned(&mut data, &self.#ident, #ty::alignment());
                    }
                });

                quote! {
                    let mut data = Vec::new();

                    #(
                        #recurse
                    )*

                    data
                }
            }
            _ => unimplemented!(),
        },
        _ => unimplemented!(),
    }
}
