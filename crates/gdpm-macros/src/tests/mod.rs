use proc_macro::TokenStream;
use quote::{format_ident, quote, TokenStreamExt};
use syn::{
    parenthesized,
    parse::{Parse, ParseStream, Result},
    parse_macro_input, Expr, Ident, Token,
};

struct CompArgs {
    list: Vec<(Ident, Expr)>,
}

impl Parse for CompArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut output = vec![];

        loop {
            let content;
            let _ = parenthesized!(content in input);
            let ident = content.parse()?;
            content.parse::<Token![,]>()?;
            let lit = content.parse()?;
            output.push((ident, lit));

            match input.parse::<Token![,]>() {
                Ok(_) => (),
                Err(e) => {
                    if input.is_empty() {
                        ()
                    } else {
                        return Err(e);
                    }
                }
            }

            if input.is_empty() {
                break;
            }
        }

        Ok(Self { list: output })
    }
}

struct TokenArgs {
    list: Vec<(Ident, Expr, Expr)>,
}

impl Parse for TokenArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut output = vec![];

        loop {
            let content;
            let _ = parenthesized!(content in input);
            let ident = content.parse()?;
            content.parse::<Token![,]>()?;
            let lit = content.parse()?;
            content.parse::<Token![,]>()?;
            let tokens = content.parse()?;
            output.push((ident, lit, tokens));

            match input.parse::<Token![,]>() {
                Ok(_) => (),
                Err(e) => {
                    if input.is_empty() {
                        ()
                    } else {
                        return Err(e);
                    }
                }
            }

            if input.is_empty() {
                break;
            }
        }

        Ok(Self { list: output })
    }
}

pub fn tests_comp_fn(stream: TokenStream) -> TokenStream {
    let args = parse_macro_input!(stream as CompArgs);
    let mut tests = vec![];

    for (idx, (rule, contents)) in args.list.into_iter().enumerate() {
        let test_name = format_ident!("test_comp_{}", idx);
        tests.push(quote! {
            #[test]
            fn #test_name() {
                should_compile_rule_fn(Rule::#rule, #contents);
            }
        })
    }

    let mut q = quote! {};
    q.append_all(tests);
    q.into()
}

pub fn tests_comp_error_fn(stream: TokenStream) -> TokenStream {
    let args = parse_macro_input!(stream as CompArgs);
    let mut tests = vec![];

    for (idx, (rule, contents)) in args.list.into_iter().enumerate() {
        let test_name = format_ident!("test_comp_error_{}", idx);
        tests.push(quote! {
            #[test]
            fn #test_name() {
                should_not_compile_rule_fn(Rule::#rule, #contents);
            }
        })
    }

    let mut q = quote! {};
    q.append_all(tests);
    q.into()
}

pub fn tests_match_tokens_fn(stream: TokenStream) -> TokenStream {
    let args = parse_macro_input!(stream as TokenArgs);
    let mut tests = vec![];

    for (idx, (rule, contents, tokens)) in args.list.into_iter().enumerate() {
        let test_name = format_ident!("test_match_tokens_{}", idx);
        tests.push(quote! {
            #[test]
            fn #test_name() {
                should_match_tokens(Rule::#rule, #contents, #tokens);
            }
        })
    }

    let mut q = quote! {};
    q.append_all(tests);
    q.into()
}
