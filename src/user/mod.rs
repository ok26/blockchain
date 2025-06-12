use crate::{blockchain::{merkle::MerkleTree, transaction::{Transaction, TxInput, TxOutput}}, ecdsa::{self, point::AffinePoint, ECDSAPrivateKey, ECDSAPublicKey}, sha256::Sha256};

#[derive(Debug)]
pub enum UserError {
    InsufficientFunds,
}

// txid is the hash of the transaction where this fund is from
// value is the amount of coins in this fund
// vout is the index of the output in that transaction
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

    pub fn try_transaction(&self, recievers: &Vec<(ECDSAPublicKey, u64)>) -> Result<Transaction, UserError> {
        let mut total_input = 0;
        let total_output: u64 = recievers.iter().map(|(_, value)| *value).sum();
        let mut transaction = Transaction::new();
        for fund in &self.funds {
            total_input += fund.value;
            transaction.add_input(self.get_input(fund));
            if total_input >= total_output {

                for (reciever, value) in recievers {
                    transaction.add_output(TxOutput {
                        value: *value,
                        script_pubkey: reciever.clone(),
                        spent: false,
                    });
                }
                
                let change = total_input - total_output;
                if change != 0 {
                    transaction.add_output(TxOutput {
                        value: change,
                        script_pubkey: self.public_key.clone(),
                        spent: false,
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

    pub fn update_funds(&mut self, tx: &Transaction) {
        let txid = tx.hash();
        let mut value = 0;
        let mut vout = 0;

        for (i, output) in tx.outputs.iter().enumerate() {
            if output.script_pubkey == self.public_key {
                value += output.value;
                vout = i as u32;
            }
        }

        if value != 0 {
            self.funds.push(Fund {
                txid,
                value,
                vout
            });
        }
        
        for input in &tx.inputs {
            self.funds.retain(|f| !(f.txid == input.txid && f.vout == input.vout));
        }
    }

    pub fn update_funds_from_chain(&mut self, funds: &Vec<(Sha256, u32, u64)>) {
        self.funds.clear();
        for (txid, vout, value) in funds {
            self.funds.push(Fund {
                txid: txid.clone(),
                value: *value,
                vout: *vout
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

    pub fn verify_transaction_prescence(&self, tx: Transaction, branch: Vec<(Sha256, usize)>, root_hash: Sha256) -> bool {
        MerkleTree::verify_transaction_branch(tx, branch, root_hash)
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_creation() {
        let keys = ecdsa::generate_keypair();
        let user = User::new("User", keys);
        assert_eq!(user.name, "User");
        assert_eq!(user.public_key.get_der_encoding().len(), 70);
        assert_eq!(user.private_key.get_der_encoding().len(), 36);
    }

    #[test]
    fn test_user_funds() {
        let keys = ecdsa::generate_keypair();
        let mut user = User::new("Name", keys);
        
        let tx = Transaction::get_coinbase(user.public_key.clone(), 100);
        user.update_funds(&tx);
        assert_eq!(user.get_funds(), 100);
        
        let recievers = vec![(ecdsa::generate_keypair().0, 150)];
        assert!(user.try_transaction(&recievers).is_err());
        
        let tx2 = Transaction::get_coinbase(user.public_key.clone(), 50);
        user.update_funds(&tx2);
        
        assert_eq!(user.get_funds(), 150);
        assert!(user.try_transaction(&recievers).is_ok());
    }

    #[test]
    fn test_user_signing() {
        let keys = ecdsa::generate_keypair();
        let mut user = User::new("TestUser", keys);
        
        let coinbase = Transaction::get_coinbase(user.public_key.clone(), 100);
        user.update_funds(&coinbase);

        let recievers = vec![(ecdsa::generate_keypair().0, 50)];
        let transaction = user.try_transaction(&recievers).unwrap();
        
        assert_eq!(transaction.outputs.len(), 2); // One for the receiver and one for change
        assert_eq!(transaction.outputs[0].value, 50);
        assert_eq!(transaction.outputs[1].value, user.get_funds() - 50);
        
        for (i, input) in transaction.inputs.iter().enumerate() {
            let hash = transaction.get_input_hash(i, &user.public_key);
            assert!(ecdsa::verify(input.script_sig.0, hash.bytes(), &user.public_key));
        }
    }

    #[test]
    fn test_user_double_spending() {
        let keys = ecdsa::generate_keypair();
        let mut user = User::new("DoubleSpender", keys);
        
        let coinbase = Transaction::get_coinbase(user.public_key.clone(), 100);
        user.update_funds(&coinbase);

        let recievers1 = vec![(ecdsa::generate_keypair().0, 50)];
        let transaction1 = user.try_transaction(&recievers1).unwrap();
        user.update_funds(&transaction1);

        let recievers2 = vec![(ecdsa::generate_keypair().0, 60)];
        assert!(user.try_transaction(&recievers2).is_err()); // Should fail due to insufficient funds

        let recievers3 = vec![(ecdsa::generate_keypair().0, 50)];
        assert!(user.try_transaction(&recievers3).is_ok()); // Should succeed with remaining funds
    }
}