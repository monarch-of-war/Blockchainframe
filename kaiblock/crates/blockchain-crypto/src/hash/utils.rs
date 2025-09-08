use hex;

pub struct Hash([u8; 32]);

impl Hash {
    fn bin_to_hex(&self) -> String {
        hex::encode(self.0)
    }


    // A way to convert a hex string back to a Hash and handle potential errors
    fn hex_to_bin(hex_string: &str) -> Result<Hash, hex::FromHexError> {
        if hex_string.len() != 64 {
            return Err(hex::FromHexError::InvalidStringLength);
        }
        // Decode the hex string into bytes
        // Ensure the resulting byte array is of the correct length
        match hex::decode(hex_string) {
            Ok(bytes) => {
                let mut array = [0u8; 32];
                array.copy_from_slice(&bytes);
                Ok(Hash(array))
            }
            Err(err) => Err(err),
        }
    }
}