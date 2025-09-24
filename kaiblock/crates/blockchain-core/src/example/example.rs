///Directly from Claudie


use blockchain_core::*;
use blockchain_crypto::{signature::generate_keypair, address::public_key_to_address, AddressType};
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    println!("🚀 Blockchain Core Integration Demo");
    println!("=" .repeat(50));
    
    // 1. Create blockchain configuration
    demo_blockchain_setup().await?;
    
    // 2. Demonstrate transactions
    demo_transactions().await?;
    
    // 3. Demonstrate mining
    demo_mining().await?;
    
    // 4. Demonstrate state management
    demo_state_management().await?;
    
    // 5. Demonstrate validation
    demo_validation().await?;
    
    // 6. Performance benchmarks
    demo_performance().await?;
    
    println!("\n✅ All demos completed successfully!");
    Ok(())
}

async fn demo_blockchain_setup() -> Result<()> {
    println!("\n📦 1. Blockchain Setup Demo");
    println!("-".repeat(30));
    
    // Create users
    let alice_keypair = generate_keypair();
    let bob_keypair = generate_keypair();
    let charlie_keypair = generate_keypair();
    
    let alice_addr = public_key_to_address(alice_keypair.public_key(), AddressType::Base58);
    let bob_addr = public_key_to_address(bob_keypair.public_key(), AddressType::Base58);
    let charlie_addr = public_key_to_address(charlie_keypair.public_key(), AddressType::Base58);
    
    println!("👤 Created users:");
    println!("   Alice:   {}", alice_addr);
    println!("   Bob:     {}", bob_addr);
    println!("   Charlie: {}", charlie_addr);
    
    // Configure genesis with initial balances
    let mut initial_accounts = HashMap::new();
    initial_accounts.insert(alice_addr, 1000000000); // 10 coins
    initial_accounts.insert(bob_addr, 500000000);    // 5 coins
    
    let config = ChainConfig {
        network: NetworkType::Devnet,
        chain_id: 12345,
        account_model: AccountModel::Hybrid,
        genesis: GenesisConfig {
            coinbase_recipient: charlie_addr, // Charlie is the initial miner
            genesis_reward: 5000000000,       // 50 coins
            initial_accounts,
            timestamp: Some(1640995200), // Jan 1, 2022
            difficulty: 1, // Low difficulty for demo
        },
        validation_rules: ValidationRules {
            max_block_size: 1024 * 1024,
            min_transaction_fee: 100,
            coinbase_maturity: 10, // Shorter for demo
            ..Default::default()
        },
        mining: MiningConfig {
            block_reward: 2500000000, // 25 coins
            target_block_time: 10,    // 10 seconds for demo
            max_mining_iterations: 100000,
            enable_mining: true,
        },
    };
    
    // Create blockchain
    let blockchain = Blockchain::new(config)?;
    
    println!("⛓️  Blockchain created:");
    println!("   Network: {:?}", blockchain.config.network);
    println!("   Chain ID: {}", blockchain.config.chain_id);
    println!("   Genesis height: {}", blockchain.height());
    
    // Check initial balances
    println!("\n💰 Initial balances:");
    println!("   Alice:   {} satoshis", blockchain.get_balance(&alice_addr));
    println!("   Bob:     {} satoshis", blockchain.get_balance(&bob_addr));
    println!("   Charlie: {} satoshis", blockchain.get_balance(&charlie_addr));
    
    let stats = blockchain.get_stats();
    println!("\n📊 Chain stats:");
    println!("   Blocks: {}", stats.total_blocks);
    println!("   Transactions: {}", stats.total_transactions);
    println!("   Total supply: {} satoshis", stats.total_supply);
    
    Ok(())
}

async fn demo_transactions() -> Result<()> {
    println!("\n💸 2. Transaction Demo");
    println!("-".repeat(30));
    
    // Set up blockchain
    let mut blockchain = create_demo_blockchain()?;
    
    let alice_keypair = generate_keypair();
    let bob_keypair = generate_keypair();
    let alice_addr = public_key_to_address(alice_keypair.public_key(), AddressType::Base58);
    let bob_addr = public_key_to_address(bob_keypair.public_key(), AddressType::Base58);
    
    // Fund Alice
    fund_account(&mut blockchain, alice_addr, 1000000000)?;
    
    println!("📝 Creating transactions...");
    
    // Create different types of transactions
    
    // 1. Simple transfer
    let transfer_tx = Transaction::new_account(
        alice_addr,
        bob_addr, 
        100000000, // 1 coin
        0,         // nonce
        21000,     // gas limit
        50,        // gas price
        vec![],    // no data
    );
    
    println!("   💳 Transfer: {} -> {} (1 coin)", alice_addr, bob_addr);
    
    // 2. Transaction with data (contract call simulation)
    let contract_tx = Transaction::new_account(
        alice_addr,
        bob_addr,
        50000000,  // 0.5 coins
        1,         // nonce
        100000,    // higher gas for contract
        100,       // higher gas price
        b"contract_call(transfer, 0.5)".to_vec(), // contract data
    );
    
    println!("   📋 Contract call: {} -> {} (0.5 coin + data)", alice_addr, bob_addr);
    
    // Add transactions to mempool
    let tx1_id = blockchain.add_transaction(transfer_tx)?;
    let tx2_id = blockchain.add_transaction(contract_tx)?;
    
    println!("\n🏊 Mempool status:");
    println!("   Pending transactions: {}", blockchain.mempool().len());
    println!("   TX1 ID: {}", tx1_id);
    println!("   TX2 ID: {}", tx2_id);
    
    let mempool_stats = blockchain.mempool().get_stats();
    println!("   Total fees: {} satoshis", mempool_stats.total_fees);
    println!("   Avg fee/byte: {} sat/byte", mempool_stats.avg_fee_per_byte);
    
    Ok(())
}

async fn demo_mining() -> Result<()> {
    println!("\n⛏️  3. Mining Demo");
    println!("-".repeat(30));
    
    let mut blockchain = create_demo_blockchain()?;
    let miner_keypair = generate_keypair();
    let miner_addr = public_key_to_address(miner_keypair.public_key(), AddressType::Base58);
    
    // Add some transactions to mine
    let alice_keypair = generate_keypair();
    let bob_keypair = generate_keypair();
    let alice_addr = public_key_to_address(alice_keypair.public_key(), AddressType::Base58);
    let bob_addr = public_key_to_address(bob_keypair.public_key(), AddressType::Base58);
    
    fund_account(&mut blockchain, alice_addr, 1000000000)?;
    
    // Add multiple transactions
    for i in 0..3 {
        let tx = Transaction::new_account(
            alice_addr,
            bob_addr,
            10000000, // 0.1 coin each
            i,        // nonce
            21000,
            50 + i * 10, // Increasing gas price
            vec![],
        );
        blockchain.add_transaction(tx)?;
    }
    
    println!("⛏️  Mining block with {} pending transactions...", blockchain.mempool().len());
    
    let initial_height = blockchain.height();
    let mining_start = std::time::Instant::now();
    
    // Mine a block
    let mined_block = blockchain.mine_block(miner_addr)?;
    
    let mining_time = mining_start.elapsed();
    
    println!("✨ Block mined successfully!");
    println!("   Height: {} -> {}", initial_height, blockchain.height());
    println!("   Mining time: {:?}", mining_time);
    println!("   Block hash: {}", mined_block.id());
    println!("   Nonce found: {}", mined_block.header.nonce);
    println!("   Transactions: {}", mined_block.transaction_count());
    println!("   Block size: {} bytes", mined_block.size());
    
    // Check miner reward
    let miner_balance = blockchain.get_balance(&miner_addr);
    println!("   Miner reward: {} satoshis", miner_balance);
    
    // Verify mempool is cleared
    println!("   Mempool after mining: {} transactions", blockchain.mempool().len());
    
    // Mine a few more blocks
    println!("\n🔄 Mining 3 more blocks...");
    for i in 1..=3 {
        let start = std::time::Instant::now();
        let block = blockchain.mine_block(miner_addr)?;
        let duration = start.elapsed();
        println!("   Block {} mined in {:?} (hash: {})", 
                i + 1, duration, block.id());
    }
    
    let final_stats = blockchain.get_stats();
    println!("\n📈 Final mining stats:");
    println!("   Chain height: {}", final_stats.height);
    println!("   Total blocks: {}", final_stats.total_blocks);
    println!("   Miner total rewards: {} satoshis", blockchain.get_balance(&miner_addr));
    
    Ok(())
}

async fn demo_state_management() -> Result<()> {
    println!("\n🌍 4. State Management Demo");
    println!("-".repeat(30));
    
    let mut blockchain = create_demo_blockchain()?;
    
    // Create multiple users
    let users: Vec<_> = (0..5)
        .map(|i| {
            let keypair = generate_keypair();
            let address = public_key_to_address(keypair.public_key(), AddressType::Base58);
            (format!("User{}", i + 1), keypair, address)
        })
        .collect();
    
    println!("👥 Created {} users", users.len());
    
    // Fund first user
    let initial_balance = 1000000000; // 10 coins
    fund_account(&mut blockchain, users[0].2, initial_balance)?;
    
    println!("💰 Initial state:");
    for (name, _, addr) in &users {
        println!("   {}: {} satoshis", name, blockchain.get_balance(addr));
    }
    
    // Create state snapshot
    let initial_snapshot = blockchain.world_state().snapshot();
    println!("\n📸 Created state snapshot");
    
    // Simulate a series of transactions
    println!("\n🔄 Executing transaction chain...");
    let mut nonce = 0;
    
    // User1 -> User2: 2 coins
    let tx1 = Transaction::new_account(users[0].2, users[1].2, 200000000, nonce, 21000, 50, vec![]);
    blockchain.add_transaction(tx1)?;
    nonce += 1;
    
    // User1 -> User3: 1.5 coins  
    let tx2 = Transaction::new_account(users[0].2, users[2].2, 150000000, nonce, 21000, 50, vec![]);
    blockchain.add_transaction(tx2)?;
    nonce += 1;
    
    // User1 -> User4: 3 coins
    let tx3 = Transaction::new_account(users[0].2, users[3].2, 300000000, nonce, 21000, 50, vec![]);
    blockchain.add_transaction(tx3)?;
    
    // Mine block to apply transactions
    let miner_addr = users[4].2; // User5 is the miner
    blockchain.mine_block(miner_addr)?;
    
    println!("💰 State after transactions:");
    for (name, _, addr) in &users {
        let balance = blockchain.get_balance(addr);
        let nonce = blockchain.get_nonce(addr);
        println!("   {}: {} satoshis (nonce: {})", name, balance, nonce);
    }
    
    // Demonstrate state calculations
    let total_supply = blockchain.world_state().total_supply();
    println!("\n📊 State analysis:");
    println!("   Total supply: {} satoshis", total_supply);
    println!("   Active accounts: {}", blockchain.world_state().accounts().len());
    
    // Validate state consistency
    blockchain.world_state().validate()?;
    println!("   ✅ State validation passed");
    
    // Demonstrate state root calculation
    let mut state_copy = blockchain.world_state().clone();
    let state_root = state_copy.state_root();
    println!("   State root: {}", state_root);
    
    Ok(())
}

async fn demo_validation() -> Result<()> {
    println!("\n🛡️  5. Validation Demo");
    println!("-".repeat(30));
    
    let mut blockchain = create_demo_blockchain()?;
    let validator = &blockchain.validator;
    
    println!("🔍 Testing transaction validation...");
    
    let alice_keypair = generate_keypair();
    let bob_keypair = generate_keypair(); 
    let alice_addr = public_key_to_address(alice_keypair.public_key(), AddressType::Base58);
    let bob_addr = public_key_to_address(bob_keypair.public_key(), AddressType::Base58);
    
    // Fund Alice
    fund_account(&mut blockchain, alice_addr, 1000000000)?;
    
    // Test 1: Valid transaction
    let valid_tx = Transaction::new_account(alice_addr, bob_addr, 100000000, 0, 21000, 50, vec![]);
    let result = blockchain.add_transaction(valid_tx.clone());
    println!("   ✅ Valid transaction: {:?}", result.is_ok());
    
    // Test 2: Insufficient balance
    let invalid_balance_tx = Transaction::new_account(
        alice_addr, bob_addr, 2000000000, 1, 21000, 50, vec![]
    );
    let result = blockchain.add_transaction(invalid_balance_tx);
    println!("   ❌ Insufficient balance: {:?}", result.is_err());
    
    // Test 3: Invalid nonce (too low)
    blockchain.mine_block(bob_addr)?; // Apply the first transaction
    
    let invalid_nonce_tx = Transaction::new_account(
        alice_addr, bob_addr, 50000000, 0, 21000, 50, vec![] // Nonce 0 again
    );
    let result = blockchain.add_transaction(invalid_nonce_tx);
    println!("   ❌ Invalid nonce: {:?}", result.is_err());
    
    println!("\n🔍 Testing block validation...");
    
    // Create a valid block manually
    let coinbase_tx = Transaction::new_coinbase(alice_addr, 2500000000, blockchain.height() + 1);
    let valid_block = Block::new(
        blockchain.get_chain_head().unwrap().id(),
        vec![coinbase_tx],
        1,
        blockchain.height() + 1,
        blockchain.config.chain_id,
    )?;
    
    println!("   ✅ Valid block structure: {:?}", valid_block.verify_structure().is_ok());
    
    // Test full chain validation
    println!("\n🔍 Validating entire chain...");
    let chain_validation = blockchain.validate_chain();
    println!("   ✅ Full chain validation: {:?}", chain_validation.is_ok());
    
    // Show validation rules
    let rules = validator.rules();
    println!("\n⚙️  Current validation rules:");
    println!("   Max block size: {} bytes", rules.max_block_size);
    println!("   Min transaction fee: {} satoshis", rules.min_transaction_fee);
    println!("   Coinbase maturity: {} blocks", rules.coinbase_maturity);
    println!("   Max block time drift: {} seconds", rules.max_block_time_drift);
    
    Ok(())
}

async fn demo_performance() -> Result<()> {
    println!("\n🚀 6. Performance Benchmarks");
    println!("-".repeat(30));
    
    let mut blockchain = create_demo_blockchain()?;
    
    // Benchmark transaction creation
    println!("⏱️  Benchmarking transaction creation...");
    let tx_creation_start = std::time::Instant::now();