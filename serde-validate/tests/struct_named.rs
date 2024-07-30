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

use serde_validate::validate_deser;
use serde_validate::Validate;

#[validate_deser]
struct NonEmptyAndNonNegative {
    name: String,
    id: i32,
}

impl Validate for NonEmptyAndNonNegative {
    type Error = String;
    fn validate(&self) -> Result<(), Self::Error> {
        if self.name.is_empty() {
            Err("name cannot be empty".to_string())
        } else if self.id < 0 {
            Err("id cannot be negative".to_string())
        } else {
            Ok(())
        }
    }
}

#[test]
fn test_deserialize_ok() {
    assert!(
        serde_json::from_str::<NonEmptyAndNonNegative>("{ \"name\": \"Lucas\", \"id\": 1}").is_ok()
    );
}

#[test]
fn test_deserialize_empty() {
    assert!(
        serde_json::from_str::<NonEmptyAndNonNegative>("{ \"name\": \"\", \"id\": 1}").is_err()
    );
}

#[test]
fn test_deserialize_negative() {
    assert!(
        serde_json::from_str::<NonEmptyAndNonNegative>("{ \"name\": \"Lucas\", \"id\": -1}")
            .is_err()
    );
}
