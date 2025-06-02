use crate::{ecdsa::{point::AffinePoint, ECDSAPublicKey}, sha256::Sha256};

pub struct TxInput {
    pub txid: Sha256,
    pub vout: u32,
    pub script_sig: (AffinePoint, ECDSAPublicKey), // (signature, public key)
}

pub struct TxOutput {
    pub value: u64,
    pub script_pubkey: ECDSAPublicKey, // Public key or script hash
}

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

    pub fn add_input(&mut self, input: TxInput) {
        self.inputs.push(input);
    }

    pub fn add_output(&mut self, output: TxOutput) {
        self.outputs.push(output);
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut serialized = Vec::new();
        
        serialized.push(self.inputs.len() as u8);
        for input in &self.inputs {
            serialized.extend_from_slice(input.txid.bytes());
            serialized.extend_from_slice(&input.vout.to_be_bytes());
            serialized.extend_from_slice(&input.script_sig.0.get_bytes());
            serialized.extend_from_slice(&input.script_sig.1.get_der_encoding());
        }
        serialized.push(self.outputs.len() as u8);
        for output in &self.outputs {
            serialized.extend_from_slice(&output.value.to_be_bytes());
            serialized.extend_from_slice(&output.script_pubkey.get_der_encoding());
        }
        serialized
    }

    pub fn hash(&self) -> Sha256 {
        let serialized = self.serialize();
        Sha256::hash(&serialized)
    }
}