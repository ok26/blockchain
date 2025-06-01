use math::big_int::BigInt;
use sha256::Sha256;

mod blockchain;
mod rsa;
mod math;
mod sha256;
mod ecc;

fn main() {
    let (public_key, private_key) = ecc::generate_keypair();
    println!("Public Key: {}", public_key);
    println!("Private Key: {}", private_key.get_hex());
    let signature = ecc::sign(b"Hello, world!", private_key);
    println!("Signature: ({}, {})", signature.0.get_hex(), signature.1.get_hex());
    let is_valid = ecc::verify(signature, b"Hello, world!", public_key);
    println!("Is valid: {}", is_valid);
}