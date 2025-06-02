use core::hash;

use crate::sha256::Sha256;
use super::transaction::Transaction;

#[derive(Clone)]
pub struct MerkleNode {
    hash: Sha256,
    left: Option<Box<MerkleNode>>,
    right: Option<Box<MerkleNode>>
}

pub struct MerkleTree {
    root: MerkleNode
}

impl MerkleTree {
    pub fn new(hashes: Vec<Sha256>) -> MerkleTree {
        MerkleTree {
            root: Self::parse_hashes(hashes).unwrap_or_else(|| MerkleNode {
                hash: Sha256::hash(&[]),
                left: None,
                right: None
            })
        }
    }

    fn parse_hashes(hashes: Vec<Sha256>) -> Option<MerkleNode> {
        if hashes.is_empty() {
            return None;
        }

        if hashes.len() == 1 {
            return Some(MerkleNode {
                hash: hashes[0].clone(),
                left: None,
                right: None
            });
        }

        let mut hashes = hashes;

        if hashes.len() % 2 == 1 {
            // If the number of hashes is odd, duplicate the last hash
            let last_hash = hashes.last().unwrap();
            hashes.push(last_hash.clone());
        }

        let mid = hashes.len() / 2;
        let left = Self::parse_hashes(hashes[..mid].to_vec());
        let right = Self::parse_hashes(hashes[mid..].to_vec());
        let concat = left.clone().unwrap().hash.bytes().to_vec()
            .into_iter()
            .chain(right.clone().unwrap().hash.bytes().to_vec())
            .collect::<Vec<u8>>();

        Some(MerkleNode {
            hash: Sha256::hash(&concat),
            left: left.map(Box::new),
            right: right.map(Box::new)
        })
    }

    pub fn from_transactions(transactions: Vec<Transaction>) -> MerkleTree {
        let hashes: Vec<Sha256> = transactions.iter().map(|tx| tx.hash()).collect();
        MerkleTree::new(hashes)
    }
}