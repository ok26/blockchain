use super::transaction::Transaction;

pub struct MerkleTree {
    root_hash: Vec<u8>,
    transactions: Vec<Transaction>
}

impl MerkleTree {
    pub fn new() -> MerkleTree {
        MerkleTree {
            root_hash: vec![]            ,
            transactions: vec![]
        }
    }
}