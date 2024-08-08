use proc_macro2::{Span, TokenStream};

use crate::attribute::{Fn, ParameterizedList};
use crate::tests::TestCases;

pub fn generate(argument_lists: ParameterizedList, func: Fn) -> proc_macro::TokenStream {
    // Map the given arguments by their identifier
    let values = into_argument_map(&argument_lists);
    let args = function_arguments(&func);
    let amount_of_test_cases = values.amount_of_test_cases().unwrap_or_default();

    let generated_test_cases =
        (0..amount_of_test_cases).map(|i| generate_test_case(args.as_slice(), &values, i, &func));

    generate_module(generated_test_cases, &func).into()
}

/// Transform an AttributeArgList into an ordered map which orders its
/// elements by insertion order (assuming no elements will be removed).
/// The returned map contains (identifier, argument expression list) pairs.
fn into_argument_map(arguments: &ParameterizedList) -> TestCases<'_> {
    arguments
        .args
        .iter()
        .fold(TestCases::empty(), |mut acc, args| {
            let identifier = &args.id;
            let exprs = args.param_args.iter().collect::<Vec<&syn::Expr>>();

            acc.insert(identifier, exprs);

            acc
        })
}

type FnArgPair<'ctx> = (&'ctx syn::Ident, &'ctx Box<syn::Type>);

/// Returns the vector of all typed parameter pairs for a given function.
fn function_arguments(f: &Fn) -> Vec<FnArgPair> {
    f.item_fn.sig.inputs.iter().map(|fn_arg| {
        match fn_arg {
            syn::FnArg::Typed(syn::PatType { pat, ty, .. }) => match pat.as_ref() {
                syn::Pat::Ident(syn::PatIdent { ident, .. }) => (ident, ty) ,
                _ => panic!("parameterized-macro: error: No identifier found for test case")
            }
            _ => panic!("parameterized-macro: error: Unexpected receiver found in test case function arguments")
        }

    }).collect::<Vec<_>>()
}

fn generate_module<I: Iterator<Item = TokenStream>>(test_cases: I, f: &Fn) -> TokenStream {
    let name = &f.item_fn.sig.ident;
    let vis = &f.item_fn.vis;
    let mod_ident = syn::Ident::new(&format!("{}", name), name.span());

    // we need to include `use super::*` since we put the test cases in a new module
    quote::quote! {
        #[cfg(test)]
        #vis mod #mod_ident {
            use super::*;

            #(#test_cases)*
        }
    }
}

/// Generate a single test case from the attribute inputs.
fn generate_test_case(
    parameters: &[FnArgPair],
    test_cases: &TestCases,
    i: usize,
    f: &Fn,
) -> TokenStream {
    let constness = f.constness();
    let asyncness = f.asyncness();
    let unsafety = f.unsafety();
    let visibility = f.visibility();
    let identifier = syn::Ident::new(&format!("case_{}", i), Span::call_site());
    let return_type = f.return_type();
    let body = f.body();

    // Construction let bindings for all parameters
    let bindings = parameters.iter().map(|(identifier, ty)| {
        let expr = test_cases.get(identifier, i);

        generate_binding(identifier, ty, expr)
    });

    let (use_test_macro, unrelated_attributes): (Vec<_>, Vec<_>) =
        f.attrs.iter().partition(|&m| m.is_use_test_macro());

    if use_test_macro.len() > 1 {
        panic!("parameterized-macro: the #[parameterized_macro(..)] attribute should not be present more than once!");
    }

    let unrelated_attributes = unrelated_attributes.iter().map(|attr| attr.quoted());

    let test_macro = if use_test_macro.is_empty() {
        quote::quote!(#[test])
    } else {
        let meta = use_test_macro[0];
        let meta = meta.quoted();
        quote::quote!(#[#meta])
    };

    quote::quote! {
        #test_macro
        #(#unrelated_attributes)*
        #constness #asyncness #unsafety #visibility fn #identifier() #return_type {
            #(#bindings)*

            #body
        }
    }
}

fn generate_binding(identifier: &syn::Ident, ty: &syn::Type, expr: &syn::Expr) -> TokenStream {
    quote::quote! {
        let #identifier: #ty = #expr;
    }
}
