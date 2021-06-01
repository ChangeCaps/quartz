use proc_macro2::{Span, TokenStream};
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{
    parse::ParseStream, parse_macro_input, parse_quote, Attribute, Data, DeriveInput, Fields,
    GenericParam, Generics, Ident,
};

#[derive(Default)]
struct ReflectFieldAttributes {
    pub ignore: bool,
    pub reflect: bool,
}

impl ReflectFieldAttributes {
    fn parse(attributes: &Vec<Attribute>) -> Self {
        attributes
            .iter()
            .find(|a| *a.path.get_ident().as_ref().unwrap() == REFLECT_ATTRIBUTE_NAME)
            .map_or_else(Self::default, |a| {
                syn::custom_keyword!(ignore);
                syn::custom_keyword!(reflect);

                let mut attributes = Self::default();
                a.parse_args_with(|input: ParseStream| {
                    if input.parse::<Option<ignore>>()?.is_some() {
                        attributes.ignore = true;
                    }

                    if input.parse::<Option<reflect>>()?.is_some() {
                        attributes.reflect = true;
                    }

                    Ok(())
                })
                .expect("Invalid 'reflect' attribute format.");

                attributes
            })
    }
}

const REFLECT_ATTRIBUTE_NAME: &str = "reflect";

pub fn derive_reflect(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;

    let generics = add_trait_bounds(input.generics);

    let reflect = reflect(&name, &generics, &input.data);
    let serialize = serialize(&name, &input.data);

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let expanded = quote! {
        impl #impl_generics quartz_engine::core::serde::Serialize for #name #ty_generics #where_clause {
            fn serialize<S: quartz_engine::core::serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                #serialize
            }
        }

        impl #impl_generics quartz_engine::core::reflect::Reflect for #name #ty_generics #where_clause {
            fn reflect(&mut self, deserializer: &mut dyn quartz_engine::core::erased_serde::Deserializer) {
                #reflect
            }

            fn as_serialize(&self) -> &dyn quartz_engine::core::erased_serde::Serialize {
                self
            }

            fn short_name_const() -> &'static str {
                stringify!(#name)
            }
        }
    };

    proc_macro::TokenStream::from(expanded)
}

fn add_trait_bounds(mut generics: Generics) -> Generics {
    for param in &mut generics.params {
        if let GenericParam::Type(type_param) = param {
            type_param.bounds.push(parse_quote!(
                quartz_engine::core::serde::de::DeserializeOwned
            ));

            type_param
                .bounds
                .push(parse_quote!(quartz_engine::core::serde::Serialize));
        }
    }

    generics
}

fn add_de_lifetime(mut generics: Generics) -> Generics {
    generics.params.insert(0, parse_quote!('de));

    generics
}

fn reflect(ident: &Ident, generics: &Generics, data: &Data) -> TokenStream {
    let name = ident.to_string();
    match data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => {
                let fields = fields
                    .named
                    .iter()
                    .filter_map(|f| {
                        let attrs = ReflectFieldAttributes::parse(&f.attrs);

                        if attrs.ignore {
                            None
                        } else {
                            Some(f)
                        }
                    })
                    .collect::<Vec<_>>();

                let len = fields.len();

                let idents = fields.iter().map(|f| f.ident.as_ref().unwrap());

                let names = fields.iter().map(|f| f.ident.as_ref().unwrap().to_string());

                let expecting = fields.iter().fold(String::new(), |e, f| {
                    let ident = f.ident.as_ref().unwrap();

                    if e.is_empty() {
                        format!("'{}'", ident)
                    } else {
                        format!("{} or '{}'", e, ident)
                    }
                });

                let field = {
                    let idents = idents.clone();

                    quote! {
                        #[allow(non_camel_case_types)]
                        enum Field { #( #idents ),* }
                    }
                };

                let field_match = {
                    let names = names.clone();
                    let idents = idents.clone();

                    quote! {
                        match value {
                            #(
                                #names => Ok(Field::#idents),
                            )*
                            _ => Err(quartz_engine::core::serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                };

                
                let visit_seq = {
                    let mut index = 0usize;

                    let fields_seq = fields.iter().map(|f| {
                        index += 1;

                        let attrs = ReflectFieldAttributes::parse(&f.attrs);
                        let ident = f.ident.as_ref().unwrap();
    
                        if attrs.reflect {
                            quote! {
                                seq.next_element_seed(
                                    quartz_engine::core::reflect::ReflectDeserializer {
                                        reflect: &mut self.#ident,
                                    }
                                )?.ok_or(Error::invalid_length(#index, &self))?;
                            }
                        } else {
                            quote! {
                                self.#ident = seq.next_element()?.ok_or(Error::invalid_length(#index, &self))?;
                            }
                        }
                    });

                    quote! {
                        fn visit_seq<V>(self, mut seq: V) -> Result<(), V::Error>
                        where
                            V: quartz_engine::core::serde::de::SeqAccess<'de>,
                        {
                            use quartz_engine::core::serde::de::{SeqAccess, Error};

                            #(
                                #fields_seq
                            )*

                            Ok(())
                        }
                    }
                };

                let fields_map = fields.iter().map(|f| {
                    let attrs = ReflectFieldAttributes::parse(&f.attrs);
                    let ident = f.ident.as_ref().unwrap();

                    if attrs.reflect {
                        quote!(
                            Field::#ident => {
                                map.next_value_seed(
                                    quartz_engine::core::reflect::ReflectDeserializer {
                                        reflect: &mut self.#ident,
                                    }
                                )?;
                            }
                        )
                    } else {
                        quote!(
                            Field::#ident => {
                                self.#ident = map.next_value()?;
                            }
                        )
                    }
                });

                let type_params = generics.type_params().map(|t| &t.ident);

                let type_params = quote!(#(#type_params)*);

                let visitor_generics = add_de_lifetime(generics.clone());
                let (impl_generics, _, where_clause) = visitor_generics.split_for_impl();

                if len > 0 {
                    quote! {
                        #field
                        use quartz_engine::core::serde::Deserializer;

                        impl<'de> quartz_engine::core::serde::Deserialize<'de> for Field {
                            fn deserialize<D>(deserializer: D) -> Result<Field, D::Error>
                            where
                                D: quartz_engine::core::serde::Deserializer<'de>,
                            {
                                use quartz_engine::core::serde::Deserializer;

                                struct FieldVisitor;

                                impl<'a, 'de> quartz_engine::core::serde::de::Visitor<'de> for FieldVisitor {
                                    type Value = Field;

                                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                                        formatter.write_str(#expecting)?;

                                        Ok(())
                                    }

                                    fn visit_str<E>(self, value: &str) -> Result<Field, E>
                                    where
                                        E: quartz_engine::core::serde::de::Error,
                                    {
                                        #field_match
                                    }
                                }

                                deserializer.deserialize_identifier(FieldVisitor)
                            }
                        }

                        impl #impl_generics quartz_engine::core::serde::de::Visitor<'de> for
                            &mut #ident <#type_params> #where_clause
                        {
                            type Value = ();

                            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                                formatter.write_str(&format!("struct {}", #name))?;

                                Ok(())
                            }

                            #visit_seq

                            fn visit_map<V>(self, mut map: V) -> Result<(), V::Error>
                            where
                                V: quartz_engine::core::serde::de::MapAccess<'de>,
                            {
                                use quartz_engine::core::serde::de::MapAccess;

                                while let Some(key) = map.next_key::<Field>()? {
                                    match key {
                                        #(
                                            #fields_map
                                        ),*
                                    }
                                }

                                Ok(())
                            }
                        }

                        const FIELDS: &[&str] = &[#(#names),*];
                        deserializer.deserialize_struct(#name, FIELDS, self).unwrap();
                    }
                } else {
                    quote! {
                        use quartz_engine::core::serde::Deserializer;
                        deserializer.deserialize_unit_struct(
                            #name,
                            quartz_engine::core::serde::de::IgnoredAny
                        ).unwrap();
                    }
                }
            }
            _ => unimplemented!(),
        },
        _ => unimplemented!(),
    }
}

fn serialize(ident: &Ident, data: &Data) -> TokenStream {
    let name = ident.to_string();

    match data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => {
                let fields = fields
                    .named
                    .iter()
                    .filter_map(|f| {
                        let attrs = ReflectFieldAttributes::parse(&f.attrs);

                        if attrs.ignore {
                            None
                        } else {
                            let ident = f.ident.as_ref().unwrap();
                            let name = ident.to_string();
                            Some(quote_spanned! {f.span()=>
                                state.serialize_field(#name, &self.#ident)?;
                            })
                        }
                    })
                    .collect::<Vec<_>>();

                let num_fields = fields.len();

                if num_fields > 0 {
                    quote! {
                        use quartz_engine::core::serde::ser::SerializeStruct;
                        use quartz_engine::core::serde::Serializer;

                        let mut state = serializer.serialize_struct(#name, #num_fields)?;

                        #(
                            #fields
                        )*

                        state.end()
                    }
                } else {
                    quote! {
                        serializer.serialize_unit_struct(#name)
                    }
                }
            }
            _ => unimplemented!(),
        },
        _ => unimplemented!(),
    }
}
