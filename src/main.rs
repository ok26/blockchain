use blockchain::{transaction::Transaction, Blockchain, MINING_REWARD};
use user::User;

use crate::node::Node;

mod rsa;
mod math;
mod sha256;
mod ecdsa;
mod util;
mod blockchain;
mod node;
mod user;


// Simple simulation of a blockchain with users and nodes
// Should be handled separately by user and node interfaces in the future
fn main() {
    // Should read existing blockchain from file
    // For now, create a new one with a coinbase transaction
    let coinbase = Transaction::get_coinbase(
        ecdsa::generate_keypair().0, 
        MINING_REWARD
    );
    let blockchain = Blockchain::new(coinbase.clone());

    // Some example users and nodes
    let mut user1 = User::new("User 1", ecdsa::generate_keypair());
    let mut user2 = User::new("User 2", ecdsa::generate_keypair());
    let mut miner1 = Node::new("Miner 1", blockchain.clone(), ecdsa::generate_keypair());
    let mut miner2 = Node::new("Miner 2", blockchain.clone(), ecdsa::generate_keypair());

    // First block consists only of the coinbase transaction
    let mined_block = miner1.mine();
    println!("Mined Block: {:?}", mined_block);

    // Miner 2 accepts the block that miner 1 mined
    miner2.accept_block(mined_block).unwrap();

    // Show funds of users
    println!("User 1 Funds: {:?}", user1.get_funds());
    println!("User 2 Funds: {:?}", user2.get_funds());
    println!("Miner 1 Funds: {:?}", miner1.user.get_funds());
    println!("Miner 2 Funds: {:?}", miner2.user.get_funds());

    // Some transactions are broadcasted between users and nodes
    let tx1 = miner1.user.try_transaction(&user1.public_key, 15).unwrap();
    let tx2 = miner1.user.try_transaction(&user2.public_key, 20).unwrap();

    // Miner2 gathers transactions and mines a new block
    miner2.add_transaction(tx1.clone()).unwrap();
    miner2.add_transaction(tx2.clone()).unwrap();

    let mined_block = miner2.mine();
    println!("Mined Block: {:?}", mined_block);

    // Miner 1 accepts the block that miner 2 mined
    miner1.accept_block(mined_block).unwrap();

    // The users involved in the transactions query the nodes and check if their transactions are confirmed
    if !miner1.is_transaction_confirmed(&tx1) || !miner2.is_transaction_confirmed(&tx1) {
        println!("Transaction 1 is not confirmed yet");
    }
    // If the transaction is confirmed, the users can now use the funds
    else {
        println!("Transaction 1 confirmed");
        user1.update_funds(&tx1);
        miner1.user.update_funds(&tx1);
    }

    // Same for the second transaction
    if !miner2.is_transaction_confirmed(&tx1) || !miner1.is_transaction_confirmed(&tx1) {
        println!("Transaction 2 is not confirmed yet.");
    }
    else {
        println!("Transaction 2 confirmed");
        user2.update_funds(&tx2);
        miner1.user.update_funds(&tx2);
    }

    // Show funds of users
    println!("User 1 Funds: {}", user1.get_funds());
    println!("User 2 Funds: {}", user2.get_funds());
    println!("Miner 1 Funds: {}", miner1.user.get_funds());
    println!("Miner 2 Funds: {}", miner2.user.get_funds());

    // More transactions are created
    let tx1 = miner2.user.try_transaction(&user1.public_key, 5).unwrap();
    let tx2 = user1.try_transaction(&user2.public_key, 10).unwrap();
    let tx3 = user2.try_transaction(&miner1.user.public_key, 7).unwrap();

    // Miner 1 gathers transactions and mines a new block
    miner1.add_transaction(tx1.clone()).unwrap();
    miner1.add_transaction(tx2.clone()).unwrap();
    miner1.add_transaction(tx3.clone()).unwrap();

    let mined_block = miner1.mine();
    println!("Mined Block: {:?}", mined_block);

    // Miner 2 accepts the block that miner 1 mined
    miner2.accept_block(mined_block).unwrap();

    // The transactions are once again verified (For simplicity, not completely)
    if !miner1.is_transaction_confirmed(&tx1) || !miner2.is_transaction_confirmed(&tx2) || !miner2.is_transaction_confirmed(&tx3) {
        println!("Some transactions are not confirmed yet.");
    } else {
        println!("All transactions confirmed");
        miner2.user.update_funds(&tx1);
        user1.update_funds(&tx1);
        user1.update_funds(&tx2);
        user2.update_funds(&tx2);
        user2.update_funds(&tx3);
        miner1.user.update_funds(&tx3);
    }

    // Show funds of users
    println!("User 1 Funds: {}", user1.get_funds());
    println!("User 2 Funds: {}", user2.get_funds());
    println!("Miner 1 Funds: {}", miner1.user.get_funds());
    println!("Miner 2 Funds: {}", miner2.user.get_funds());
}