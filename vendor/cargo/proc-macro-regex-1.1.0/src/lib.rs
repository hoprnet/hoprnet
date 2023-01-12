mod character;
mod dfa;
mod macro_input;
mod nfa;

use crate::{
    dfa::{Dfa, DfaToTokens},
    macro_input::MacroInput,
    nfa::Nfa,
};
use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;

/// The macro creates a function which returns `true` if the argument matches the regex.
///
/// If the first argument is an identifier (name), then this is the name of  the function, which
/// would be generated. Example:
/// ```rust
/// use proc_macro_regex::regex;
///
/// regex!(the_name_of_the_function "the regex to check");
/// ```
///
/// Alternative, if the first argument is a visibility keyword, then this is the visibility of the
/// function. Otherwise, the function is private. Example:
/// ```rust
/// # use proc_macro_regex::regex;
/// regex!(pub public_function "the function is public");
/// regex!(private_function "the function is private");
/// ```
///
/// The next argument is a string of the regex, which the function should check. Alternative, a
/// byte string can be given, if the input should be a byte array (`&[u8]`). otherwise a string is
/// taken.
/// ```rust
/// # use proc_macro_regex::regex;
/// regex!(string_function "This function takes a string");
/// regex!(bytes_function "This function takes a byte array");
/// ```
///
/// At the end, a positive number can be given to set the limit of the lookup table
/// (see `README.md`).
/// ```rust
/// # use proc_macro_regex::regex;
/// regex!(limit_function "The limit is set to 100 bytes" 100);
/// ```
///
/// # Syntax
/// The given regex works the same as in the [regex](https://crates.io/crates/regex) crate.
/// * If the `^` is at the beginning of the regex, then it is checked if the input is match at the
///   beginning of the text.
/// * If the `$` is at the end, then it is checked if the input is match at the end of the text.
/// * If both are present then the whole input is checked.
/// * Otherwise, is check if the string contains the regex.
#[proc_macro]
pub fn regex(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as MacroInput);
    let visibility = input.get_visibility();
    let name = input.get_name();
    let threshold = input.get_threshold();
    let (argument_type, body) = if input.is_str() {
        let nfa = Nfa::<char>::try_from(&input).unwrap();
        let dfa = Dfa::from(nfa);
        (
            quote! {
                str
            },
            DfaToTokens::new(dfa, threshold).get_token_streams(),
        )
    } else {
        let nfa = Nfa::<u8>::try_from(&input).unwrap();
        let dfa = Dfa::from(nfa);
        (
            quote! {
                [u8]
            },
            DfaToTokens::new(dfa, threshold).get_token_streams(),
        )
    };
    let function = quote! {
        #visibility fn #name(s: &#argument_type) -> bool {
            #body
        }
    };
    function.into()
}
