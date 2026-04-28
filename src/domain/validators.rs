//! Domain: Validation logic
//!
//! Pure functions — no I/O, no UI. Data in, Result out.

/// Validate part number format: `###-PCA######-X`
/// e.g., `127-PCA000097-E`
pub fn validate_part_number(input: &str) -> Result<(), String> {
    if input.is_empty() {
        return Err("Part number cannot be empty".into());
    }
    let chars: Vec<char> = input.chars().collect();
    // ###-PCA######-X = 3+1+3+6+1+1 = 15 chars
    if chars.len() != 15 {
        return Err(format!(
            "Part number must be 15 characters (got {}). Format: ###-PCA######-X",
            chars.len()
        ));
    }
    for i in 0..3 {
        if !chars[i].is_ascii_digit() {
            return Err(format!("Position {} must be a digit", i + 1));
        }
    }
    if chars[3] != '-' {
        return Err("Expected '-' at position 4".into());
    }
    if chars[4] != 'P' || chars[5] != 'C' || chars[6] != 'A' {
        return Err("Expected 'PCA' at positions 5-7".into());
    }
    for i in 7..13 {
        if !chars[i].is_ascii_digit() {
            return Err(format!("Position {} must be a digit", i + 1));
        }
    }
    if chars[13] != '-' {
        return Err("Expected '-' at position 14".into());
    }
    if !chars[14].is_ascii_uppercase() {
        return Err("Last character must be an uppercase letter (A-Z)".into());
    }
    Ok(())
}

/// Validate a field is non-empty.
pub fn validate_required(label: &str, input: &str) -> Result<(), String> {
    if input.trim().is_empty() {
        return Err(format!("'{}' is required", label));
    }
    Ok(())
}

/// Validate that a numeric string parses as u64.
pub fn validate_u64(label: &str, input: &str) -> Result<(), String> {
    if input.trim().is_empty() {
        return Ok(()); // optional
    }
    input.trim().parse::<u64>()
        .map_err(|_| format!("'{}' must be a number", label))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_part_numbers() {
        assert!(validate_part_number("127-PCA000097-E").is_ok());
        assert!(validate_part_number("000-PCA999999-Z").is_ok());
    }

    #[test]
    fn invalid_part_numbers() {
        assert!(validate_part_number("").is_err());
        assert!(validate_part_number("12-PCA000097-E").is_err());
        assert!(validate_part_number("127-pca000097-E").is_err());
        assert!(validate_part_number("127-PCA000097-").is_err());
    }
}
