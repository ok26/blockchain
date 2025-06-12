use crate::{
    blockchain::{block::Block, transaction::Transaction, BlockError, Blockchain, TransactionError, MINING_REWARD}, 
    ecdsa::{ECDSAPrivateKey, ECDSAPublicKey}, sha256::Sha256, user::User
};

pub struct Node {
    blockchain: Blockchain,
    current_transactions: Vec<Transaction>,
    pub user: User
}

impl Node {
    pub fn new(name: &str, history: Blockchain, keys: (ECDSAPublicKey, ECDSAPrivateKey)) -> Self {
        Node {
            blockchain: history,
            current_transactions: Vec::new(),
            user: User::new(name, keys),
        }
    }

    pub fn add_transaction(&mut self, transaction: Transaction) -> Result<(), TransactionError> {
        self.blockchain.verify_new_transaction(&transaction)?;
        for input in &transaction.inputs {
            self.blockchain.set_output_spent(&input.txid, input.vout, true);
        }
        self.current_transactions.push(transaction);
        Ok(())
    }

    pub fn remove_transaction(&mut self, txid: &Sha256) -> Result<(), ()> {
        if let Some(pos) = self.current_transactions.iter().position(|tx| tx.hash() == *txid) {
            let transaction = self.current_transactions.remove(pos);
            for input in &transaction.inputs {
                self.blockchain.set_output_spent(&input.txid, input.vout, false);
            }
            Ok(())
        } else {
            Err(())
        }
    }

    pub fn clear_current_transactions(&mut self) {
        for tx in self.current_transactions.clone() {
            let _ = self.remove_transaction(&tx.hash());
        }
    }

    pub fn mine(&mut self) -> Block {
        let coinbase = Transaction::get_coinbase(self.user.public_key.clone(), MINING_REWARD);
        let mut block = self.blockchain.create_block(coinbase.clone(), self.current_transactions.clone());
        block.mine();
        self.clear_current_transactions();
        self.blockchain.add_block(block.clone()).unwrap();
        self.user.update_funds(&coinbase);
        block
    }

    pub fn accept_block(&mut self, block: Block) -> Result<(), BlockError> {
        let transactions = block.merkle_tree.transactions();

        // Remove confirmed transactions from current transactions
        for tx in block.merkle_tree.transactions() {
            if self.current_transactions.iter().any(|t| t.hash() == tx.hash()) {
                self.remove_transaction(&tx.hash()).unwrap();
            }
        }

        let res = self.blockchain.add_block(block.clone());

        if res.is_err() {
            // Add all transactions back to current transactions
            for tx in transactions {
                self.current_transactions.push(tx.clone());
            }

            return Err(res.err().unwrap());
        }

        Ok(())
    }

    pub fn is_transaction_confirmed(&self, tx: &Transaction) -> bool {
        self.blockchain.has_transaction(tx)
    }

    pub fn get_funds_from_chain(&self, user: &ECDSAPublicKey) -> Vec<(Sha256, u32, u64)> {
        self.blockchain.get_user_funds(user)
    }

    // Might need to find the block in another way in the future
    pub fn get_verifiyng_transaction_branch(&self, tx: Transaction, block_idx: usize) -> Option<Vec<(Sha256, usize)>> {
        if block_idx >= self.blockchain.blocks.len() {
            return None;
        }
        self.blockchain.blocks[block_idx].merkle_tree.get_branch_hashes(tx)
    }
}

#[cfg(test)]
mod tests {
    use crate::{ecdsa, user::Fund};

    use super::*;

    #[test]
    fn test_node_creation() {
        let keys = ecdsa::generate_keypair();
        let blockchain = Blockchain::new(Transaction::get_coinbase(keys.0.clone(), MINING_REWARD));
        let node = Node::new("TestNode", blockchain, keys);
        assert_eq!(node.user.name, "TestNode");
    }

    #[test]
    fn test_node_mining() {
        let keys = ecdsa::generate_keypair();
        let blockchain = Blockchain::new(Transaction::get_coinbase(keys.0.clone(), MINING_REWARD));
        let mut node = Node::new("Miner", blockchain, keys);
        
        let block = node.mine();
        assert_eq!(block.merkle_tree.transactions().len(), 1);
        assert_eq!(block.merkle_tree.transactions()[0].outputs[0].value, MINING_REWARD);
        assert!(node.user.get_funds() == MINING_REWARD); // Previous is ignored if not queried

        node.user.update_funds_from_chain(&node.get_funds_from_chain(&node.user.public_key));
        assert!(node.user.get_funds() == 2 * MINING_REWARD);
    }

    #[test]
    fn test_node_add_transaction() {
        let keys = ecdsa::generate_keypair();
        let blockchain = Blockchain::new(Transaction::get_coinbase(keys.0.clone(), MINING_REWARD));
        let mut node = Node::new("TestNode", blockchain, keys);
        node.user.update_funds_from_chain(&node.get_funds_from_chain(&node.user.public_key));
        
        let recipient_keys = ecdsa::generate_keypair();
        let recievers = vec![(recipient_keys.0, MINING_REWARD)];
        let transaction = node.user.try_transaction(&recievers).unwrap();

        assert!(node.add_transaction(transaction).is_ok());

        // Verify chain can only be called with no current transactions
        // Mining the block will ensure this
        node.mine();
        assert_eq!(node.blockchain.verify_chain(), Ok(()));
    }

    #[test]
    fn test_add_invalid_transaction() {
        let keys = ecdsa::generate_keypair();
        let blockchain = Blockchain::new(Transaction::get_coinbase(keys.0.clone(), MINING_REWARD));
        let mut node = Node::new("TestNode", blockchain, keys);

        // Insert a dummy fund to allow transaction creation
        node.user.funds.push(Fund {
            txid: Sha256::hash(&[]),
            value: 3 * MINING_REWARD,
            vout: 0
        });
        
        let recipient_keys = ecdsa::generate_keypair();
        let recievers = vec![(recipient_keys.0, 3 * MINING_REWARD)]; // More than available funds
        let transaction = node.user.try_transaction(&recievers).unwrap();
        
        assert_eq!(node.add_transaction(transaction), Err(TransactionError::InsufficientFunds));

        // Ensure the transaction was not added to the blockchain
        assert_eq!(node.blockchain.verify_chain(), Ok(()));
    }

    #[test]
    fn test_double_spending() {
        let keys = ecdsa::generate_keypair();
        let blockchain = Blockchain::new(Transaction::get_coinbase(keys.0.clone(), MINING_REWARD));
        let mut node = Node::new("TestNode", blockchain, keys);        
        node.user.update_funds_from_chain(&node.get_funds_from_chain(&node.user.public_key));
        
        let recipient_keys1 = ecdsa::generate_keypair();
        let recipient_keys2 = ecdsa::generate_keypair();
        
        let recievers1 = vec![(recipient_keys1.0, MINING_REWARD)];
        let recievers2 = vec![(recipient_keys2.0, MINING_REWARD)];
        
        let transaction1 = node.user.try_transaction(&recievers1).unwrap();
        let transaction2 = node.user.try_transaction(&recievers2).unwrap();
        
        assert!(node.add_transaction(transaction1).is_ok());
        assert_eq!(node.add_transaction(transaction2), Err(TransactionError::InsufficientFunds));

        node.mine();

        // Ensure the first transaction was mined and the second was not added
        // Check the entire chain
        assert_eq!(node.blockchain.verify_chain(), Ok(()));
    }

    #[test]
    fn test_blockchain_remove_utxo() {
        let keys = ecdsa::generate_keypair();
        let blockchain = Blockchain::new(Transaction::get_coinbase(keys.0.clone(), MINING_REWARD));
        let mut node = Node::new("TestNode", blockchain, keys);
        assert_eq!(node.blockchain.get_utxo().len(), 1);
        node.user.update_funds_from_chain(&node.get_funds_from_chain(&node.user.public_key));
        
        let recipient_keys = ecdsa::generate_keypair();
        let recievers = vec![(recipient_keys.0, MINING_REWARD)];
        let transaction = node.user.try_transaction(&recievers).unwrap();
        
        assert!(node.add_transaction(transaction).is_ok());
        
        node.mine();
        
        // Now only two unspent transactions should remain: the second coinbase and the transaction to the recipient
        assert_eq!(node.blockchain.get_utxo().len(), 2);

        // Check entire chain integrity
        assert_eq!(node.blockchain.verify_chain(), Ok(()));
    }

    #[test]
    fn test_invalid_signature() {
        let keys = ecdsa::generate_keypair();
        let blockchain = Blockchain::new(Transaction::get_coinbase(keys.0.clone(), MINING_REWARD));
        let mut node = Node::new("TestNode", blockchain, keys);
        
        node.user.update_funds_from_chain(&node.get_funds_from_chain(&node.user.public_key));

        let recievers = vec![(ecdsa::generate_keypair().0, 50)];
        let mut transaction = node.user.try_transaction(&recievers).unwrap();

        // Add an invalid input to the transaction, making the signature invalid
        let input = transaction.inputs[0].clone();
        transaction.add_input(input);

        assert_eq!(node.add_transaction(transaction), Err(TransactionError::InvalidSignature));
    }

    #[test]
    fn test_invalid_chain() {
        let keys = ecdsa::generate_keypair();
        let blockchain = Blockchain::new(Transaction::get_coinbase(keys.0.clone(), MINING_REWARD));
        let mut node = Node::new("TestNode", blockchain, keys);
        
        node.user.update_funds_from_chain(&node.get_funds_from_chain(&node.user.public_key));

        let recipient_keys = ecdsa::generate_keypair();
        let recievers = vec![(recipient_keys.0, 50)];
        let transaction = node.user.try_transaction(&recievers).unwrap();

        assert!(node.add_transaction(transaction).is_ok());
        node.mine();

        // Manually change the blockchain to create an invalid state
        node.blockchain.blocks[0].nonce += 1;

        assert_eq!(node.blockchain.verify_chain(), Err(BlockError::InvalidHash));
    }
}