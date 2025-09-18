use serde::{Serialize, Deserialize};

/// Different address encoding formats
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AddressType {
	///Base58 encoding (Bitcoin style)
	Base58,
	///Hexadecimal with checksum
	HexChecksum,
	///Raw hexadecimal
	Hex,
}


impl AddressType{
	pub fn prefix(&self) -> &'static str{
		match self{
			AddressType::Base58 => "1",
			AddressType::HexChecksum => "0x",
			AddressType::Hex => "0x",
		}
	}

	///detect address type from string
	pub fn detect(address_str: &str) -> Option<Self>{
		if address_str.starts_with("0x") {
			if address_str.len() == 42 { //0x + 40 chars
				Some(AddressType::HexChecksum)
			}else{
				Some(AddressType::Hex)
			}
		}else if address_str.chars().all(|c| "123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz".contains(c)){
			Some(AddressType::Base58)
		}else{
			None
		}
	}

}


impl Default for AddressType{
	fn dafault() -> Self{
		AddressType::Base58
	}
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_address_type_detection() {
        assert_eq!(AddressType::detect("0x1234567890123456789012345678901234567890"), Some(AddressType::HexChecksum));
        assert_eq!(AddressType::detect("0x123"), Some(AddressType::Hex));
        assert_eq!(AddressType::detect("1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa"), Some(AddressType::Base58));
        assert_eq!(AddressType::detect("invalid!"), None);
    }

    #[test]
    fn test_address_type_prefix() {
        assert_eq!(AddressType::Base58.prefix(), "1");
        assert_eq!(AddressType::HexChecksum.prefix(), "0x");
        assert_eq!(AddressType::Hex.prefix(), "0x");
    }
}