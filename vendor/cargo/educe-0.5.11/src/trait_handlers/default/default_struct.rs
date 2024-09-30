use quote::quote;
use syn::{Data, DeriveInput, Fields, Meta, Type};

use super::{
    models::{FieldAttributeBuilder, TypeAttributeBuilder},
    TraitHandler,
};
use crate::Trait;

pub(crate) struct DefaultStructHandler;

impl TraitHandler for DefaultStructHandler {
    fn trait_meta_handler(
        ast: &mut DeriveInput,
        token_stream: &mut proc_macro2::TokenStream,
        traits: &[Trait],
        meta: &Meta,
    ) -> syn::Result<()> {
        let type_attribute = TypeAttributeBuilder {
            enable_flag:       true,
            enable_new:        true,
            enable_expression: true,
            enable_bound:      true,
        }
        .build_from_default_meta(meta)?;

        let mut default_types: Vec<&Type> = Vec::new();

        let mut default_token_stream = proc_macro2::TokenStream::new();

        if let Data::Struct(data) = &ast.data {
            if let Some(expression) = type_attribute.expression {
                for field in data.fields.iter() {
                    let _ = FieldAttributeBuilder {
                        enable_flag:       false,
                        enable_expression: false,
                    }
                    .build_from_attributes(&field.attrs, traits, &field.ty)?;
                }

                default_token_stream.extend(quote!(#expression));
            } else {
                match &data.fields {
                    Fields::Unit => {
                        default_token_stream.extend(quote!(Self));
                    },
                    Fields::Named(_) => {
                        let mut fields_token_stream = proc_macro2::TokenStream::new();

                        for field in data.fields.iter() {
                            let field_attribute = FieldAttributeBuilder {
                                enable_flag:       false,
                                enable_expression: true,
                            }
                            .build_from_attributes(&field.attrs, traits, &field.ty)?;

                            let field_name = field.ident.as_ref().unwrap();

                            if let Some(expression) = field_attribute.expression {
                                fields_token_stream.extend(quote! {
                                    #field_name: #expression,
                                });
                            } else {
                                let ty = &field.ty;

                                default_types.push(ty);

                                fields_token_stream.extend(quote! {
                                    #field_name: <#ty as ::core::default::Default>::default(),
                                });
                            }
                        }

                        default_token_stream.extend(quote! {
                            Self {
                                #fields_token_stream
                            }
                        });
                    },
                    Fields::Unnamed(_) => {
                        let mut fields_token_stream = proc_macro2::TokenStream::new();

                        for field in data.fields.iter() {
                            let field_attribute = FieldAttributeBuilder {
                                enable_flag:       false,
                                enable_expression: true,
                            }
                            .build_from_attributes(&field.attrs, traits, &field.ty)?;

                            if let Some(expression) = field_attribute.expression {
                                fields_token_stream.extend(quote!(#expression,));
                            } else {
                                let ty = &field.ty;

                                default_types.push(ty);

                                fields_token_stream
                                    .extend(quote!(<#ty as ::core::default::Default>::default(),));
                            }
                        }

                        default_token_stream.extend(quote!(Self ( #fields_token_stream )));
                    },
                }
            }
        }

        let ident = &ast.ident;

        let bound = type_attribute.bound.into_where_predicates_by_generic_parameters_check_types(
            &ast.generics.params,
            &syn::parse2(quote!(::core::default::Default)).unwrap(),
            &default_types,
            Some((false, false, false)),
        );

        let where_clause = ast.generics.make_where_clause();

        for where_predicate in bound {
            where_clause.predicates.push(where_predicate);
        }

        let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

        token_stream.extend(quote! {
            impl #impl_generics ::core::default::Default for #ident #ty_generics #where_clause {
                #[inline]
                fn default() -> Self {
                    #default_token_stream
                }
            }
        });

        if type_attribute.new {
            token_stream.extend(quote! {
                impl #impl_generics #ident #ty_generics #where_clause {
                    /// Returns the "default value" for a type.
                    #[inline]
                    pub fn new() -> Self {
                        <Self as ::core::default::Default>::default()
                    }
                }
            });
        }

        Ok(())
    }
}
