use math::big_int::BigInt;

mod blockchain;
mod rsa;
mod math;

fn main() {
    let rsa_keys = rsa::generate_keys();
    let public_key = rsa_keys.0;
    let private_key = rsa_keys.1;
    println!("Public Key:\nn = {}\ne = {}", public_key.n, public_key.e);
    println!("Private Key:\np = {}\nq = {}\ndp = {}\ndq = {}\nqinv = {}\nqpp = {}", 
             private_key.p, private_key.q, private_key.dp, private_key.dq, private_key.qinv, private_key.qpp);
    
    let message = BigInt::<1>::from_hex_string("123456789ABCDEF0");
    println!("Message: {}", message);
    let encrypted_message = rsa::encrypt(message.resize(), &public_key);
    println!("Encrypted Message: {}", encrypted_message);
    let decrypted_message = rsa::decrypt(encrypted_message, &private_key);
    println!("Decrypted Message: {}", decrypted_message);

    let signature = rsa::sign(message.resize(), &private_key);
    println!("Signature: {}", signature);
    if rsa::verify(signature, message.resize(), &public_key) {
        println!("Signature is valid.");
    } else {
        println!("Signature is invalid.");
    }
}