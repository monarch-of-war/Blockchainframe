mod address;
mod types;

pub use address::Address;
pub use types::AddressType;


use crate::signature::PublicKey;
use crate::{CryptoError, Result};


//Generate an addredd from a public key
pub fn public_key_to_address(public_key: &PublicKey, address_type: AddressType) ->Address{
	Address::from_public_key(public_key, address_type)
}

///Validate an address form string
pub fn validate_address(address_str: &str) -> Result<AddressType> {
	Address::validate(address_str)
}


//check validity of an addredd
pub fn is_valid_address(address_str: &str){
	Address::validate(address_str).is_ok()
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::signature::generate_keypair;

    #[test]
    fn test_address_generation() {
        let keypair = generate_keypair();
        let address = public_key_to_address(keypair.public_key(), AddressType::Base58);
        
        assert!(!address.to_string().is_empty());
        assert!(is_valid_address(&address.to_string()));
    }

    #[test]
    fn test_address_validation() {
        let keypair = generate_keypair();
        let address = public_key_to_address(keypair.public_key(), AddressType::Base58);
        let address_str = address.to_string();
        
        let addr_type = validate_address(&address_str).unwrap();
        assert_eq!(addr_type, AddressType::Base58);
    }

    #[test]
    fn test_invalid_address() {
        assert!(!is_valid_address("invalid_address"));
        assert!(!is_valid_address(""));
        assert!(!is_valid_address("1234567890"));
    }
}