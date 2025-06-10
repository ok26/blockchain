use std::collections::HashMap;
use block::Block;
use merkle::MerkleTree;
use transaction::{Transaction, TxOutput};
use crate::{ecdsa, sha256::Sha256};

pub mod block;
pub mod merkle;
pub mod transaction;

pub const MINING_REWARD: u64 = 50;

#[derive(Debug, PartialEq)]
pub enum TransactionError {
    InvalidSignature,
    InsufficientFunds,
    UnallowedTransaction,
    MismatchedOutput
}

#[derive(Debug)]
pub enum BlockError {
    InvalidHash,
    InvalidMerkleRoot,
    InvalidPreviousBlockHash,
    InvalidCoinbase,
    InvalidTransactions(Vec<TransactionError>)
}

#[derive(Clone)]
pub struct Blockchain {
    pub blocks: Vec<Block>,
    utxo: HashMap<Sha256, Vec<TxOutput>>,
}

impl Blockchain {
    pub fn new(coinbase: Transaction) -> Self {
        let mut blockchain = Self {
            blocks: vec![],
            utxo: HashMap::new(),
        };
        let mut block = blockchain.create_block(coinbase, vec![]);
        block.mine();
        blockchain.add_block(block).unwrap();
        blockchain
    }

    pub fn create_block(&self, coinbase: Transaction, transactions: Vec<Transaction>) -> Block {
        let previous_block_hash = if self.blocks.is_empty() {
            Sha256::hash(&[])
        } else {
            self.blocks.last().unwrap().hash()
        };
        Block::new(previous_block_hash, {
            let mut txs = Vec::with_capacity(1 + transactions.len());
            txs.push(coinbase);
            txs.extend(transactions);
            txs
        })
    }

    pub fn add_block(&mut self, block: Block) -> Result<(), BlockError> {
        self.verify_new_block(&block)?;
        for transaction in block.merkle_tree.transactions() {
            self.utxo.insert(transaction.hash(), transaction.outputs.clone());
            for input in &transaction.inputs {
                let v = self.utxo.get_mut(&input.txid).unwrap();
                v[input.vout as usize].spent = true;
                if v.iter().all(|output| output.spent) {
                    self.utxo.remove(&input.txid);
                }
            }
        }
        self.blocks.push(block);
        Ok(())
    }

    pub fn verify_new_transaction(&self, tx: &transaction::Transaction) -> Result<(), TransactionError> {
        let mut total_input = 0;
        for (i, input) in tx.inputs.iter().enumerate() {
            let ref_output = self.utxo.get(&input.txid);
            if ref_output.is_none() {
                return Err(TransactionError::InsufficientFunds);
            }
            let ref_output = ref_output.unwrap().get(input.vout as usize);
            if ref_output.is_none() || ref_output.unwrap().spent {
                return Err(TransactionError::InsufficientFunds);
            }
            let ref_output = ref_output.unwrap();
            if ref_output.script_pubkey != input.script_sig.1 {
                return Err(TransactionError::UnallowedTransaction);
            }

            let hash = tx.get_input_hash(i, &ref_output.script_pubkey);
            if !ecdsa::verify(input.script_sig.0, hash.bytes(), &input.script_sig.1) {
                return Err(TransactionError::InvalidSignature);
            }

            total_input += ref_output.value;
        }

        let mut total_output = 0;
        for output in &tx.outputs {
            total_output += output.value;
        }

        if tx.is_coinbase() && total_output == MINING_REWARD {
            return Ok(());
        }

        if total_input != total_output {
            return Err(TransactionError::MismatchedOutput);
        }

        return Ok(());
    }

    fn verify_new_block(&self, block: &Block) -> Result<(), BlockError> {
        if !(self.blocks.is_empty() || block.previous_block_hash == self.blocks.last().unwrap().hash) {
            return Err(BlockError::InvalidPreviousBlockHash);
        }

        if block.hash() != block.hash || !block.hash.is_valid(block.difficulty) {
            return Err(BlockError::InvalidHash);
        }

        if MerkleTree::new(block.merkle_tree.transactions().clone()).root_hash() != block.merkle_tree.root_hash() {
            return Err(BlockError::InvalidMerkleRoot);
        }

        let mut transaction_errors = Vec::new();
        let mut coinbase_cnt = 0;
        for tx in block.merkle_tree.transactions() {
            if tx.is_coinbase() {
                coinbase_cnt += 1;
            }
            let _ = self.verify_new_transaction(tx).map_err(|e| {
                transaction_errors.push(e);
            });
        }

        if coinbase_cnt != 1 {
            return Err(BlockError::InvalidCoinbase)
        }

        if transaction_errors.len() != 0 {
            return Err(BlockError::InvalidTransactions(transaction_errors));
        }
        
        return Ok(());
    }

    pub fn has_transaction(&self, tx: &Transaction) -> bool {
        if let Some(utxo) = self.utxo.get(&tx.hash()) {
            return utxo == &tx.outputs;
        }
        else {
            return false;
        }
    }

    pub fn get_user_funds(&self, pubkey: &ecdsa::ECDSAPublicKey) -> Vec<(Sha256, u32, u64)> {
        let mut funds = Vec::new();
        for (txid, outputs) in &self.utxo {
            for (vout, output) in outputs.iter().enumerate() {
                if output.script_pubkey == *pubkey {
                    funds.push((txid.clone(), vout as u32, output.value));
                }
            }
        }
        funds
    }

    pub fn get_utxo(&self) -> HashMap<Sha256, Vec<TxOutput>> {
        self.utxo.clone()
    }

    pub fn set_output_spent(&mut self, txid: &Sha256, vout: u32, spent: bool) {
        if let Some(outputs) = self.utxo.get_mut(txid) {
            if let Some(output) = outputs.get_mut(vout as usize) {
                output.spent = spent;
            }
        }
    }
}

impl std::fmt::Debug for Blockchain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut res = String::from("Blockchain: \n");
        for block in &self.blocks {
            res.push_str(format!("{:?}\n", block).as_str());
        }
        write!(f, "{}", res)
    }
}