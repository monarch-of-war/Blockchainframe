use hex;

struct Hash([u8; 32]);

impl Hash {
    fn bin_to_hex(&self) -> String {
        hex::encode(self)
    }

    fn hex_to_bin(hex_string: &str) -> Hash {
        let bytes = hex::decode(hex_string).expect("Decoding failed");
        assert!(bytes.len() == 32, "Hex string must represent 32 bytes");
        let mut array = [0u8, 32];
        array.copy_from_slice(&bytes);
        Hash(array)
    }
}