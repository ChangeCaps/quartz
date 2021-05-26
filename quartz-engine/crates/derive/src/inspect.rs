use proc_macro2::{Span, TokenStream};
use proc_macro_crate::FoundCrate;
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{
    parse::ParseStream, parse_macro_input, parse_quote, Attribute, Data, DeriveInput, Fields,
    GenericParam, Generics, Ident,
};

#[derive(Default)]
struct InspectFieldAttributes {
    pub ignore: bool,
    pub collapsing: bool,
}

impl InspectFieldAttributes {
    fn parse(attributes: &Vec<Attribute>) -> Self {
        attributes
            .iter()
            .find(|a| *a.path.get_ident().as_ref().unwrap() == INSPECT_ATTRIBUTE_NAME)
            .map_or_else(Self::default, |a| {
                syn::custom_keyword!(ignore);
                syn::custom_keyword!(collapsing);
                let mut attributes = Self::default();

                a.parse_args_with(|input: ParseStream| {
                    if input.parse::<Option<ignore>>()?.is_some() {
                        attributes.ignore = true;
                    }

                    if input.parse::<Option<collapsing>>()?.is_some() {
                        attributes.collapsing = true;
                    }

                    Ok(())
                })
                .expect("Invalid 'reflect' attribute format.");

                attributes
            })
    }
}

const INSPECT_ATTRIBUTE_NAME: &str = "inspect";

pub fn derive_inspect(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;

    let generics = add_trait_bounds(input.generics);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let inspect = inspect(&input.data);

    let expanded = quote! {
        impl #impl_generics quartz_engine::core::inspect::Inspect for #name #ty_generics #where_clause {
            fn inspect(&mut self, ui: &mut quartz_engine::core::egui::Ui) -> bool {
                #inspect
            }
        }
    };

    proc_macro::TokenStream::from(expanded)
}

fn add_trait_bounds(mut generics: Generics) -> Generics {
    for param in &mut generics.params {
        if let GenericParam::Type(type_param) = param {
            type_param
                .bounds
                .push(parse_quote!(quartz_engine::core::inspect::Inspect));
        }
    }

    generics
}

fn inspect(data: &Data) -> TokenStream {
    match data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => {
                let fields = fields.named.iter().filter_map(|f| {
                    let attributes = InspectFieldAttributes::parse(&f.attrs);

                    if attributes.ignore {
                        None
                    } else {
                        let ident = f.ident.as_ref().unwrap();
                        let name = ident.to_string();

                        if attributes.collapsing {
                            Some(quote_spanned! {f.ty.span()=>
                                ui.collapsing(#name, |ui| {
                                    mutated |= self.#ident.inspect(ui);
                                });
                            })
                        } else {
                            Some(quote_spanned! {f.ty.span()=>
                                ui.label(#name);
                                ui.indent(#name, |ui| {
                                    mutated |= self.#ident.inspect(ui);
                                });
                            })
                        }
                    }
                });

                quote! {
                    ui.vertical(|ui| {
                        let mut mutated = false;

                        #(
                            #fields
                        )*

                        mutated
                    }).inner
                }
            }
            _ => unimplemented!(),
        },
        _ => unimplemented!(),
    }
}
