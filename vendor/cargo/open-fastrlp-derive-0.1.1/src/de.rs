use proc_macro2::TokenStream;
use quote::quote;

pub fn impl_decodable(ast: &syn::DeriveInput) -> TokenStream {
    let body = if let syn::Data::Struct(s) = &ast.data {
        s
    } else {
        panic!("#[derive(RlpDecodable)] is only defined for structs.");
    };

    let stmts: Vec<_> =
        body.fields.iter().enumerate().map(|(i, field)| decodable_field(i, field)).collect();
    let name = &ast.ident;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    let impl_block = quote! {
        impl #impl_generics open_fastrlp::Decodable for #name #ty_generics #where_clause {
            fn decode(mut buf: &mut &[u8]) -> Result<Self, open_fastrlp::DecodeError> {
                let b = &mut &**buf;
                let rlp_head = open_fastrlp::Header::decode(b)?;

                if !rlp_head.list {
                    return Err(open_fastrlp::DecodeError::UnexpectedString);
                }

                let started_len = b.len();
                let this = Self {
                    #(#stmts)*
                };

                let consumed = started_len - b.len();
                if consumed != rlp_head.payload_length {
                    return Err(open_fastrlp::DecodeError::ListLengthMismatch {
                        expected: rlp_head.payload_length,
                        got: consumed,
                    });
                }

                *buf = *b;

                Ok(this)
            }
        }
    };

    quote! {
        const _: () = {
            extern crate open_fastrlp;
            #impl_block
        };
    }
}

pub fn impl_decodable_wrapper(ast: &syn::DeriveInput) -> TokenStream {
    let body = if let syn::Data::Struct(s) = &ast.data {
        s
    } else {
        panic!("#[derive(RlpEncodableWrapper)] is only defined for structs.");
    };

    assert_eq!(
        body.fields.iter().count(),
        1,
        "#[derive(RlpEncodableWrapper)] is only defined for structs with one field."
    );

    let name = &ast.ident;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    let impl_block = quote! {
        impl #impl_generics open_fastrlp::Decodable for #name #ty_generics #where_clause {
            fn decode(buf: &mut &[u8]) -> Result<Self, open_fastrlp::DecodeError> {
                Ok(Self(open_fastrlp::Decodable::decode(buf)?))
            }
        }
    };

    quote! {
        const _: () = {
            extern crate open_fastrlp;
            #impl_block
        };
    }
}

fn decodable_field(index: usize, field: &syn::Field) -> TokenStream {
    let id = if let Some(ident) = &field.ident {
        quote! { #ident }
    } else {
        let index = syn::Index::from(index);
        quote! { #index }
    };

    quote! { #id: open_fastrlp::Decodable::decode(b)?, }
}
