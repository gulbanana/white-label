use std::env;

use proc_macro::TokenStream;
use quote::quote;
use syn::{
    Lit, Result, Token,
    parse::{Parse, ParseStream},
    parse_macro_input,
};

// "foo" or _
enum WLBrand {
    Named(String),
    Wildcard,
}

// a block similar to a match arm
struct WLMatch {
    brand: WLBrand,
    literal: Lit,
}

// a sequence of literal/wildcard matches
struct WLInput {
    matches: Vec<WLMatch>,
}

impl Parse for WLBrand {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.peek(Token![_]) {
            input.parse::<Token![_]>()?;
            Ok(WLBrand::Wildcard)
        } else {
            let lit: syn::LitStr = input.parse()?;
            Ok(WLBrand::Named(lit.value()))
        }
    }
}

impl Parse for WLMatch {
    fn parse(input: ParseStream) -> Result<Self> {
        let brand = input.parse()?;
        input.parse::<Token![=>]>()?;
        let literal = input.parse()?;
        Ok(WLMatch { brand, literal })
    }
}

impl Parse for WLInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut matches = Vec::new();
        while !input.is_empty() {
            matches.push(input.parse::<WLMatch>()?);

            // optional trailing comma
            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }
        Ok(WLInput { matches })
    }
}

/// Compile-time brand selection macro for white-label builds.
///
/// This macro reads the `WHITE_LABEL_BRAND` environment variable at compile time
/// and selects the appropriate value based on match-style syntax.
///
/// # Syntax
///
/// ```ignore
/// brand! {
///     "BrandName" => value,
///     "AnotherBrand" => other_value,
///     _ => default_value,
/// }
/// ```
///
/// # Examples
///
/// ```ignore
/// use white_label::brand;
///
/// // String literals
/// const ENDPOINT: &str = brand! {
///     "Northwind" => "https://northwind.example.com/",
///     "Contoso" => "https://contoso.example.com/",
/// };
///
/// // Boolean values with wildcard fallback
/// const DEBUG_MODE: bool = brand! {
///     "Development" => true,
///     _ => false,
/// };
///
/// // Numeric values
/// const PORT: u16 = brand! {
///     "Northwind" => 8080,
///     "Contoso" => 9090,
/// };
/// ```
///
/// # Panics
///
/// Panics at compile time if:
/// - `WHITE_LABEL_BRAND` environment variable is not set
/// - The brand value doesn't match any of the provided patterns and no wildcard is present
///
/// # Environment Variable
///
/// The macro requires `WHITE_LABEL_BRAND` to be set before compilation:
/// ```powershell
/// $env:WHITE_LABEL_BRAND = "Northwind"
/// cargo build
/// ```
#[proc_macro]
pub fn brand(input: TokenStream) -> TokenStream {
    let parsed_input = parse_macro_input!(input as WLInput);

    // check all the match arms against the environment variable value and exit early if matched
    if let Ok(env_value) = env::var("WHITE_LABEL_BRAND") {
        for WLMatch { brand, literal } in parsed_input.matches {
            match brand {
                WLBrand::Named(s) if s == env_value => {
                    return quote!(#literal).into();
                }
                WLBrand::Wildcard => {
                    return quote!(#literal).into();
                }
                _ => continue,
            }
        }
    }

    panic!("WHITE_LABEL_BRAND must be set.")
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;
    use syn::parse2;

    #[test]
    fn test_parse_brand_named() {
        let input = quote! { "Northwind" };
        let brand: WLBrand = parse2(input).unwrap();

        match brand {
            WLBrand::Named(s) => assert_eq!(s, "Northwind"),
            WLBrand::Wildcard => panic!("Expected Named, got Wildcard"),
        }
    }

    #[test]
    fn test_parse_brand_wildcard() {
        let input = quote! { _ };
        let brand: WLBrand = parse2(input).unwrap();

        match brand {
            WLBrand::Wildcard => (),
            WLBrand::Named(_) => panic!("Expected Wildcard, got Named"),
        }
    }

    #[test]
    fn test_parse_literal_string() {
        let input = quote! { "Northwind" => "https://northwind.example.com/" };
        let wl_match: WLMatch = parse2(input).unwrap();

        match wl_match.brand {
            WLBrand::Named(s) => assert_eq!(s, "Northwind"),
            WLBrand::Wildcard => panic!("Expected Named brand"),
        }

        match wl_match.literal {
            Lit::Str(s) => assert_eq!(s.value(), "https://northwind.example.com/"),
            _ => panic!("Expected string literal"),
        }
    }

    #[test]
    fn test_parse_literal_bool() {
        let input = quote! { "Development" => true };
        let wl_match: WLMatch = parse2(input).unwrap();

        match wl_match.literal {
            Lit::Bool(b) => assert!(b.value),
            _ => panic!("Expected bool literal"),
        }
    }

    #[test]
    fn test_parse_literal_int() {
        let input = quote! { "Northwind" => 8080 };
        let wl_match: WLMatch = parse2(input).unwrap();

        match wl_match.literal {
            Lit::Int(i) => assert_eq!(i.base10_parse::<u32>().unwrap(), 8080),
            _ => panic!("Expected int literal"),
        }
    }

    #[test]
    fn test_parse_literal_float() {
        let input = quote! { "Northwind" => 1.5 };
        let wl_match: WLMatch = parse2(input).unwrap();

        match wl_match.literal {
            Lit::Float(f) => assert_eq!(f.base10_parse::<f64>().unwrap(), 1.5),
            _ => panic!("Expected float literal"),
        }
    }

    #[test]
    fn test_parse_literal_char() {
        let input = quote! { "Northwind" => 'N' };
        let wl_match: WLMatch = parse2(input).unwrap();

        match wl_match.literal {
            Lit::Char(c) => assert_eq!(c.value(), 'N'),
            _ => panic!("Expected char literal"),
        }
    }

    #[test]
    fn test_parse_single_match() {
        let input = quote! { _ => "always" };
        let wl_input: WLInput = parse2(input).unwrap();

        assert_eq!(wl_input.matches.len(), 1);
    }

    #[test]
    fn test_parse_multiple_matches() {
        let input = quote! {
            "Northwind" => "value1",
            "Contoso" => "value2",
            _ => "default"
        };
        let wl_input: WLInput = parse2(input).unwrap();

        assert_eq!(wl_input.matches.len(), 3);

        // Check first match
        match &wl_input.matches[0].brand {
            WLBrand::Named(s) => assert_eq!(s, "Northwind"),
            _ => panic!("Expected Named brand"),
        }

        // Check wildcard is last
        match &wl_input.matches[2].brand {
            WLBrand::Wildcard => (),
            _ => panic!("Expected Wildcard"),
        }
    }

    #[test]
    fn test_parse_with_trailing_comma() {
        let input = quote! {
            "Northwind" => "value1",
            "Contoso" => "value2",
        };
        let wl_input: WLInput = parse2(input).unwrap();

        assert_eq!(wl_input.matches.len(), 2);
    }

    #[test]
    fn test_parse_without_trailing_comma() {
        let input = quote! {
            "Northwind" => "value1",
            "Contoso" => "value2"
        };
        let wl_input: WLInput = parse2(input).unwrap();

        assert_eq!(wl_input.matches.len(), 2);
    }
}
