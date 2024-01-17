use proc_macro2::{Span, TokenStream};

use crate::parser::AttributeArgList;
use crate::test_cases::TestCases;

pub(crate) fn generate_test_cases(
    argument_lists: AttributeArgList,
    func: syn::ItemFn,
) -> proc_macro::TokenStream {
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
fn into_argument_map(arguments: &AttributeArgList) -> TestCases<'_> {
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
fn function_arguments(f: &syn::ItemFn) -> Vec<FnArgPair> {
    f.sig.inputs.iter().map(|fn_arg| {
        match fn_arg {
            syn::FnArg::Typed(syn::PatType { pat, ty, .. }) => match pat.as_ref() {
                syn::Pat::Ident(syn::PatIdent { ident, .. }) => (ident, ty) ,
                _ => panic!("[parameterized-macro] error: No identifier found for test case")
            }
            _ => panic!("[parameterized-macro] error: Unexpected receiver found in test case function arguments")
        }

    }).collect::<Vec<_>>()
}

fn generate_module<I: Iterator<Item = TokenStream>>(test_cases: I, f: &syn::ItemFn) -> TokenStream {
    let name = &f.sig.ident;
    let vis = &f.vis;
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
    f: &syn::ItemFn,
) -> TokenStream {
    let attributes = f.attrs.as_slice();
    let vis = &f.vis;
    let body_block = &f.block;
    let identifier = syn::Ident::new(&format!("case_{}", i), Span::call_site());
    let return_type = &f.sig.output;

    // Construction let bindings for all parameters
    let bindings = parameters.iter().map(|(identifier, ty)| {
        let expr = test_cases.get(identifier, i);

        generate_binding(identifier, ty, expr)
    });

    quote::quote! {
        #[test]
        #(#attributes)*
        #vis fn #identifier() #return_type {
            #(#bindings)*

            #body_block
        }
    }
}

fn generate_binding(identifier: &syn::Ident, ty: &syn::Type, expr: &syn::Expr) -> TokenStream {
    quote::quote! {
        let #identifier: #ty = #expr;
    }
}
