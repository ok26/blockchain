use math::big_int::BigInt;

mod blockchain;
mod rsa;
mod math;

fn main() {
    let rsa_keys = rsa::generate_keys();
    let public_key = rsa_keys.0;
    let private_key = rsa_keys.1;
    public_key.save("public_key.txt");
    private_key.save("private_key.txt");

    let check_public_key = rsa::RSAPublicKey::load("public_key.txt");
    let check_private_key = rsa::RSAPrivateKey::load("private_key.txt");
    assert_eq!(public_key.n, check_public_key.n);
    assert_eq!(public_key.e, check_public_key.e);
    assert_eq!(private_key.p, check_private_key.p);
    assert_eq!(private_key.q, check_private_key.q);
    assert_eq!(private_key.dp, check_private_key.dp);
    assert_eq!(private_key.dq, check_private_key.dq);
    assert_eq!(private_key.qinv, check_private_key.qinv);
    assert_eq!(private_key.qpp, check_private_key.qpp);

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