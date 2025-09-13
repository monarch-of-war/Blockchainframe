use blockchain_crypto::hashing::Hashable;
use serde::{Serialize, Deserialize};
use std::collections::VecDeque;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub from: String,
    pub to: String,
    pub amount: u64,
    pub signature: Vec<u8>,
}

impl Hashable for Transaction {
    fn hash(&self) -> Vec<u8> {
        use blockchain_crypto::hashing::sha256;
        let data = format!("{}{}{}{:?}", self.from, self.to, self.amount, self.signature);
        sha256(data.as_bytes())
    }
}

#[derive(Default)]
pub struct Mempool {
    transactions: VecDeque<Transaction>,
}

impl Mempool {
    pub fn new() -> Self {
        Self { transactions: VecDeque::new() }
    }

    pub fn add_transaction(&mut self, tx: Transaction) {
        self.transactions.push_back(tx);
    }

    pub fn collect(&mut self, max: usize) -> Vec<Transaction> {
        let mut txs = Vec::new();
        for _ in 0..max {
            if let Some(tx) = self.transactions.pop_front() {
                txs.push(tx);
            } else {
                break;
            }
        }
        txs
    }
}
