// blockchain-cli/src/main.rs
use blockchain_core::{Block, Transaction, Blockchain};
use blockchain_crypto::{KeyPair, Signature};
use blockchain_network::P2PNode;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "blockchain-node")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Start { port: u16 },
    Mine,
    Wallet,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Start { port } => {
            println!("Starting blockchain node on port {}", port);
            // Use library crates to start the node
            let blockchain = Blockchain::new();
            let node = P2PNode::new(port, blockchain);
            node.start().await?;
        }
        Commands::Mine => {
            println!("Starting mining...");
            // Use consensus crate for mining
        }
        Commands::Wallet => {
            println!("Opening wallet interface...");
            // Use wallet crate
        }
    }
    
    Ok(())
}