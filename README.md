# white-label

Compile-time rebranding for Rust projects.

## Usage
The `brand!` macro supports all Rust literal types: strings, integers, floats, booleans, and characters.

```rust
use white_label::brand;

const ENDPOINT: &str = brand! {
    "Northwind" => "https://northwind.example.com/",
    "Contoso" => "https://contoso.example.com/",
    _ => "https://default.example.com/",
};
```

Set a default brand in `.cargo/config.toml`:

```toml
[env]
WHITE_LABEL_BRAND = "Northwind"
```
