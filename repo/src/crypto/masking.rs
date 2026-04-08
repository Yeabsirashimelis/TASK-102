/// Masks a sensitive value, showing only the last 4 characters.
/// Values with 4 or fewer characters are fully masked.
pub fn mask_sensitive(value: &str) -> String {
    if value.len() <= 4 {
        return "****".to_string();
    }
    let last4 = &value[value.len() - 4..];
    format!("****{}", last4)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mask_long_value() {
        assert_eq!(mask_sensitive("123-45-6789"), "****6789");
    }

    #[test]
    fn test_mask_short_value() {
        assert_eq!(mask_sensitive("abc"), "****");
    }

    #[test]
    fn test_mask_exactly_four() {
        assert_eq!(mask_sensitive("1234"), "****");
    }

    #[test]
    fn test_mask_five_chars() {
        assert_eq!(mask_sensitive("12345"), "****2345");
    }

    #[test]
    fn test_mask_empty_string() {
        assert_eq!(mask_sensitive(""), "****");
    }

    #[test]
    fn test_mask_single_char() {
        assert_eq!(mask_sensitive("x"), "****");
    }

    #[test]
    fn test_mask_credit_card() {
        assert_eq!(mask_sensitive("4111111111111111"), "****1111");
    }
}
