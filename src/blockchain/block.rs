use std::time::{SystemTime, UNIX_EPOCH};

fn now() -> u128 {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap();
    duration.as_secs() as u128 * 1000 + duration.subsec_millis() as u128
}

pub struct BlockHeader {
    pub index: u32,
    pub timestamp: u128,
    pub hash: Vec<u8>,
    pub previous_block_hash: Vec<u8>,
    pub nonce: u64,
    pub merkle_root_hash: Vec<u8>,
    pub difficulty: u128
}

pub struct Block {
    header: BlockHeader,
    merkle: Vec<Vec<u8>> 
}

impl Block {
    pub fn new_genesis() -> Block {
        Block {
            header: BlockHeader {
                index: 0,
                timestamp: now(),
                hash: vec![0; 32],
                previous_block_hash: vec![0; 32],
                nonce: 0,
                merkle_root_hash: vec![0; 32],
                difficulty: 0
            },
            merkle: vec![]
        }
    }
}