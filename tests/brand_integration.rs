//! Integration tests for the `brand!` macro with deterministic brand selection.
//! These tests compile with `WHITE_LABEL_BRAND="TestBrand"` set by `build.rs`.

use white_label::brand;

#[test]
fn literal_string() {
    let result = brand! {
        "Northwind" => "https://northwind.example.com/",
        "TestBrand" => "https://test.example.com/",
    };

    assert_eq!(result, "https://test.example.com/");
}

#[test]
fn literal_int() {
    let result = brand! {
        "Northwind" => 8080,
        "TestBrand" => 7777,
    };

    assert_eq!(result, 7777);
}

#[test]
fn literal_bool() {
    let result = brand! {
        "Northwind" => true,
        "TestBrand" => false,
    };

    assert_eq!(result, false);
}

#[test]
fn literal_float() {
    let result = brand! {
        "Northwind" => 1.5,
        "TestBrand" => 3.14,
    };

    assert_eq!(result, 3.14);
}

#[test]
fn literal_char() {
    let result = brand! {
        "Northwind" => 'A',
        "TestBrand" => 'T',
    };

    assert_eq!(result, 'T');
}

#[test]
fn wildcard_fallback() {
    let result: &str = brand! {
        "NonExistentBrand" => "specific",
        _ => "fallback",
    };

    assert_eq!(result, "fallback");
}

#[test]
fn wildcard_no_fallback() {
    let result: &str = brand! {
        "TestBrand" => "specific",
        _ => "fallback",
    };

    assert_eq!(result, "specific");
}

#[test]
fn wildcard_only() {
    let result = brand! {
        _ => "always_this",
    };

    assert_eq!(result, "always_this");
}

#[cfg(test)]
mod compile_time_tests {
    use super::*;

    // These tests verify that the macro generates valid Rust code at compile time
    #[test]
    #[allow(unused_mut)]
    fn assign_mut() {
        let mut value: &str = brand! {
            "Northwind" => "const_northwind",
            "TestBrand" => "const_test"
        };

        assert_eq!(value, "const_test");
    }

    #[test]
    fn assign_const() {
        const VALUE: &str = brand! {
            "Northwind" => "const_northwind",
            "TestBrand" => "const_test"
        };

        assert_eq!(VALUE, "const_test");
    }

    #[test]
    fn assign_static() {
        static VALUE: bool = brand! {
            "Development" => true,
            _ => false,
        };

        assert_eq!(VALUE, false);
    }
}
