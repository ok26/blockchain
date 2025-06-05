use crate::blockchain::{block::Block, transaction::Transaction, Blockchain, TransactionError};

pub struct Node {
    blockchain: Blockchain,
    current_transactions: Vec<Transaction>,
}

impl Node {
    pub fn new(history: Blockchain) -> Self {
        Node {
            blockchain: history,
            current_transactions: Vec::new(),
        }
    }
}