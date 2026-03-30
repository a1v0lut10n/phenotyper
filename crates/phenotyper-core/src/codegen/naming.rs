// SPDX-License-Identifier: Apache-2.0
//! Naming strategy — DSL names → idiomatic Rust names (REQ-CODEGEN-009).

/// Convert a DSL type name to idiomatic Rust PascalCase.
///
/// The strategy: detect transitions between uppercase runs and lowercase
/// letters, inserting a word boundary. E.g., `CSVLine` → `CsvLine`.
pub fn to_rust_type_name(name: &str) -> String {
    let chars: Vec<char> = name.chars().collect();
    if chars.is_empty() {
        return String::new();
    }

    let mut result = String::with_capacity(name.len());
    let mut i = 0;

    while i < chars.len() {
        if chars[i].is_ascii_uppercase() {
            // Count consecutive uppercase chars
            let start = i;
            while i < chars.len() && chars[i].is_ascii_uppercase() {
                i += 1;
            }
            let upper_run = i - start;

            if upper_run == 1 {
                // Single uppercase: just emit it
                result.push(chars[start]);
            } else if i < chars.len() && chars[i].is_ascii_lowercase() {
                // Uppercase run followed by lowercase: the last upper is the
                // start of a new word. E.g., CSV|Line → Csv + Line
                // Emit first uppercase, then remaining as lowercase, except last
                result.push(chars[start]);
                for c in chars.iter().take(i - 1).skip(start + 1) {
                    result.push(c.to_ascii_lowercase());
                }
                // Last uppercase of the run starts a new word
                result.push(chars[i - 1]);
            } else {
                // Uppercase run at end of string or followed by non-alpha
                result.push(chars[start]);
                for c in chars.iter().take(i).skip(start + 1) {
                    result.push(c.to_ascii_lowercase());
                }
            }
        } else {
            result.push(chars[i]);
            i += 1;
        }
    }

    result
}

/// Convert a DSL field name to idiomatic Rust snake_case.
///
/// For v1, field names in Phenotyper are already lowercase identifiers,
/// so this is mostly an identity function.
pub fn to_rust_field_name(name: &str) -> String {
    // DSL field names are already snake_case
    name.to_string()
}

/// Convert an enum member name to PascalCase variant.
pub fn to_rust_variant_name(name: &str) -> String {
    // Capitalize first letter
    let mut chars = name.chars();
    match chars.next() {
        Some(c) => {
            let mut result = c.to_uppercase().to_string();
            result.extend(chars);
            result
        }
        None => String::new(),
    }
}

/// Convert a DSL type name to a Rust builder struct name.
pub fn to_builder_name(type_name: &str) -> String {
    format!("{}Builder", to_rust_type_name(type_name))
}

/// Generate a union enum name from a type and field name.
pub fn to_union_enum_name(type_name: &str, field_name: &str) -> String {
    format!(
        "{}{}Value",
        to_rust_type_name(type_name),
        to_rust_variant_name(field_name)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn type_name_simple() {
        assert_eq!(to_rust_type_name("Record"), "Record");
    }

    #[test]
    fn type_name_all_caps_prefix() {
        assert_eq!(to_rust_type_name("CSVLine"), "CsvLine");
        assert_eq!(to_rust_type_name("CSVFieldValue"), "CsvFieldValue");
        assert_eq!(to_rust_type_name("CSVFieldValues"), "CsvFieldValues");
        assert_eq!(to_rust_type_name("CSVFile"), "CsvFile");
        assert_eq!(to_rust_type_name("CSVLines"), "CsvLines");
    }

    #[test]
    fn type_name_all_caps() {
        assert_eq!(to_rust_type_name("CSV"), "Csv");
        assert_eq!(to_rust_type_name("IO"), "Io");
    }

    #[test]
    fn type_name_already_pascal() {
        assert_eq!(to_rust_type_name("ScalarValue"), "ScalarValue");
        assert_eq!(to_rust_type_name("Node"), "Node");
    }

    #[test]
    fn variant_name() {
        assert_eq!(to_rust_variant_name("public"), "Public");
        assert_eq!(to_rust_variant_name("protected"), "Protected");
        assert_eq!(to_rust_variant_name("Red"), "Red");
    }

    #[test]
    fn builder_name() {
        assert_eq!(to_builder_name("CSVLine"), "CsvLineBuilder");
        assert_eq!(to_builder_name("Record"), "RecordBuilder");
    }

    #[test]
    fn union_enum_name() {
        assert_eq!(
            to_union_enum_name("CSVFieldValue", "value"),
            "CsvFieldValueValueValue"
        );
    }
}
