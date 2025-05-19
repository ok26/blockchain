use crate::{
    transaction::Transaction,
    common::*
};

pub struct MerkleTree {
    root_hash: HashBytes,
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