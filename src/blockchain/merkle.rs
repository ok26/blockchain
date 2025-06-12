use crate::sha256::Sha256;
use super::transaction::Transaction;

#[derive(Clone, Debug)]
pub struct MerkleNode {
    hash: Sha256,
    left: Option<Box<MerkleNode>>,
    right: Option<Box<MerkleNode>>
}

impl MerkleNode {
    pub fn is_leaf_node(&self) -> bool {
        return self.left.is_none() || self.right.is_none();
    }
}

#[derive(Clone, Debug)]
pub struct MerkleTree {
    root: MerkleNode,
    transactions: Vec<Transaction>
}

impl MerkleTree {
    pub fn new(transactions: Vec<Transaction>) -> MerkleTree {
        let mut hashes: Vec<Sha256> = transactions.iter().map(|tx| tx.hash()).collect();
        println!("{}", hashes[0]);

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
        let left = Self::parse_hashes(hashes[..mid].to_vec()).unwrap();
        let right = Self::parse_hashes(hashes[mid..].to_vec()).unwrap();

        let concat = left.clone().hash.bytes().to_vec()
            .into_iter()
            .chain(right.clone().hash.bytes().to_vec())
            .collect::<Vec<u8>>();

        Some(MerkleNode {
            hash: Sha256::hash(&concat),
            left: Some(Box::new(left)),
            right: Some(Box::new(right))
        })
    }

    pub fn root_hash(&self) -> &Sha256 {
        &self.root.hash
    }

    pub fn transactions(&self) -> &Vec<Transaction> {
        &self.transactions
    }

    // Will return the hashes of the opposing child combined with either the value 0 or 1 representing left or right
    pub fn get_branch_hashes(&self, transaction: Transaction) -> Option<Vec<(Sha256, usize)>> {
        let transaction_idx = self.transactions.iter().position(|tx| tx == &transaction);
        if transaction_idx.is_none() {
            return None;
        }
        let transaction_idx = transaction_idx.unwrap();
        let transaction_cnt = self.transactions.len();
        let mut res = vec![];
        self.recursive_find_transaction(&self.root, &mut res, transaction_cnt, transaction_idx);
        res.reverse();
        Some(res)
    }

    fn recursive_find_transaction(&self, node: &MerkleNode, branch: &mut Vec<(Sha256, usize)>, tx_cnt: usize, tx_idx: usize) {
        
        if node.is_leaf_node() {
            return;
        }

        let tx_cnt = (tx_cnt + 1) / 2;
        if tx_idx >= tx_cnt {
            branch.push((node.left.as_ref().unwrap().hash.clone(), 0));
            self.recursive_find_transaction(node.right.as_ref().unwrap(), branch, tx_cnt, tx_idx % tx_cnt);
        }
        else {
            branch.push((node.right.as_ref().unwrap().hash.clone(), 1));
            self.recursive_find_transaction(node.left.as_ref().unwrap(), branch, tx_cnt, tx_idx);
        }
    }

    pub fn verify_transaction_branch(tx: Transaction, branch: Vec<(Sha256, usize)>, root_hash: Sha256) -> bool {
        let tx_hash = tx.hash();
        println!("{}", tx_hash);
        let mut node_hash = tx_hash.clone();
        for (hash, side) in branch {
            
            let mut branch_hash = hash;

            // If side == 0 it means that the opposing hash (branch_hash) is to 
            // the left in the tree. Later we concat "node_hash" to the left and
            // therefore we need to swap if side == 0
            if side == 0 {
                std::mem::swap(&mut branch_hash, &mut node_hash);
            }

            let concat = node_hash.clone().bytes().to_vec()
                .into_iter()
                .chain(branch_hash.bytes().to_vec())
                .collect::<Vec<u8>>();

            node_hash = Sha256::hash(&concat);
            println!("{}", node_hash);
        }

        println!("{}", root_hash);
        node_hash == root_hash
    }
}

#[cfg(test)]
mod tests {
    use crate::ecdsa;
    use super::*;

    #[test]
    fn test_retrieve_all_branch() {
        let mut transactions = vec![];
        let (pubkey, _) = ecdsa::generate_keypair();
        for _ in 0..10 {
            transactions.push(Transaction::get_coinbase(pubkey.clone(), 10));
        }

        let merkle = MerkleTree::new(transactions.clone());

        for tx in transactions {
            let branch = merkle.get_branch_hashes(tx.clone());
            assert!(branch.is_some());
            let branch = branch.unwrap();
            assert!(MerkleTree::verify_transaction_branch(tx, branch, merkle.root_hash().clone()));
        }
    }

    #[test]
    fn test_with_single_transaction() {
        let transactions = vec![Transaction::get_coinbase(ecdsa::generate_keypair().0, 1000)];
        let merkle = MerkleTree::new(transactions.clone());
        let branch = merkle.get_branch_hashes(transactions[0].clone());
        assert!(branch.is_some());
        let branch = branch.unwrap();
        assert!(MerkleTree::verify_transaction_branch(transactions[0].clone(), branch, merkle.root_hash().clone()));
    }

    #[test]
    fn test_with_invalid_transaction() {
        let mut transactions = vec![];
        let (pubkey, _) = ecdsa::generate_keypair();
        for _ in 0..10 {
            transactions.push(Transaction::get_coinbase(pubkey.clone(), 10));
        }

        let merkle = MerkleTree::new(transactions.clone());
        let invalid_transaction = Transaction::get_coinbase(pubkey, 10);
        let branch = merkle.get_branch_hashes(invalid_transaction);
        assert!(branch.is_none());
    }

    #[test]
    fn test_with_modified_tree() {
        let mut transactions = vec![];
        let (pubkey, _) = ecdsa::generate_keypair();
        for _ in 0..10 {
            transactions.push(Transaction::get_coinbase(pubkey.clone(), 10));
        }

        let mut merkle = MerkleTree::new(transactions.clone());
        merkle.root.left.as_mut().unwrap().hash = Sha256::hash(&[]);

        // All transactions to the right should fail if the left root-childs hash has been modified
        for tx in &transactions[5..] {
            let branch = merkle.get_branch_hashes(tx.clone());
            assert!(branch.is_some());
            let branch = branch.unwrap();
            assert!(!MerkleTree::verify_transaction_branch(tx.clone(), branch, merkle.root_hash().clone()));
        }

        // While all to the left still should succeed
        for tx in &transactions[..5] {
            let branch = merkle.get_branch_hashes(tx.clone());
            assert!(branch.is_some());
            let branch = branch.unwrap();
            assert!(MerkleTree::verify_transaction_branch(tx.clone(), branch, merkle.root_hash().clone()));
        }
    }
}