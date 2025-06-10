use crate::{ecdsa::{point::AffinePoint, ECDSAPublicKey}, math::random, sha256::Sha256};

// txid is the hash of the transaction that created this input
// vout is the index of the output in that transaction
// script_sig is the signature and public key used to unlock this input
#[derive(Clone, Debug)]
pub struct TxInput {
    pub txid: Sha256,
    pub vout: u32,
    pub script_sig: (AffinePoint, ECDSAPublicKey),
}

// value is the amount of coins being sent
// script_pubkey is the public key that can unlock this output
#[derive(Clone, Debug, PartialEq)]
pub struct TxOutput {
    pub value: u64,
    pub script_pubkey: ECDSAPublicKey,
}

#[derive(Clone)]
pub struct Transaction {
    pub inputs: Vec<TxInput>,
    pub outputs: Vec<TxOutput>,
}

impl Transaction {
    pub fn new() -> Transaction {
        Transaction {
            inputs: Vec::new(),
            outputs: Vec::new(),
        }
    }

    pub fn get_coinbase(miner: ECDSAPublicKey, value: u64) -> Transaction {
        let mut tx = Transaction::new();
        tx.outputs.push(TxOutput {
            value,
            script_pubkey: miner,
        });
        tx
    }

    pub fn add_input(&mut self, input: TxInput) {
        self.inputs.push(input);
    }

    pub fn add_output(&mut self, output: TxOutput) {
        self.outputs.push(output);
    }

    pub fn is_coinbase(&self) -> bool {
        self.inputs.is_empty()
    }

    pub fn serialize_for_input(&self, idx: usize, utxo_key: &ECDSAPublicKey) -> Vec<u8> {
        let mut serialized = Vec::new();
        
        serialized.push(self.inputs.len() as u8);
        for (i, input) in self.inputs.iter().enumerate() {
            serialized.extend_from_slice(input.txid.bytes());
            serialized.extend_from_slice(&input.vout.to_be_bytes());
            if i == idx {
                serialized.extend_from_slice(&utxo_key.get_der_encoding());
            }
        }
        serialized.push(self.outputs.len() as u8);
        for output in &self.outputs {
            serialized.extend_from_slice(&output.value.to_be_bytes());
            serialized.extend_from_slice(&output.script_pubkey.get_der_encoding());
        }
        serialized
    }

    pub fn get_input_hash(&self, idx: usize, utxo_key: &ECDSAPublicKey) -> Sha256 {
        let serialized = self.serialize_for_input(idx, utxo_key);
        Sha256::hash(&serialized)
    }

    pub fn hash(&self) -> Sha256 {
        let mut serialized = Vec::new();
        serialized.push(self.inputs.len() as u8);
        for input in &self.inputs {
            serialized.extend_from_slice(input.txid.bytes());
            serialized.extend_from_slice(&input.vout.to_be_bytes());
            serialized.extend_from_slice(&input.script_sig.0.get_bytes());
            serialized.extend_from_slice(&input.script_sig.1.get_der_encoding());
        }

        // If this is a coinbase, we add random bytes to distinguish it 
        // from the same transaction in different blocks
        if self.is_coinbase() {
            let random_bytes = random::get_nrandom_u64(4);
            for b in &random_bytes {
                serialized.extend_from_slice(&b.to_be_bytes());
            }
        }

        serialized.push(self.outputs.len() as u8);
        for output in &self.outputs {
            serialized.extend_from_slice(&output.value.to_be_bytes());
            serialized.extend_from_slice(&output.script_pubkey.get_der_encoding());
        }
        Sha256::hash(&serialized)
    }
}

impl std::fmt::Debug for Transaction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut res = String::from("Inputs: ");
        for input in &self.inputs {
            res.push_str(&format!(
                "txid: {}, vout: {}, script_sig: (AffinePoint: {}, PubKey: {})",
                input.txid, input.vout, input.script_sig.0, input.script_sig.1
            ));
        }
        for output in &self.outputs {
            res.push_str(&format!(
                "Outputs: value: {}, script_pubkey: {}",
                output.value, output.script_pubkey
            ));
        }
        write!(f, "Transaction: {}", res)
    }
}