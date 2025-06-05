use crate::{blockchain::transaction::{Transaction, TxInput, TxOutput}, ecdsa::{self, point::AffinePoint, ECDSAPrivateKey, ECDSAPublicKey}, sha256::Sha256};

#[derive(Debug)]
pub enum UserError {
    InsufficientFunds,
}

pub struct Fund {
    pub txid: Sha256,
    pub value: u64,
    pub vout: u32
}

pub struct User {
    pub name: String,
    pub public_key: ECDSAPublicKey,
    pub private_key: ECDSAPrivateKey,
    pub funds: Vec<Fund>
}

impl User {
    pub fn new(name: &str, keys: (ECDSAPublicKey, ECDSAPrivateKey)) -> Self {
        User {
            name: name.to_string(),
            public_key: keys.0,
            private_key: keys.1,
            funds: vec![]
        }
    }

    pub fn try_transaction(&self, reciever: &ECDSAPublicKey, value: u64) -> Result<Transaction, UserError> {
        let mut total_input = 0;
        let mut transaction = Transaction::new();
        for fund in &self.funds {
            total_input += fund.value;
            transaction.add_input(self.get_input(fund));
            if total_input >= value {
                transaction.add_output(TxOutput {
                    value: value,
                    script_pubkey: reciever.clone(),
                });
                let change = total_input - value;
                if change > 0 {
                    transaction.add_output(TxOutput {
                        value: change,
                        script_pubkey: self.public_key.clone(),
                    });
                }

                return Ok(self.sign_transaction(&transaction));
            }
        }
        
        Err(UserError::InsufficientFunds)
    }

    fn get_input(&self, fund: &Fund) -> TxInput {
        TxInput {
            txid: fund.txid.clone(),
            vout: fund.vout,
            script_sig: (AffinePoint::infinity(), self.public_key.clone()),
        }
    }

    fn sign_transaction(&self, transaction: &Transaction) -> Transaction {
        let mut signed_transaction = transaction.clone();
        for (i, input) in signed_transaction.inputs.iter_mut().enumerate() {
            let hash = transaction.get_input_hash(i, &self.public_key);
            input.script_sig.0 = ecdsa::sign(hash.bytes(), &self.private_key);
        }
        signed_transaction
    }

    pub fn update_funds(&mut self, transaction: &Transaction) {
        for input in &transaction.inputs {
            self.funds.retain(|f| !(f.txid == input.txid && f.vout == input.vout));
        }
        
        for (i, output) in transaction.outputs.iter().enumerate() {
            if output.script_pubkey != self.public_key {
                continue;
            }
            self.funds.push(Fund {
                txid: transaction.hash(),
                value: output.value,
                vout: i as u32,
            });
        }
    }

    pub fn get_funds(&self) -> u64 {
        let mut total = 0;
        for fund in &self.funds {
            total += fund.value;
        }
        total
    }
}