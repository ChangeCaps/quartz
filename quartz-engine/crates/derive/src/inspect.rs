use proc_macro2::{Span, TokenStream};
use proc_macro_crate::FoundCrate;
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{
    parse::ParseStream, parse_macro_input, parse_quote, Attribute, Data, DeriveInput, Fields,
    GenericParam, Generics, Ident, Lit, LitStr, Meta, MetaNameValue,
};

#[derive(Default)]
struct InspectFieldAttributes {
    pub ignore: bool,
    pub collapsing: bool,
    pub doc_comments: Vec<String>,
}

impl InspectFieldAttributes {
    fn parse(attributes: &Vec<Attribute>) -> Self {
        let mut attrs = attributes
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
            });

        attrs.doc_comments = attributes
            .iter()
            .filter(|a| a.path.is_ident("doc"))
            .filter_map(|a| {
                if let Ok(Meta::NameValue(MetaNameValue {
                    lit: Lit::Str(s), ..
                })) = a.parse_meta()
                {
                    Some(s.value())
                } else {
                    None
                }
            })
            .collect();

        attrs
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

                        let tooltip = if attributes.doc_comments.len() > 0 {
                            let mut tooltip = String::new();

                            for doc_comment in &attributes.doc_comments[..attributes.doc_comments.len() - 1] {
                                tooltip.push_str(&doc_comment);
                                tooltip.push_str("\n");
                            }

                            tooltip.push_str(attributes.doc_comments.last().unwrap());

                            Some(quote!(
                                quartz_engine::core::egui::popup::show_tooltip_text(
                                    ui.ctx(), 
                                    quartz_engine::core::egui::Id::new(stringify!(#name)), 
                                    #tooltip
                                );
                            ))
                        } else {
                            None
                        };

                        if attributes.collapsing {
                            Some(quote_spanned! {f.ty.span()=>
                                let response = ui.collapsing(#name, |ui| {
                                    mutated |= self.#ident.inspect(ui);
                                });

                                if response.header_response.hovered() {
                                    #tooltip
                                }
                            })
                        } else {
                            Some(quote_spanned! {f.ty.span()=>
                                let response = ui.label(#name);
                                ui.indent(#name, |ui| {
                                    mutated |= self.#ident.inspect(ui);
                                });

                                if response.hovered() {
                                    #tooltip
                                }
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
