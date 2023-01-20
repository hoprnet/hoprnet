use proc_macro2::TokenStream;
use quote::quote;

pub fn impl_encodable(ast: &syn::DeriveInput) -> TokenStream {
    let body = if let syn::Data::Struct(s) = &ast.data {
        s
    } else {
        panic!("#[derive(RlpEncodable)] is only defined for structs.");
    };

    let length_stmts: Vec<_> =
        body.fields.iter().enumerate().map(|(i, field)| encodable_length(i, field)).collect();

    let stmts: Vec<_> =
        body.fields.iter().enumerate().map(|(i, field)| encodable_field(i, field)).collect();
    let name = &ast.ident;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    let impl_block = quote! {
        trait E {
            fn rlp_header(&self) -> open_fastrlp::Header;
        }

        impl #impl_generics E for #name #ty_generics #where_clause {
            fn rlp_header(&self) -> open_fastrlp::Header {
                let mut rlp_head = open_fastrlp::Header { list: true, payload_length: 0 };
                #(#length_stmts)*
                rlp_head
            }
        }

        impl #impl_generics open_fastrlp::Encodable for #name #ty_generics #where_clause {
            fn length(&self) -> usize {
                let rlp_head = E::rlp_header(self);
                return open_fastrlp::length_of_length(rlp_head.payload_length) + rlp_head.payload_length;
            }
            fn encode(&self, out: &mut dyn open_fastrlp::BufMut) {
                E::rlp_header(self).encode(out);
                #(#stmts)*
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

pub fn impl_encodable_wrapper(ast: &syn::DeriveInput) -> TokenStream {
    let body = if let syn::Data::Struct(s) = &ast.data {
        s
    } else {
        panic!("#[derive(RlpEncodableWrapper)] is only defined for structs.");
    };

    let ident = {
        let fields: Vec<_> = body.fields.iter().collect();
        if fields.len() == 1 {
            let field = fields.first().expect("fields.len() == 1; qed");
            field_ident(0, field)
        } else {
            panic!("#[derive(RlpEncodableWrapper)] is only defined for structs with one field.")
        }
    };

    let name = &ast.ident;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    let impl_block = quote! {
        impl #impl_generics open_fastrlp::Encodable for #name #ty_generics #where_clause {
            fn length(&self) -> usize {
                self.#ident.length()
            }
            fn encode(&self, out: &mut dyn open_fastrlp::BufMut) {
                self.#ident.encode(out)
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

pub fn impl_max_encoded_len(ast: &syn::DeriveInput) -> TokenStream {
    let body = if let syn::Data::Struct(s) = &ast.data {
        s
    } else {
        panic!("#[derive(RlpEncodable)] is only defined for structs.");
    };

    let stmts: Vec<_> = body
        .fields
        .iter()
        .enumerate()
        .map(|(index, field)| encodable_max_length(index, field))
        .collect();
    let name = &ast.ident;

    let impl_block = quote! {
        unsafe impl open_fastrlp::MaxEncodedLen<{ open_fastrlp::const_add(open_fastrlp::length_of_length(#(#stmts)*), #(#stmts)*) }> for #name {}
        unsafe impl open_fastrlp::MaxEncodedLenAssoc for #name {
            const LEN: usize = { open_fastrlp::const_add(open_fastrlp::length_of_length(#(#stmts)*), { #(#stmts)* }) };
        }
    };

    quote! {
        const _: () = {
            extern crate open_fastrlp;
            #impl_block
        };
    }
}

fn field_ident(index: usize, field: &syn::Field) -> TokenStream {
    if let Some(ident) = &field.ident {
        quote! { #ident }
    } else {
        let index = syn::Index::from(index);
        quote! { #index }
    }
}

fn encodable_length(index: usize, field: &syn::Field) -> TokenStream {
    let ident = field_ident(index, field);

    quote! { rlp_head.payload_length += open_fastrlp::Encodable::length(&self.#ident); }
}

fn encodable_max_length(index: usize, field: &syn::Field) -> TokenStream {
    let fieldtype = &field.ty;

    if index == 0 {
        quote! { <#fieldtype as open_fastrlp::MaxEncodedLenAssoc>::LEN }
    } else {
        quote! { + <#fieldtype as open_fastrlp::MaxEncodedLenAssoc>::LEN }
    }
}

fn encodable_field(index: usize, field: &syn::Field) -> TokenStream {
    let ident = field_ident(index, field);

    let id = quote! { self.#ident };

    quote! { open_fastrlp::Encodable::encode(&#id, out); }
}
