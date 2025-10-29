/// Generates a two-level sharded storage path for R2 based on file_id.
///
/// The path structure is: `{level1}/{level2}/{file_id}`
/// where:
/// - level1: first 2 characters of file_id (lowercase)
/// - level2: characters 2-4 of file_id (lowercase)
///
/// # Examples
///
/// ```
/// let path = generate_storage_path("aBcD1234xyz");
/// assert_eq!(path, "ab/cd/aBcD1234xyz");
/// ```
///
/// # Arguments
///
/// * `file_id` - The unique file identifier
///
/// # Returns
///
/// A string representing the full storage path in R2
pub fn generate_storage_path(file_id: &str) -> String {
    let id_lower = file_id.to_lowercase();
    let chars: Vec<char> = id_lower.chars().collect();

    if chars.len() < 4 {
        // Fallback: if file_id is too short, pad with zeros
        let level1 = format!("{:0<2}", chars.get(0..2.min(chars.len()))
            .map(|s| s.iter().collect::<String>())
            .unwrap_or_default());
        let level2 = format!("{:0<2}", chars.get(2..4.min(chars.len()))
            .map(|s| s.iter().collect::<String>())
            .unwrap_or_default());

        return format!("{}/{}/{}", level1, level2, file_id);
    }

    let level1: String = chars[0..2].iter().collect();
    let level2: String = chars[2..4].iter().collect();

    format!("{}/{}/{}", level1, level2, file_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normal_file_id() {
        let path = generate_storage_path("aBcD1234xyz");
        assert_eq!(path, "ab/cd/aBcD1234xyz");
    }

    #[test]
    fn test_all_lowercase() {
        let path = generate_storage_path("abcd5678");
        assert_eq!(path, "ab/cd/abcd5678");
    }

    #[test]
    fn test_all_uppercase() {
        let path = generate_storage_path("ABCD5678");
        assert_eq!(path, "ab/cd/ABCD5678");
    }

    #[test]
    fn test_numeric_prefix() {
        let path = generate_storage_path("1234abcd");
        assert_eq!(path, "12/34/1234abcd");
    }

    #[test]
    fn test_short_id_3_chars() {
        let path = generate_storage_path("abc");
        assert_eq!(path, "ab/c0/abc");
    }

    #[test]
    fn test_short_id_2_chars() {
        let path = generate_storage_path("ab");
        assert_eq!(path, "ab/00/ab");
    }

    #[test]
    fn test_short_id_1_char() {
        let path = generate_storage_path("a");
        assert_eq!(path, "a0/00/a");
    }

    #[test]
    fn test_empty_id() {
        let path = generate_storage_path("");
        assert_eq!(path, "00/00/");
    }

    #[test]
    fn test_base58_typical_id() {
        // Typical Base58 IDs from our ID generator
        let path = generate_storage_path("5HpHagT");
        assert_eq!(path, "5h/ph/5HpHagT");
    }

    #[test]
    fn test_special_characters() {
        let path = generate_storage_path("a-b_c123");
        assert_eq!(path, "a-/b_/a-b_c123");
    }
}
