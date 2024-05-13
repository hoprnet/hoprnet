#[macro_use]
extern crate syn;
extern crate proc_macro;

mod attribute;
mod generation;
mod tests;

#[proc_macro_attribute]
pub fn parameterized(
    args: ::proc_macro::TokenStream,
    input: ::proc_macro::TokenStream,
) -> ::proc_macro::TokenStream {
    impl_macro(args, input)
}

fn impl_macro(
    args: ::proc_macro::TokenStream,
    input: ::proc_macro::TokenStream,
) -> ::proc_macro::TokenStream {
    let argument_lists = parse_macro_input!(args as attribute::ParameterizedList);
    let func = parse_macro_input!(input as attribute::Fn);

    generation::generate(argument_lists, func)
}
