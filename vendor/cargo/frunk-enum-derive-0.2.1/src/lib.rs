//! This crate adds the `LabelledGenericEnum` derive which integrates enums with the frunk
//! transmogrification function.

extern crate proc_macro;

use quote::quote;
use syn::parse_macro_input;

use syn::spanned::Spanned as _;

/// A representation of a variant's member field.  This is far simpler than the one presented by
/// `syn` as it always has a field name, and it doesn't track things like access permissions.
struct Field {
    pub ident: syn::Ident,
    pub ty: syn::Type,
}

/// Convert the fields of a variant into a simpler, more consistent form.  This also creates
/// artificial names for "tuple structs" (`_0`, `_1`, ...) so downstream code can just read the
/// `ident` field and get the correct name.
fn simplify_fields(fields: &syn::Fields) -> Vec<Field> {
    use syn::Fields::*;
    match fields {
        Unit => Vec::new(),
        Named(named) => named
            .named
            .iter()
            .map(|f| Field {
                ident: f.ident.as_ref().unwrap().clone(),
                ty: f.ty.clone(),
            })
            .collect(),
        Unnamed(unnamed) => unnamed
            .unnamed
            .iter()
            .enumerate()
            .map(|(i, f)| Field {
                ident: syn::Ident::new(&format!("_{}", i), f.span()),
                ty: f.ty.clone(),
            })
            .collect(),
    }
}

/// Recursively pack up variants into a chain of `HCons` types (with associated field names).
fn create_hlist_repr<'a>(mut fields: impl Iterator<Item = &'a Field>) -> proc_macro2::TokenStream {
    match fields.next() {
        None => quote!(frunk::HNil),
        Some(Field { ref ident, ref ty }) => {
            let tail = create_hlist_repr(fields);
            let ident = frunk_proc_macro_helpers::build_label_type(ident);
            quote!(frunk::HCons<frunk::labelled::Field<#ident, #ty>, #tail>)
        }
    }
}

/// Recursively pack up the variants into a chain of `HEither` generic enums (with associated
/// variant names).
fn create_repr_for0<'a>(
    mut variants: impl Iterator<Item = &'a syn::Variant>,
) -> proc_macro2::TokenStream {
    match variants.next() {
        None => quote!(frunk_enum_core::Void),
        Some(v) => {
            let ident_ty = frunk_proc_macro_helpers::build_label_type(&v.ident);
            let fields = simplify_fields(&v.fields);
            let hlist = create_hlist_repr(fields.iter());
            let tail = create_repr_for0(variants);
            quote! {
                frunk_enum_core::HEither<frunk_enum_core::Variant<#ident_ty, #hlist>, #tail>
            }
        }
    }
}

/// Generates the `Repr` for a given `enum` definition.
///
/// ```ignore
/// type Repr = HEither<Variant<(f,i,r,s,t), Hlist![Field<(_0), A>]>, HEither<Variant<(s,e,c,o,n,d), Hlist![Field<(_0), B>]>, Void>>;
/// ```
fn create_repr_for(input: &syn::ItemEnum) -> proc_macro2::TokenStream {
    let repr = create_repr_for0(input.variants.iter());
    quote!(type Repr = #repr;)
}

/// Create the body of the cases in the `into()` implementation for a given variant.  Assumes that
/// the captured fields are in bindings named as per the field names.
///
/// The `depth` argument indicates how many `Tail` wrappers to add (e.g. how far down the `HEither`
/// chain this variant lies).
fn create_into_case_body_for<'a>(
    ident: &syn::Ident,
    fields: impl Iterator<Item = &'a Field>,
    depth: usize,
) -> proc_macro2::TokenStream {
    let fields = fields.map(|f| {
        let ident = &f.ident;
        let ident_ty = frunk_proc_macro_helpers::build_label_type(ident);
        quote!(frunk::field!(#ident_ty, #ident))
    });
    let ident_ty = frunk_proc_macro_helpers::build_label_type(ident);
    let mut inner = quote!(frunk_enum_core::HEither::Head(
        frunk_enum_core::variant!(#ident_ty, frunk::hlist![#(#fields),*])
    ));
    for _ in 0..depth {
        inner = quote!(frunk_enum_core::HEither::Tail(#inner))
    }
    inner
}

/// Create cases for the variants for the `into()` implementation.  Captures the fields of the
/// variant into bindings corresponding to the field names.
fn create_into_cases_for<'a>(
    enum_ident: &'a syn::Ident,
    variants: impl Iterator<Item = &'a syn::Variant> + 'a,
) -> impl Iterator<Item = proc_macro2::TokenStream> + 'a {
    use syn::Fields::*;
    variants.enumerate().map(move |(idx, v)| {
        let variant_ident = &v.ident;
        let labelled_fields = simplify_fields(&v.fields);
        let pattern_vars = labelled_fields.iter().map(|f| &f.ident);
        let body = create_into_case_body_for(variant_ident, labelled_fields.iter(), idx);

        // Tediously patterns are rendered differently for the three styles so add appropriate wrapping
        // here.
        let pattern_vars = match v.fields {
            Unit => quote!(),
            Unnamed(_) => quote!((#(#pattern_vars),*)),
            Named(_) => quote!({#(#pattern_vars),*}),
        };

        quote!(#enum_ident::#variant_ident #pattern_vars => #body)
    })
}

/// Generate the implementation of `into()` for the given enum.
///
/// ```ignore
///  fn into(self) -> Self::Repr {
///     match self {
///         First(v) => HEither::Head(variant!((f, i, r, s, t), hlist!(field!((_0), v)))),
///         Second(v) => HEither::Tail(HEither::Head(variant!((s, e, c, o, n, d), hlist!(field!((_0), v))))),
///     }
/// }
/// ```
fn create_into_for(input: &syn::ItemEnum) -> proc_macro2::TokenStream {
    let cases = create_into_cases_for(&input.ident, input.variants.iter());
    quote! {
        fn into(self) -> Self::Repr {
            match self {
                #(#cases),*
            }
        }
    }
}

/// Create the pattern of the case statement for the `from()` implementation.  This captures the
/// fields from the `Repr` into bindings to be re-constituted into the `Self` in the case body.
///
/// The `depth` argument indicates how many `Tail` wrappers to unpack (e.g. how far down the `HEither`
/// chain this variant lies).
fn create_from_case_pattern_for<'a>(
    fields: impl Iterator<Item = &'a Field>,
    depth: usize,
) -> proc_macro2::TokenStream {
    let fields = fields.map(|f| &f.ident);
    let mut inner = quote!(
        frunk_enum_core::HEither::Head(frunk_enum_core::Variant {
            value: frunk::hlist_pat!(#(#fields),*),
            ..
        })
    );
    for _ in 0..depth {
        inner = quote!(frunk_enum_core::HEither::Tail(#inner));
    }
    inner
}

/// Create the body of the case statement for the `from()` implementation.  This builds the output
/// variant from the captured fields.  It assumes that the fields are captured in the pattern as
/// variables named as per the field identifiers.
fn create_from_case_body_for<'a>(
    ident: &syn::Ident,
    variant: &syn::Variant,
    fields: impl Iterator<Item = &'a Field>,
) -> proc_macro2::TokenStream {
    use syn::Fields::*;
    let variant_ident = &variant.ident;
    let fields = fields.map(|f| &f.ident);
    let fields = match variant.fields {
        Unit => quote!(),
        Unnamed(_) => quote!((#(#fields.value),*)),
        Named(_) => {
            let fields = fields.map(|f| quote!(#f: #f.value));
            quote!({#(#fields),*})
        }
    };
    quote!(#ident::#variant_ident #fields)
}

/// Generate a case for the `from()` implementation for each variant (equivalently, for each `Repr`
/// variant).  These cases are not complete (as they don't cover the "all Tail" case).
fn create_from_cases_for<'a>(
    enum_ident: &'a syn::Ident,
    variants: impl Iterator<Item = &'a syn::Variant> + 'a,
) -> impl Iterator<Item = proc_macro2::TokenStream> + 'a {
    variants.enumerate().map(move |(idx, v)| {
        let labelled_fields = simplify_fields(&v.fields);
        let pattern = create_from_case_pattern_for(labelled_fields.iter(), idx);
        let body = create_from_case_body_for(enum_ident, &v, labelled_fields.iter());

        quote!(#pattern => #body)
    })
}

/// Generate an unreacahble case for unpacking a `Repr` (the `Tail(Tail(...(Void)...))` case).
fn create_void_from_case(depth: usize) -> proc_macro2::TokenStream {
    let mut pattern = quote!(void);
    for _ in 0..depth {
        pattern = quote!(frunk_enum_core::HEither::Tail(#pattern));
    }
    quote!(#pattern => match void {})
}

/// Generate the implementation of `from()` for the given enum.
///
/// ```ignore
/// fn from(repr: Self::Repr) -> Self {
///     match repr {
///         HEither::Head(Variant { value: hlist_pat!(v), .. }) => First(v.value),
///         HEither::Tail(HEither::Head(Variant { value: hlist_pat!(v), .. }))=> Second(e.value),
///         HEither::Tail(HEither::Tail(void)) => match void {}, // Unreachable
///     }
/// }
/// ```
///
/// The final case is needed for match-completeness, but fortunately it's uninhabited, so it'll
/// never be hit.
fn create_from_for(input: &syn::ItemEnum) -> proc_macro2::TokenStream {
    let cases = create_from_cases_for(&input.ident, input.variants.iter());
    let void_case = create_void_from_case(input.variants.len());
    quote! {
        fn from(repr: Self::Repr) -> Self {
            match repr {
                #(#cases),*,
                #void_case,
            }
        }
    }
}

/// Generates the complete derived code for an enum.  This is the main functional entrypoint for
/// this crate, and is the entry point used for testing (as the proc-macro entrypoint cannot).
fn generate_for_derive_input(input: &syn::ItemEnum) -> proc_macro2::TokenStream {
    let ident = &input.ident;
    let (_, ty_generics, where_clause) = input.generics.split_for_impl();
    let repr = create_repr_for(&input);
    let into = create_into_for(&input);
    let from = create_from_for(&input);

    quote! {
        impl #ty_generics frunk::LabelledGeneric for #ident #ty_generics #where_clause {
            #repr
            #into
            #from
        }
    }
}

/// ```edition2018
/// #[derive(frunk_enum_derive::LabelledGenericEnum)]
/// enum Foo<A, B> {
///   Bar,
///   Baz(u32, A, String),
///   Quux { name: String, inner: B },
/// }
/// ```
#[proc_macro_derive(LabelledGenericEnum)]
pub fn derive_labelled_generic(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as syn::ItemEnum);
    generate_for_derive_input(&input).into()
}

#[test]
fn test_generate_for_enum() {
    let raw_enum = syn::parse_str::<syn::ItemEnum>(
        r#"
        enum Foo<C, E> {
            A,
            B(C, C, C),
            D { foo: E, bar: E },
        }
    "#,
    )
    .unwrap();

    let derived = generate_for_derive_input(&raw_enum);

    assert!(syn::parse_str::<syn::ItemImpl>(&derived.to_string()).is_ok());
}
