use crate::{sha256::Sha256, util};
use super::{merkle::MerkleTree, transaction::Transaction};

const DEFAULT_DIFFICULTY: u64 = 5;

pub struct Block {
    pub timestamp: u64,
    pub hash: Sha256,
    pub previous_block_hash: Sha256,
    pub nonce: u64,
    pub difficulty: u64,
    pub merkle_tree: MerkleTree
}

impl Block {
    pub fn new_genesis(coinbase: Transaction) -> Block {
        Block {
            timestamp: util::timestamp(),
            hash: Sha256::hash(&[]),
            previous_block_hash: Sha256::hash(&[]),
            nonce: 0,
            difficulty: DEFAULT_DIFFICULTY,
            merkle_tree: MerkleTree::new(vec![coinbase])
        }
    }

    pub fn new(previous_block_hash: Sha256, transactions: Vec<Transaction>) -> Block {
        let merkle_tree = MerkleTree::new(transactions);
        Block {
            timestamp: util::timestamp(),
            hash: Sha256::hash(&[]),
            previous_block_hash,
            nonce: 0,
            difficulty: DEFAULT_DIFFICULTY,
            merkle_tree
        }
    }

    pub fn mine(&mut self) {
        loop {
            self.timestamp = util::timestamp();
            let hash = self.hash();
            if hash.is_valid(self.difficulty) {
                self.hash = hash;
                break;
            }
            self.nonce += 1;
        }
    }

    pub fn hash(&self) -> Sha256 {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(self.previous_block_hash.bytes());
        bytes.extend_from_slice(self.merkle_tree.root_hash().bytes());
        bytes.extend_from_slice(&self.timestamp.to_be_bytes());
        bytes.extend_from_slice(&self.nonce.to_be_bytes());
        bytes.extend_from_slice(&self.difficulty.to_be_bytes());
        Sha256::hash(&bytes)
    }
}