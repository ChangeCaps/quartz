use proc_macro2::{Span, TokenStream};
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{
    parse::ParseStream, parse_macro_input, Attribute, Data, DeriveInput, Fields, Generics, Ident,
};

#[derive(Default)]
struct ReflectFieldAttributes {
    pub ignore: bool,
}

impl ReflectFieldAttributes {
    fn get(attributes: &Vec<Attribute>) -> ReflectFieldAttributes {
        attributes
            .iter()
            .find(|a| *a.path.get_ident().as_ref().unwrap() == REFLECT_ATTRIBUTE_NAME)
            .map_or_else(Self::default, |a| {
                syn::custom_keyword!(ignore);
                let mut attributes = Self::default();
                a.parse_args_with(|input: ParseStream| {
                    if input.parse::<Option<ignore>>()?.is_some() {
                        attributes.ignore = true;
                    }

                    Ok(())
                })
                .expect("Invalid 'reflect' attribute format.");

                attributes
            })
    }
}

const REFLECT_ATTRIBUTE_NAME: &str = "reflect";

#[proc_macro_derive(Reflect, attributes(reflect))]
pub fn derive_reflect(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;

    let generics = input.generics;

    let reflect = reflect(&name, generics.clone(), &input.data);
    let serialize = serialize(&name, &input.data);

    let (impl_generics, ty_generics, _where_clause) = generics.split_for_impl();

    let expanded = quote! {
        impl #impl_generics quartz_engine::serde::Serialize for #name #ty_generics {
            fn serialize<S: quartz_engine::serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                #serialize
            }
        }

        impl #impl_generics quartz_engine::reflect::Reflect for #name #ty_generics {
            fn reflect(&mut self, deserializer: &mut dyn quartz_engine::erased_serde::Deserializer) {
                #reflect
            }

            fn as_serialize(&self) -> &dyn quartz_engine::erased_serde::Serialize {
                self
            }
        }
    };

    proc_macro::TokenStream::from(expanded)
}

fn reflect(ident: &Ident, generics: Generics, data: &Data) -> TokenStream {
    let name = ident.to_string();
    match data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => {
                let idents = fields.named.iter().map(|f| f.ident.as_ref().unwrap());

                let names = fields
                    .named
                    .iter()
                    .map(|f| f.ident.as_ref().unwrap().to_string());

                let expecting = fields.named.iter().fold(String::new(), |e, f| {
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
                            _ => Err(quartz_engine::serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                };

                quote! {
                    #field
                    use quartz_engine::serde::Deserializer;

                    impl<'de> quartz_engine::serde::Deserialize<'de> for Field {
                        fn deserialize<D>(deserializer: D) -> Result<Field, D::Error>
                        where
                            D: quartz_engine::serde::Deserializer<'de>,
                        {
                            use quartz_engine::serde::Deserializer;

                            struct FieldVisitor;

                            impl<'a, 'de> quartz_engine::serde::de::Visitor<'de> for FieldVisitor {
                                type Value = Field;

                                fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                                    formatter.write_str(#expecting)?;

                                    Ok(())
                                }

                                fn visit_str<E>(self, value: &str) -> Result<Field, E>
                                where
                                    E: quartz_engine::serde::de::Error,
                                {
                                    #field_match
                                }
                            }

                            deserializer.deserialize_identifier(FieldVisitor)
                        }
                    }

                    struct ReflectVisitor<'a> {
                        reflect: &'a mut #ident,
                    }

                    impl<'a, 'de> quartz_engine::serde::de::Visitor<'de> for ReflectVisitor<'a> {
                        type Value = ();

                        fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                            formatter.write_str(&format!("struct {}", #name))?;

                            Ok(())
                        }

                        fn visit_map<V>(self, mut map: V) -> Result<(), V::Error>
                        where
                            V: quartz_engine::serde::de::MapAccess<'de>,
                        {
                            use quartz_engine::serde::de::MapAccess;

                            while let Some(key) = map.next_key()? {
                                match key {
                                    #(
                                        Field::#idents => {
                                            self.reflect.#idents = map.next_value()?;
                                        },
                                    )*
                                }
                            }

                            Ok(())
                        }
                    }

                    const FIELDS: &[&str] = &[#(#names),*];
                    deserializer.deserialize_struct(#name, FIELDS, ReflectVisitor {
                        reflect: self,
                    }).unwrap();
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
                let num_fields = fields.named.len();

                let fields = fields.named.iter().filter_map(|f| {
                    let attrs = ReflectFieldAttributes::get(&f.attrs);

                    if attrs.ignore {
                        None
                    } else {
                        let ident = f.ident.as_ref().unwrap();
                        let name = ident.to_string();
                        Some(quote_spanned! {f.span()=>
                            state.serialize_field(#name, &self.#ident)?;
                        })
                    }
                });

                quote! {
                    use quartz_engine::serde::ser::SerializeStruct;
                    use quartz_engine::serde::Serializer;

                    let mut state = serializer.serialize_struct(#name, #num_fields)?;

                    #(
                        #fields
                    )*

                    state.end()
                }
            }
            _ => unimplemented!(),
        },
        _ => unimplemented!(),
    }
}
