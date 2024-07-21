/*
 * serde-validate - A library for validating deserialized structs and enums
 *
 * Copyright (C) 2024 Lucas M. de Jong Larrarte
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program. If not, see <https://www.gnu.org/licenses/>.
 */

//! # serde-validate
//!
//! The `serde-validate` crate provides utilities for validating deserialized structs and enums in Rust.
//!
//! The core of the crate is the `Validate` trait, which defines a method for validating instances of types
//! and returning a custom error type if validation fails. The crate also includes a procedural serde-validate-macro,
//! `serde-validate`, that generates deserialization code that validates the deserialized data.
//!
//! ## Example
//!
//! ```rust
//! use serde_validate::{Validate, validate_deser};
//!
//! #[validate_deser]
//! struct MyStruct {
//!     value: i32,
//! }
//!
//! impl Validate for MyStruct {
//!     type Error = String;
//!
//!     fn validate(&self) -> Result<(), Self::Error> {
//!         if self.value < 0 {
//!             Err("Value must be non-negative".into())
//!         } else {
//!             Ok(())
//!         }
//!     }
//! }
//!
//! // Assuming you have a JSON input as below:
//! let good_json_input = r#"{ "value": 10 }"#;
//!
//! // Deserialize and validate the JSON input
//! let my_struct: Result<MyStruct, _> = serde_json::from_str(good_json_input);
//! assert!(my_struct.is_ok());
//!
//! // Assuming you have a JSON input as below:
//! let bad_json_input = r#"{ "value": -10 }"#;
//!
//! // Deserialize and validate the JSON input
//! let my_struct: Result<MyStruct, _> = serde_json::from_str(bad_json_input);
//! assert!(my_struct.is_err());
//! ```

/// The `Validate` trait defines the contract for validating deserialized structs.
///
/// Implementors of this trait are required to provide their own validation logic
/// and an associated error type.
///
/// # Example
///
/// ```
/// use serde_validate::Validate;
///
/// struct MyStruct {
///     value: i32,
/// }
///
/// impl Validate for MyStruct {
///     type Error = String;
///
///     fn validate(&self) -> Result<(), Self::Error> {
///         if self.value < 0 {
///             Err("Value must be non-negative".into())
///         } else {
///             Ok(())
///         }
///     }
/// }
///
/// let my_struct = MyStruct { value: 10 };
/// assert!(my_struct.validate().is_ok());
/// ```
pub trait Validate: Sized {
    /// The error type returned by the `validate` method.
    type Error;

    /// Validates the instance, returning `Ok(())` if serde-validate, or an `Error` otherwise.
    fn validate(&self) -> Result<(), Self::Error>;

    /// Consumes the instance, validating it and returning the instance itself if serde-validate.
    ///
    /// This method provides a convenient way to validate and immediately use the instance.
    fn validated(self) -> Result<Self, Self::Error> {
        self.validate().map(|_| self)
    }
}

#[cfg(feature = "macro")]
pub use serde_validate_macro::validate_deser;
