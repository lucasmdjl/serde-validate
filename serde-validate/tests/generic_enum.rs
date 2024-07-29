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

use std::fmt::Display;
use serde::Deserialize;
use serde_validate_macro::validate_deser;
use serde_validate::Validate;

#[validate_deser]
enum NonEmptyAndNonNegative<T> where T: Validate, T::Error: Display {
    String { name: String },
    Int(i32),
    Something(T)
}

impl <T> Validate for NonEmptyAndNonNegative<T> where T: Validate, T::Error: Display {
    type Error = String;
    fn validate(&self) -> Result<(), Self::Error> {
        match self {
            NonEmptyAndNonNegative::String { name } if name.is_empty() => Err("name cannot be empty".to_string()),
            NonEmptyAndNonNegative::Int(i) if *i < 0 => Err("id cannot be negative".to_string()),
            NonEmptyAndNonNegative::Something(t) => t.validate().map_err(|e| e.to_string()),
            _ => Ok(())
        }
    }
}

#[derive(Deserialize)]
struct True(bool);
impl Validate for True {
    type Error = String;
    fn validate(&self) -> Result<(), Self::Error> {
        if self.0 { Ok(()) } else { Err("not true".to_string()) }
    }
}


#[test]
fn test_deserialize_not_empty() {
    assert!(serde_json::from_str::<NonEmptyAndNonNegative<True>>("{\"String\": {\"name\": \"Lucas\"}}").is_ok());
}

#[test]
fn test_deserialize_empty() {
    assert!(serde_json::from_str::<NonEmptyAndNonNegative<True>>("{\"String\": {\"name\": \"\"}}").is_err());
}

#[test]
fn test_deserialize_negative() {
    assert!(serde_json::from_str::<NonEmptyAndNonNegative<True>>("{\"Int\": -1}").is_err());
}

#[test]
fn test_deserialize_not_negative() {
    assert!(serde_json::from_str::<NonEmptyAndNonNegative<True>>("{\"Int\": 1}").is_ok());
}

#[test]
fn test_deserialize_true() {
    assert!(serde_json::from_str::<NonEmptyAndNonNegative<True>>("{\"Something\": true}").is_ok());
}

#[test]
fn test_deserialize_false() {
    assert!(serde_json::from_str::<NonEmptyAndNonNegative<True>>("{\"Something\": false}").is_err());
}
