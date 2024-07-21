# serde-validate

The `serde-validate` crate provides utilities for validating deserialized structs and enums in Rust.

The core of the crate is the `Validate` trait, which defines a method for validating instances of types and returning a custom error type if validation fails.
The crate also includes a procedural macro, `validate_deser`, that generates deserialization code that validates the deserialized data.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
serde-validate = "0.1"
```

To use the macro the `macro` feature needs to be enabled (enabled by default):

```toml
[dependencies]
serde-validate = { version = "0.1", default-features = false, features = ["macro"] }
```

## Usage

### Validate Trait

Implement the `Validate` trait for your struct or enum to define custom validation logic.

```rust
use serde_validate::Validate;
use serde::Deserialize;

#[derive(Deserialize)]
struct MyStruct {
    value: i32,
}

impl Validate for MyStruct {
    type Error = String;
    fn validate(&self) -> Result<(), Self::Error> {
        if self.value < 0 {
            Err("Value must be non-negative".into())
        } else {
            Ok(())
        }
    }
}

let my_struct = MyStruct { value: 10 };
assert!(my_struct.validate().is_ok());
```

### validate_deser Macro

Use the `validate_deser` macro to automatically generate deserialization code that validates the deserialized data.

```rust
use serde-validate::{Validate, serde-validate};

#[validate_deser]
struct MyStruct {
    value: i32,
}

impl Validate for MyStruct {
    type Error = String;
    fn validate(&self) -> Result<(), Self::Error> {
        if self.value < 0 {
            Err("Value must be non-negative".into())
        } else {
            Ok(())
        }
    }
}

// Assuming you have a JSON input as below:
let json_input = r#"{ "value": 10 }"#;

// Deserialize and validate the JSON input
let my_struct: Result<MyStruct, _> = serde_json::from_str(json_input);
assert!(my_struct.validate().is_ok());

// Assuming you have a JSON input as below:
let bad_json_input = r#"{ "value": -10 }"#;

// Deserialize and validate the JSON input
let my_struct: Result<MyStruct, _> = serde_json::from_str(bad_json_input);
assert!(my_struct.is_err());
```

## License

This project is licensed under the GPL-3.0 License - see the [LICENSE](LICENSE) file for details.