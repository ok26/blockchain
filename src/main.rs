mod rsa;
mod math;
mod sha256;
mod ecdsa;
mod util;
mod blockchain;

fn main() {
    let public_key_ecdsa = ecdsa::ECDSAPublicKey::load("example_keys/public_key_ecdsa.txt");
    let private_key_ecdsa = ecdsa::ECDSAPrivateKey::load("example_keys/private_key_ecdsa.txt");

    println!("ECDSA Public Key: {}", public_key_ecdsa);
    println!("ECDSA Private Key: {}", private_key_ecdsa);

    let message = b"Hello, world!";
    println!("Message: {}", String::from_utf8_lossy(message));

    let signature_ecc = ecdsa::sign(message, &private_key_ecdsa);
    println!("ECC Signature: {}", signature_ecc);

    let is_valid_ecc = ecdsa::verify(signature_ecc, message, &public_key_ecdsa);
    println!("ECC Signature Valid: {}", is_valid_ecc);
}