# Blockchain Crypto Library

A comprehensive cryptographic library for blockchain applications, providing secure hash functions, digital signatures, and address generation utilities.

## Features

### 🔐 Hash Functions (`src/hash/`)
- **SHA-256** wrapper with consistent interface
- **Double SHA-256** for enhanced security (Bitcoin-style)
- **Merkle Tree** implementation with proof generation and verification
- **Hash utilities** including difficulty calculation and serialization helpers

### ✍️ Digital Signatures (`src/signature/`)
- **Ed25519** key pair generation, signing, and verification
- **Key serialization** with hex encoding support
- **Secure key management** with hidden private key display
- **Cross-platform** random key generation

### 🏠 Address Generation (`src/address/`)
- **Multiple encoding formats**: Base58, Hex with checksum, Raw hex
- **Public key to address** conversion
- **Address validation** and format detection
- **Checksum verification** for data integrity

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
blockchain-crypto = { path = "../blockchain-crypto" }
```

### Basic Usage

```rust
use blockchain_crypto::*;

fn main() -> Result<()> {
    // Generate a key pair
    let keypair = signature::generate_keypair();
    
    // Create an address
    let address = address::public_key_to_address(
        keypair.public_key(), 
        AddressType::Base58
    );
    
    // Sign a message
    let message = b"Hello, blockchain!";
    let signature = keypair.sign(message);
    
    // Verify signature
    let is_valid = keypair.verify(message, &signature);
    assert!(is_valid);
    
    // Hash data
    let hash = hash::sha256(message);
    println!("Hash: {}", hash);
    
    Ok(())
}
```

## Detailed Examples

### Hash Functions

```rust
use blockchain_crypto::hash::*;

// Basic hashing
let data = b"transaction data";
let hash = sha256(data);
let double_hash = double_sha256(data);

// Merkle tree for transaction verification
let transactions = vec![b"tx1", b"tx2", b"tx3", b"tx4"];
let merkle_tree = merkle_tree_from_data(&transactions)?;

// Generate proof that tx2 is in the tree
let proof = merkle_tree.generate_proof(1)?;
assert!(MerkleTree::verify_proof(&proof));
```

### Digital Signatures

```rust
use blockchain_crypto::signature::*;

// Generate keys
let keypair = generate_keypair();

// Sign message
let message = b"transfer 100 coins";
let signature = keypair.sign(message);

// Verify signature
assert!(keypair.verify(message, &signature));

// Key serialization
let private_hex = keypair.export_private_key();
let restored = KeyPair::from_private_hex(&private_hex)?;
```

### Address Generation

```rust
use blockchain_crypto::address::*;

let keypair = generate_keypair();

// Different address formats
let base58_addr = public_key_to_address(
    keypair.public_key(), 
    AddressType::Base58
);
let hex_addr = public_key_to_address(
    keypair.public_key(), 
    AddressType::HexChecksum
);

// Address validation
assert!(is_valid_address(&base58_addr.to_string()));

// Parse address from string
let parsed = Address::from_string(&base58_addr.to_string())?;
```

## Architecture

```
blockchain-crypto/
├── src/
│   ├── lib.rs              # Main library interface
│   ├── hash/               # Hash functions module
│   │   ├── mod.rs          # Module interface
│   │   ├── types.rs        # Hash256 type definition
│   │   ├── merkle.rs       # Merkle tree implementation
│   │   └── utils.rs        # Hash utilities
│   ├── signature/          # Digital signatures module
│   │   ├── mod.rs          # Module interface
│   │   ├── types.rs        # Key types (PublicKey, PrivateKey)
│   │   ├── signature.rs    # Signature type
│   │   └── keypair.rs      # KeyPair implementation
│   └── address/            # Address generation module
│       ├── mod.rs          # Module interface
│       ├── types.rs        # AddressType enum
│       └── address.rs      # Address implementation
└── examples/
    └── crypto_usage.rs     # Comprehensive usage examples
```

## Security Features

- **Ed25519** cryptography for quantum-resistant signatures
- **Secure random** key generation using OS entropy
- **Checksum validation** for all address formats
- **Memory-safe** implementation in Rust
- **Constant-time** operations where applicable

## Address Formats

| Format | Example | Use Case |
|--------|---------|----------|
| **Base58** | `1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa` | Bitcoin-compatible addresses |
| **Hex+Checksum** | `0x742d35Cc6635C0532925a3b8d28a9b4CcE6b8B0` | Ethereum-style addresses |
| **Raw Hex** | `0x742d35cc6635c0532925a3b8d28a9b4cce6b8b0` | Simple hex encoding |

## Hash Utilities

- **Difficulty calculation**: Count leading zero bits
- **Target generation**: Create difficulty targets
- **Chain hashing**: Sequential hash dependencies
- **Serialization helpers**: Hash complex data structures

## Error Handling

The library uses a comprehensive error system:

```rust
pub enum CryptoError {
    InvalidKey(String),
    InvalidSignature,
    InvalidHash(String),
    AddressError(String),
    SerializationError(String),
    InvalidMerkleProof,
}
```

All functions return `Result<T, CryptoError>` for proper error handling.

## Testing

Run tests with:

```bash
cargo test
```

The library includes comprehensive tests for:
- Hash function correctness
- Signature verification
- Address validation
- Merkle proof verification
- Serialization round-trips
- Error conditions

## Performance Considerations

- **Ed25519** provides fast signature generation and verification
- **Merkle trees** enable efficient proof generation for large datasets
- **Streaming hash** calculation for large data
- **Zero-allocation** operations where possible

## Dependencies

- `sha2`: SHA-256 hash functions
- `ed25519-dalek`: Ed25519 cryptography
- `bs58`: Base58 encoding
- `hex`: Hexadecimal encoding
- `serde`: Serialization support
- `rand`: Secure random number generation

## Future Enhancements

- [ ] Support for secp256k1 signatures (Bitcoin compatibility)
- [ ] BIP32 hierarchical deterministic keys
- [ ] Multi-signature address support
- [ ] Hardware security module (HSM) integration
- [ ] Batch signature verification
- [ ] Post-quantum cryptography options

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Contributing

Contributions are welcome! Please ensure all tests pass and add tests for new functionality.

## Security Notice

This library is designed for educational and development purposes. For production use, please ensure proper security auditing and consider using established cryptographic libraries.