use math::big_int::BigInt;
use sha256::Sha256;

mod blockchain;
mod rsa;
mod math;
mod sha256;

fn main() {
    let test_data = b"Some test data for SHA-256 hashing and RSA signing.";
    let public_key = rsa::RSAPublicKey::load("public_key.txt");
    let private_key = rsa::RSAPrivateKey::load("private_key.txt");

    let sha256_hash = Sha256::hash(test_data);
    println!("SHA-256 Hash: {}", sha256_hash.digest());

    let big_int_data = BigInt::from_hex_string(&sha256_hash.digest());
    let signature = rsa::sign(big_int_data, &private_key);
    let is_valid = rsa::verify(signature, big_int_data, &public_key);
    println!("Signature valid: {}", is_valid);

    let mut manipulated_data = test_data.clone();
    manipulated_data[0] = manipulated_data[0].wrapping_add(1);
    let sha256_manipulated_hash = Sha256::hash(&manipulated_data);
    println!("Manipulated SHA-256 Hash: {}", sha256_manipulated_hash.digest());
    let manipulated_big_int_data = BigInt::from_hex_string(&sha256_manipulated_hash.digest());
    let manipulated_signature = rsa::sign(manipulated_big_int_data, &private_key);
    let is_manipulated_valid = rsa::verify(manipulated_signature, big_int_data, &public_key);
    println!("Manipulated signature valid: {}", is_manipulated_valid);
}