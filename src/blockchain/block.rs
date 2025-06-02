use crate::sha256::Sha256;
use super::merkle::MerkleTree;

pub struct Block {
    pub index: u32,
    pub timestamp: u128,
    pub hash: Sha256,
    pub previous_block_hash: Sha256,
    pub nonce: u64,
    pub merkle_tree: MerkleTree 
}

impl Block {
    pub fn new_genesis() -> Block {
        Block {
            index: 0,
            timestamp: 0,
            hash: Sha256::hash(&[]),
            previous_block_hash: Sha256::hash(&[]),
            nonce: 0,
            merkle_tree: MerkleTree::new(vec![])
        }
    }
}