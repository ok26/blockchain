use math::big_int::BigInt;
use sha256::Sha256;
use ecc::{point::JacobianPoint, secp256k1::*};


mod blockchain;
mod rsa;
mod math;
mod sha256;
mod ecc;

fn main() {
    let keys = ecc::generate_keypair();
    let public_key = keys.0;
    let private_key = keys.1;

    let message = b"Hello, world!";
    let signature = ecc::sign(message, private_key);
    let is_valid = ecc::verify(signature, message, public_key);
    println!("Public Key: {}", public_key);
    println!("Private Key: {}", private_key);
    println!("Signature: {}, {}", signature.0, signature.1);
    println!("Is signature valid? {}", is_valid);
}