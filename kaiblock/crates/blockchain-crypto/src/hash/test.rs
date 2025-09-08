use super::utils::Hash;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bin_to_hex_and_hex_to_bin_roundtrip() {
        let original_bytes = [0xAB; 32];
        let hash = Hash(original_bytes);
        let hex = hash.bin_to_hex();
        assert_eq!(hex.len(), 64);

        let parsed = Hash::hex_to_bin(&hex).unwrap();
        assert_eq!(parsed.0, original_bytes);
    }

    #[test]
    fn test_hex_to_bin_invalid_length() {
        // 62 chars, should fail
        let short_hex = "aabbccddeeff".repeat(5);
        assert_eq!(short_hex.len(), 60);
        let result = Hash::hex_to_bin(&short_hex);
        assert!(result.is_err());
    }

    #[test]
    fn test_hex_to_bin_invalid_chars() {
        // 64 chars, but not valid hex
        let bad_hex = "zzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzz";
        let result = Hash::hex_to_bin(bad_hex);
        assert!(result.is_err());
    }

    #[test]
    fn test_bin_to_hex_known_value() {
        let bytes = [0x01u8; 32];
        let hash = Hash(bytes);
        let hex = hash.bin_to_hex();
        assert_eq!(hex, "0101010101010101010101010101010101010101010101010101010101010101");
    }
}