#[macro_use]
extern crate syn;
extern crate proc_macro;

mod generation;
mod parser;
mod test_cases;

#[proc_macro_attribute]
pub fn parameterized(
    args: ::proc_macro::TokenStream,
    input: ::proc_macro::TokenStream,
) -> ::proc_macro::TokenStream {
    let argument_lists = parse_macro_input!(args as parser::AttributeArgList);
    let func = parse_macro_input!(input as ::syn::ItemFn);

    generation::generate_test_cases(argument_lists, func)
}
