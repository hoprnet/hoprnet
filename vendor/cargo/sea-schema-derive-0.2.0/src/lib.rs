use heck::ToSnakeCase;
use proc_macro::{self, TokenStream};
use proc_macro2::Span;
use quote::{quote, quote_spanned};
use syn::{
    parse_macro_input, Attribute, DataEnum, DataStruct, DeriveInput, Fields, Ident, Lit, Meta,
    Variant,
};

fn get_iden_attr(attrs: &[Attribute]) -> Option<syn::Lit> {
    for attr in attrs {
        let name_value = match attr.parse_meta() {
            Ok(Meta::NameValue(nv)) => nv,
            _ => continue,
        };
        if name_value.path.is_ident("iden") || // interoperate with sea_query_derive Iden
            name_value.path.is_ident("name")
        {
            return Some(name_value.lit);
        }
    }
    None
}

fn get_catch_attr(attrs: &[Attribute]) -> Option<syn::Lit> {
    for attr in attrs {
        let name_value = match attr.parse_meta() {
            Ok(Meta::NameValue(nv)) => nv,
            _ => continue,
        };
        if name_value.path.is_ident("catch") {
            return Some(name_value.lit);
        }
    }
    None
}

#[proc_macro_derive(Name, attributes(iden, name, catch))]
pub fn derive_iden(input: TokenStream) -> TokenStream {
    let DeriveInput {
        ident, data, attrs, ..
    } = parse_macro_input!(input);

    let table_name = match get_iden_attr(&attrs) {
        Some(lit) => quote! { #lit },
        None => {
            let normalized = ident.to_string().to_snake_case();
            quote! { #normalized }
        }
    };

    let catch = match get_catch_attr(&attrs) {
        Some(lit) => {
            let name: String = match lit {
                Lit::Str(name) => name.value(),
                _ => panic!("expected string for `catch`"),
            };
            let method = Ident::new(name.as_str(), Span::call_site());

            quote! { #ident::#method(string) }
        }
        None => {
            quote! { None }
        }
    };

    // Currently we only support enums and unit structs
    let variants =
        match data {
            syn::Data::Enum(DataEnum { variants, .. }) => variants,
            syn::Data::Struct(DataStruct {
                fields: Fields::Unit,
                ..
            }) => {
                return quote! {
                    impl sea_schema::Name for #ident {
                        fn from_str(string: &str) -> Option<Self> {
                            if string == #table_name {
                                Some(Self)
                            } else {
                                None
                            }
                        }
                    }
                }
                .into()
            }
            _ => return quote_spanned! {
                ident.span() => compile_error!("you can only derive Name on enums or unit structs");
            }
            .into(),
        };

    if variants.is_empty() {
        return TokenStream::new();
    }

    let variant = variants
        .iter()
        .filter(|v| get_catch_attr(&v.attrs).is_none() && matches!(v.fields, Fields::Unit))
        .map(|Variant { ident, fields, .. }| match fields {
            Fields::Unit => quote! { #ident },
            _ => panic!(),
        });

    let name = variants.iter().map(|v| {
        if let Some(lit) = get_iden_attr(&v.attrs) {
            // If the user supplied a name, just use it
            quote! { #lit }
        } else if v.ident == "Table" {
            table_name.clone()
        } else {
            let ident = v.ident.to_string().to_snake_case();
            quote! { #ident }
        }
    });

    let output = quote! {
        impl sea_schema::Name for #ident {
            fn from_str(string: &str) -> Option<Self> {
                let result = match string {
                    #(#name => Some(Self::#variant),)*
                    _ => None,
                };
                if result.is_some() {
                    result
                } else {
                    #catch
                }
            }
        }
    };

    output.into()
}
