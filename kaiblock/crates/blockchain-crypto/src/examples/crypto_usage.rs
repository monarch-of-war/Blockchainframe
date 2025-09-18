use blockchain_crypto::{
    // Hash functions
    hash::{sha256, double_sha256, MerkleTree, merkle_tree_from_data, Hash256},
    // Signatures
    signature::{generate_keypair, KeyPair, sign_message, verify_signature},
    // Addresses
    address::{public_key_to_address, AddressType, Address},
    // Error handling
    Result, CryptoError,
};

fn main() -> Result<()> {
    println!("=== Blockchain Crypto Library Demo ===\n");
    
    // 1. Hash Functions Demo
    hash_demo()?;
    
    // 2. Digital Signatures Demo  
    signature_demo()?;
    
    // 3. Address Generation Demo
    address_demo()?;
    
    // 4. Merkle Tree Demo
    merkle_demo()?;
    
    // 5. Complete Workflow Demo
    complete_workflow_demo()?;
    
    Ok(())
}

fn hash_demo() -> Result<()> {
    println!("1. === Hash Functions Demo ===");
    
    let data = b"Hello, Blockchain!";
    
    // Basic SHA-256
    let hash = sha256(data);
    println!("SHA-256 of '{}': {}", 
             String::from_utf8_lossy(data), hash);
    
    // Double SHA-256
    let double_hash = double_sha256(data);
    println!("Double SHA-256: {}", double_hash);
    
    // Hash from hex
    let hex_hash = Hash256::from_hex("0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef")?;
    println!("Hash from hex: {}", hex_hash);
    
    println!();
    Ok(())
}

fn signature_demo() -> Result<()> {
    println!("2. === Digital Signatures Demo ===");
    
    // Generate a key pair
    let keypair = generate_keypair();
    println!("Generated key pair:");
    println!("  Public key:  {}", keypair.public_key().to_hex());
    println!("  Private key: [HIDDEN]");
    
    // Sign a message
    let message = b"This is a signed message";
    let signature = sign_message(keypair.private_key(), message);
    println!("Message: '{}'", String::from_utf8_lossy(message));
    println!("Signature: {}", signature.to_hex());
    
    // Verify signature
    let is_valid = verify_signature(keypair.public_key(), message, &signature);
    println!("Signature valid: {}", is_valid);
    
    // Test with tampered message
    let tampered_message = b"This is a tampered message";
    let is_tampered_valid = verify_signature(keypair.public_key(), tampered_message, &signature);
    println!("Tampered message valid: {}", is_tampered_valid);
    
    // Key serialization
    let private_hex = keypair.export_private_key();
    let restored_keypair = KeyPair::from_private_hex(&private_hex)?;
    println!("Key pair serialization successful: {}", keypair == restored_keypair);
    
    println!();
    Ok(())
}

fn address_demo() -> Result<()> {
    println!("3. === Address Generation Demo ===");
    
    let keypair = generate_keypair();
    println!("Public key: {}", keypair.public_key().to_hex());
    
    // Generate different address formats
    let base58_addr = public_key_to_address(keypair.public_key(), AddressType::Base58);
    let hex_checksum_addr = public_key_to_address(keypair.public_key(), AddressType::HexChecksum);
    let hex_addr = public_key_to_address(keypair.public_key(), AddressType::Hex);
    
    println!("Addresses generated:");
    println!("  Base58:      {}", base58_addr);
    println!("  Hex+Checksum: {}", hex_checksum_addr);  
    println!("  Hex:         {}", hex_addr);
    
    // Address validation
    println!("Address validation:");
    println!("  Base58 valid:      {}", Address::validate(&base58_addr.to_string()).is_ok());
    println!("  Hex+Checksum valid: {}", Address::validate(&hex_checksum_addr.to_string()).is_ok());
    println!("  Invalid address:   {}", Address::validate("invalid123").is_ok());
    
    // Address parsing
    let parsed_addr = Address::from_string(&base58_addr.to_string())?;
    println!("Parsed address matches: {}", parsed_addr == base58_addr);
    
    println!();
    Ok(())
}

fn merkle_demo() -> Result<()> {
    println!("4. === Merkle Tree Demo ===");
    
    // Create some sample transaction data
    let transactions = vec![
        b"tx1: Alice -> Bob: 10 coins",
        b"tx2: Bob -> Charlie: 5 coins", 
        b"tx3: Charlie -> David: 3 coins",
        b"tx4: David -> Eve: 2 coins",
        b"tx5: Eve -> Alice: 1 coin",
    ];
    
    // Build merkle tree
    let merkle_tree = merkle_tree_from_data(&transactions)?;
    println!("Merkle tree created with {} transactions", transactions.len());
    println!("Merkle root: {}", merkle_tree.root());
    
    // Generate proof for transaction 2 (index 1)
    let proof = merkle_tree.generate_proof(1)?;
    println!("\nProof for transaction 2:");
    println!("  Leaf index: {}", proof.leaf_index);
    println!("  Leaf hash:  {}", proof.leaf_hash);
    println!("  Siblings:   {} hashes", proof.siblings.len());
    println!("  Root hash:  {}", proof.root);
    
    // Verify the proof
    let is_valid = MerkleTree::verify_proof(&proof);
    println!("  Proof valid: {}", is_valid);
    
    // Test with all transactions
    println!("\nVerifying all transactions:");
    for (i, _) in transactions.iter().enumerate() {
        let tx_proof = merkle_tree.generate_proof(i)?;
        let valid = MerkleTree::verify_proof(&tx_proof);
        println!("  Transaction {}: {}", i + 1, if valid { "✓" } else { "✗" });
    }
    
    println!();
    Ok(())
}

fn complete_workflow_demo() -> Result<()> {
    println!("5. === Complete Workflow Demo ===");
    
    // Simulate a simple blockchain transaction workflow
    
    // 1. Create user identities
    let alice_keypair = generate_keypair();
    let bob_keypair = generate_keypair();
    
    let alice_addr = public_key_to_address(alice_keypair.public_key(), AddressType::Base58);
    let bob_addr = public_key_to_address(bob_keypair.public_key(), AddressType::Base58);
    
    println!("Users created:");
    println!("  Alice: {}", alice_addr);
    println!("  Bob:   {}", bob_addr);
    
    // 2. Create a transaction
    let transaction_data = format!(
        "{{\"from\": \"{}\", \"to\": \"{}\", \"amount\": 100, \"nonce\": 1}}", 
        alice_addr, bob_addr
    );
    
    // 3. Hash the transaction
    let tx_hash = sha256(transaction_data.as_bytes());
    println!("\nTransaction created:");
    println!("  Data: {}", transaction_data);
    println!("  Hash: {}", tx_hash);
    
    // 4. Alice signs the transaction
    let signature = alice_keypair.sign(tx_hash.as_bytes());
    println!("  Signature: {}", signature.to_hex());
    
    // 5. Verify the signature (as would be done by network)
    let signature_valid = alice_keypair.public_key().verify(tx_hash.as_bytes(), &signature);
    println!("  Signature valid: {}", signature_valid);
    
    // 6. Create a block with multiple transactions
    let block_transactions = vec![
        transaction_data.as_bytes(),
        b"tx2: Bob -> Charlie: 50",
    0    b"tx3: Charlie -> David: 25", 
    ];
    
    // 7. Build merkle tree for the block
    let block_merkle = merkle_tree_from_data(&block_transactions)?;
    println!("\nBlock created:");
    println!("  Transactions: {}", block_transactions.len());
    println!("  Merkle root:  {}", block_merkle.root());
    
    // 8. Create block header hash
    let block_header = format!(
        "{{\"merkle_root\": \"{}\", \"timestamp\": {}, \"nonce\": 12345}}", 
        block_merkle.root(),
        1234567890u64
    );
    let block_hash = sha256(block_header.as_bytes());
    
    println!("  Block header: {}", block_header);
    println!("  Block hash:   {}", block_hash);
    
    // 9. Generate and verify merkle proof for first transaction
    let tx_proof = block_merkle.generate_proof(0)?;
    let proof_valid = MerkleTree::verify_proof(&tx_proof);
    
    println!("\nMerkle proof verification:");
    println!("  Transaction included in block: {}", proof_valid);
    
    println!("\n=== Workflow completed successfully! ===");
    
    Ok(())
}

// Additional utility functions for demonstration

#[allow(dead_code)]
fn demonstrate_hash_difficulty() -> Result<()> {
    use blockchain_crypto::hash::{hash_difficulty, meets_difficulty};
    
    println!("=== Hash Difficulty Demo ===");
    
    // Create some sample hashes with different difficulties
    let easy_hash = Hash256::from_hex("00123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef")?;
    let hard_hash = Hash256::from_hex("00000056789abcdef0123456789abcdef0123456789abcdef0123456789abcdef")?;
    
    println!("Easy hash: {} (difficulty: {})", easy_hash, hash_difficulty(&easy_hash));
    println!("Hard hash: {} (difficulty: {})", hard_hash, hash_difficulty(&hard_hash));
    
    println!("Easy hash meets difficulty 8: {}", meets_difficulty(&easy_hash, 8));
    println!("Hard hash meets difficulty 20: {}", meets_difficulty(&hard_hash, 20));
    
    Ok(())
}

#[allow(dead_code)]  
fn demonstrate_serialization() -> Result<()> {
    use blockchain_crypto::signature::SerializableKeyPair;
    
    println!("=== Serialization Demo ===");
    
    let keypair = generate_keypair();
    let serializable: SerializableKeyPair = (&keypair).into();
    
    // In a real application, you would serialize this to JSON/file
    let json_str = serde_json::to_string(&serializable).unwrap();
    println!("Serialized keypair: {}", json_str);
    
    // Deserialize back
    let deserialized: SerializableKeyPair = serde_json::from_str(&json_str).unwrap();
    let restored_keypair = KeyPair::try_from(deserialized)?;
    
    println!("Restoration successful: {}", keypair == restored_keypair);
    
    Ok(())
}