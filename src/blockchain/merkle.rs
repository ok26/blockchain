use crate::sha256::Sha256;
use super::transaction::Transaction;

#[derive(Clone, Debug)]
pub struct MerkleNode {
    hash: Sha256,
    left: Option<Box<MerkleNode>>,
    right: Option<Box<MerkleNode>>
}

#[derive(Debug)]
pub struct MerkleTree {
    root: MerkleNode,
    transactions: Vec<Transaction>
}

impl MerkleTree {
    pub fn new(transactions: Vec<Transaction>) -> MerkleTree {
        let mut hashes: Vec<Sha256> = transactions.iter().map(|tx| tx.hash()).collect();

        // Pad with the last hash
        if hashes.len() == 1 {
            hashes.push(hashes[0].clone());
        }
        
        MerkleTree {
            root: Self::parse_hashes(hashes).unwrap_or_else(|| MerkleNode {
                hash: Sha256::hash(&[]),
                left: None,
                right: None
            }),
            transactions
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

    pub fn root_hash(&self) -> &Sha256 {
        &self.root.hash
    }

    pub fn transactions(&self) -> &Vec<Transaction> {
        &self.transactions
    }
}