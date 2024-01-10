// Copyright 2019-2022 Parity Technologies (UK) Ltd.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

extern crate alloc;
extern crate proc_macro;

mod attr;
mod trait_bounds;
mod utils;

use self::attr::{Attributes, CaptureDocsAttr, CratePathAttr};
use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use syn::{
    parse::{Error, Result},
    parse_quote,
    punctuated::Punctuated,
    token::Comma,
    visit_mut::VisitMut,
    Data, DataEnum, DataStruct, DeriveInput, Field, Fields, Ident, Lifetime,
};

#[proc_macro_derive(TypeInfo, attributes(scale_info, codec))]
pub fn type_info(input: TokenStream) -> TokenStream {
    match generate(input.into()) {
        Ok(output) => output.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

fn generate(input: TokenStream2) -> Result<TokenStream2> {
    let type_info_impl = TypeInfoImpl::parse(input)?;
    let type_info_impl_toks = type_info_impl.expand()?;
    Ok(quote! {
        #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
        const _: () = {
            #type_info_impl_toks;
        };
    })
}

struct TypeInfoImpl {
    ast: DeriveInput,
    attrs: Attributes,
}

impl TypeInfoImpl {
    fn parse(input: TokenStream2) -> Result<Self> {
        let ast: DeriveInput = syn::parse2(input)?;
        let attrs = attr::Attributes::from_ast(&ast)?;

        Ok(Self { ast, attrs })
    }

    fn expand(&self) -> Result<TokenStream2> {
        let ident = &self.ast.ident;
        let ident_str = ident.to_string();
        let scale_info = crate_path(self.attrs.crate_path())?;

        let where_clause = trait_bounds::make_where_clause(
            &self.attrs,
            ident,
            &self.ast.generics,
            &self.ast.data,
            &scale_info,
        )?;

        let (impl_generics, ty_generics, _) = self.ast.generics.split_for_impl();

        let type_params = self.ast.generics.type_params().map(|tp| {
            let ty_ident = &tp.ident;
            let ty = if self
                .attrs
                .skip_type_params()
                .map_or(true, |skip| !skip.skip(tp))
            {
                quote! { ::core::option::Option::Some(#scale_info::meta_type::<#ty_ident>()) }
            } else {
                quote! { ::core::option::Option::None }
            };
            quote! {
                #scale_info::TypeParameter::new(::core::stringify!(#ty_ident), #ty)
            }
        });

        let build_type = match &self.ast.data {
            Data::Struct(ref s) => self.generate_composite_type(s, &scale_info),
            Data::Enum(ref e) => self.generate_variant_type(e, &scale_info),
            Data::Union(_) => return Err(Error::new_spanned(&self.ast, "Unions not supported")),
        };
        let docs = self.generate_docs(&self.ast.attrs);

        let replaces = self.attrs.replace_segments().map(|r| {
            let search = r.search();
            let replace = r.replace();

            quote!(( #search, #replace ))
        });

        Ok(quote! {
            impl #impl_generics #scale_info::TypeInfo for #ident #ty_generics #where_clause {
                type Identity = Self;
                fn type_info() -> #scale_info::Type {
                    #scale_info::Type::builder()
                        .path(#scale_info::Path::new_with_replace(
                            #ident_str,
                            ::core::module_path!(),
                            &[ #( #replaces ),* ]
                        ))
                        .type_params(#scale_info::prelude::vec![ #( #type_params ),* ])
                        #docs
                        .#build_type
                }
            }
        })
    }

    fn generate_composite_type(
        &self,
        data_struct: &DataStruct,
        scale_info: &syn::Path,
    ) -> TokenStream2 {
        let fields = match data_struct.fields {
            Fields::Named(ref fs) => {
                let fields = self.generate_fields(&fs.named);
                quote! { named()#( #fields )* }
            }
            Fields::Unnamed(ref fs) => {
                let fields = self.generate_fields(&fs.unnamed);
                quote! { unnamed()#( #fields )* }
            }
            Fields::Unit => {
                quote! {
                    unit()
                }
            }
        };

        quote! {
            composite(#scale_info::build::Fields::#fields)
        }
    }

    fn generate_fields(&self, fields: &Punctuated<Field, Comma>) -> Vec<TokenStream2> {
        fields
            .iter()
            .filter(|f| !utils::should_skip(&f.attrs))
            .map(|f| {
                let (ty, ident) = (&f.ty, &f.ident);
                // Replace any field lifetime params with `static to prevent "unnecessary lifetime parameter"
                // warning. Any lifetime parameters are specified as 'static in the type of the impl.
                struct StaticLifetimesReplace;
                impl VisitMut for StaticLifetimesReplace {
                    fn visit_lifetime_mut(&mut self, lifetime: &mut Lifetime) {
                        *lifetime = parse_quote!('static)
                    }
                }
                let mut ty = match ty {
                    // When a type is specified as part of a `macro_rules!`, the tokens passed to
                    // the `TypeInfo` derive macro are a type `Group`, which is pretty printed with
                    // invisible delimiters e.g. /*«*/ bool /*»*/. To avoid printing the delimiters
                    // the inner type element is extracted.
                    syn::Type::Group(group) => (*group.elem).clone(),
                    _ => ty.clone(),
                };
                StaticLifetimesReplace.visit_type_mut(&mut ty);

                let type_name = clean_type_string(&quote!(#ty).to_string());
                let docs = self.generate_docs(&f.attrs);
                let type_of_method = if utils::is_compact(f) {
                    quote!(compact)
                } else {
                    quote!(ty)
                };
                let name = if let Some(ident) = ident {
                    quote!(.name(::core::stringify!(#ident)))
                } else {
                    quote!()
                };
                quote!(
                    .field(|f| f
                        .#type_of_method::<#ty>()
                        #name
                        .type_name(#type_name)
                        #docs
                    )
                )
            })
            .collect()
    }

    fn generate_variant_type(&self, data_enum: &DataEnum, scale_info: &syn::Path) -> TokenStream2 {
        let variants = &data_enum.variants;

        let variants = variants
            .into_iter()
            .filter(|v| !utils::should_skip(&v.attrs))
            .enumerate()
            .map(|(i, v)| {
                let ident = &v.ident;
                let v_name = quote! {::core::stringify!(#ident) };
                let docs = self.generate_docs(&v.attrs);
                let index = utils::variant_index(v, i);

                let fields = match v.fields {
                    Fields::Named(ref fs) => {
                        let fields = self.generate_fields(&fs.named);
                        Some(quote! {
                            .fields(#scale_info::build::Fields::named()
                                #( #fields )*
                            )
                        })
                    }
                    Fields::Unnamed(ref fs) => {
                        let fields = self.generate_fields(&fs.unnamed);
                        Some(quote! {
                            .fields(#scale_info::build::Fields::unnamed()
                                #( #fields )*
                            )
                        })
                    }
                    Fields::Unit => None,
                };

                quote! {
                    .variant(#v_name, |v|
                        v
                            .index(#index as ::core::primitive::u8)
                            #fields
                            #docs
                    )
                }
            });
        quote! {
            variant(
                #scale_info::build::Variants::new()
                    #( #variants )*
            )
        }
    }

    fn generate_docs(&self, attrs: &[syn::Attribute]) -> Option<TokenStream2> {
        let docs_builder_fn = match self.attrs.capture_docs() {
            CaptureDocsAttr::Never => None, // early return if we never capture docs.
            CaptureDocsAttr::Default => Some(quote!(docs)),
            CaptureDocsAttr::Always => Some(quote!(docs_always)),
        }?;

        let docs = attrs
            .iter()
            .filter_map(|attr| {
                if let Ok(syn::Meta::NameValue(meta)) = attr.parse_meta() {
                    if meta.path.get_ident().map_or(false, |ident| ident == "doc") {
                        if let syn::Lit::Str(lit) = &meta.lit {
                            let lit_value = lit.value();
                            let stripped = lit_value.strip_prefix(' ').unwrap_or(&lit_value);
                            let lit: syn::Lit = parse_quote!(#stripped);
                            Some(lit)
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        if !docs.is_empty() {
            Some(quote! {
                .#docs_builder_fn(&[ #( #docs ),* ])
            })
        } else {
            None
        }
    }
}

/// Get the name of a crate, to be robust against renamed dependencies.
fn crate_name_path(name: &str) -> Result<syn::Path> {
    proc_macro_crate::crate_name(name)
        .map(|crate_name| {
            use proc_macro_crate::FoundCrate::*;
            match crate_name {
                Itself => Ident::new("self", Span::call_site()).into(),
                Name(name) => {
                    let crate_ident = Ident::new(&name, Span::call_site());
                    parse_quote!( ::#crate_ident )
                }
            }
        })
        .map_err(|e| syn::Error::new(Span::call_site(), &e))
}

fn crate_path(crate_path_attr: Option<&CratePathAttr>) -> Result<syn::Path> {
    crate_path_attr
        .map(|path_attr| Ok(path_attr.path().clone()))
        .unwrap_or_else(|| crate_name_path("scale-info"))
}

fn clean_type_string(input: &str) -> String {
    input
        .replace(" ::", "::")
        .replace(":: ", "::")
        .replace(" ,", ",")
        .replace(" ;", ";")
        .replace(" [", "[")
        .replace("[ ", "[")
        .replace(" ]", "]")
        .replace(" (", "(")
        // put back a space so that `a: (u8, (bool, u8))` isn't turned into `a: (u8,(bool, u8))`
        .replace(",(", ", (")
        .replace("( ", "(")
        .replace(" )", ")")
        .replace(" <", "<")
        .replace("< ", "<")
        .replace(" >", ">")
        .replace("& \'", "&'")
}
