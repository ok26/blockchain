use crate::{blockchain::{block::Block, transaction::Transaction, BlockError, Blockchain, TransactionError, MINING_REWARD}, ecdsa::{ECDSAPrivateKey, ECDSAPublicKey}, user::User};

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
            user: User::new(name, keys)
        }
    }

    pub fn add_transaction(&mut self, transaction: Transaction) -> Result<(), TransactionError> {
        self.blockchain.verify_transaction(&transaction)?;
        self.current_transactions.push(transaction);
        Ok(())
    }

    pub fn mine(&mut self) -> Block {
        let coinbase = Transaction::get_coinbase(self.user.public_key.clone(), MINING_REWARD);
        let mut block = self.blockchain.create_block(coinbase.clone(), self.current_transactions.clone());
        block.mine();
        self.blockchain.add_block(block.clone());
        self.current_transactions.clear();
        self.user.update_funds(&coinbase);
        block
    }

    pub fn accept_block(&mut self, block: Block) -> Result<(), BlockError> {
        self.blockchain.verify_block(&block)?;
        self.blockchain.add_block(block);
        Ok(())
    }

    pub fn is_transaction_confirmed(&self, tx: &Transaction) -> bool {
        self.blockchain.has_transaction(tx)
    }
}