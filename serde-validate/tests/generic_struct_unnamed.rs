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
use serde::Deserialize;
use serde_validate::Validate;
use serde_validate_macro::validate_deser;

#[validate_deser] 
struct NonEmptyAndNonNegative<T: Validate<Error = String>>(String, i32, T);

impl <T> Validate for NonEmptyAndNonNegative<T> where T: Validate<Error = String> {
    type Error = String;
    fn validate(&self) -> Result<(), Self::Error> {
        if self.0.is_empty() { return Err("name cannot be empty".to_string()) }
        else if self.1 < 0 { return Err("id cannot be negative".to_string()) }
        else { self.2.validate() }
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
fn test_deserialize_ok() {
    assert!(serde_json::from_str::<NonEmptyAndNonNegative<True>>("[\"Lucas\", 1, true]").is_ok());
}

#[test]
fn test_deserialize_empty() {
    assert!(serde_json::from_str::<NonEmptyAndNonNegative<True>>("[\"\", 1, true]").is_err());
}

#[test]
fn test_deserialize_negative() {
    assert!(serde_json::from_str::<NonEmptyAndNonNegative<True>>("[\"Lucas\", -1, true]").is_err());
}

#[test]
fn test_deserialize_false() {
    assert!(serde_json::from_str::<NonEmptyAndNonNegative<True>>("[\"Lucas\", 1, false]").is_err());
}
