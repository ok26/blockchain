use blockchain::{block::Block, transaction::Transaction, Blockchain, MINING_REWARD};
use user::User;

mod rsa;
mod math;
mod sha256;
mod ecdsa;
mod util;
mod blockchain;
mod node;
mod user;

fn main() {
    let mut user = User::new("Sender", ecdsa::generate_keypair());
    let coinbase = Transaction::get_coinbase(user.public_key.clone(), MINING_REWARD);
    let mut blockchain = Blockchain::new(coinbase.clone());
    user.update_funds(&coinbase);
    println!("{:?}", blockchain);

    let mut user2 = User::new("Receiver", ecdsa::generate_keypair());
    println!("User1 funds: {}", user.get_funds());
    println!("User2 funds: {}", user2.get_funds());

    let test_transaction = user.try_transaction(&user2.public_key, 26)
        .unwrap_or_else(|e| { println!("Transaction failed: {:?}", e); std::process::exit(0) });

    // This will the node handle in the future
    let mut miner = User::new("Miner", ecdsa::generate_keypair());
    let reward = Transaction::get_coinbase(miner.public_key.clone(), MINING_REWARD);
    let mut block = Block::new(
        blockchain.blocks.last().unwrap().hash.clone(), 
        vec![test_transaction.clone(), reward.clone()]
    );
    block.mine();
    let _ = blockchain.verify_block(&block)
        .unwrap_or_else(|e| { println!("Failed to add block: {:?}", e); std::process::exit(0); } );

    println!("Transaction passed");

    blockchain.add_block(block);
    println!("{:?}", blockchain);
    user.update_funds(&test_transaction);
    user2.update_funds(&test_transaction);
    miner.update_funds(&reward);

    println!("User1 funds: {}", user.get_funds());
    println!("User2 funds: {}", user2.get_funds());
    println!("Miner funds: {}", miner.get_funds());

    let invalid_transaction = user.try_transaction(&user2.public_key, 30)
        .unwrap_or_else(|e| { println!("Transaction failed: {:?}", e); std::process::exit(0); } );

    println!("Something went wrong");
}