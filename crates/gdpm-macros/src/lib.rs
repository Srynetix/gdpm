mod tests;
use proc_macro::TokenStream;

#[proc_macro]
pub fn tests_comp(stream: TokenStream) -> TokenStream {
    tests::tests_comp_fn(stream)
}

#[proc_macro]
pub fn tests_comp_error(stream: TokenStream) -> TokenStream {
    tests::tests_comp_error_fn(stream)
}

#[proc_macro]
pub fn tests_match_tokens(stream: TokenStream) -> TokenStream {
    tests::tests_match_tokens_fn(stream)
}
