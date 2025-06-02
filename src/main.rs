mod rsa;
mod math;
mod sha256;
mod ecdsa;

fn main() {
    let (public_key_ecdsa, private_key_ecdsa) = ecdsa::generate_keypair();
    println!("ECDSA Public Key: {}", public_key_ecdsa);
    println!("ECDSA Private Key: {}", private_key_ecdsa);

    let (public_key_rsa, private_key_rsa) = rsa::generate_keys();
    println!("RSA Public Key: {}", public_key_rsa);
    println!("RSA Private Key: {}", private_key_rsa);

    let message = b"Hello, world!";
    println!("Message: {}", String::from_utf8_lossy(message));

    let signature_ecc = ecdsa::sign(message, &private_key_ecdsa);
    println!("ECC Signature: {}", signature_ecc);

    let signature_rsa = rsa::sign(message, &private_key_rsa);
    println!("RSA Signature: {}", signature_rsa);

    let is_valid_ecc = ecdsa::verify(signature_ecc, message, &public_key_ecdsa);
    println!("ECC Signature Valid: {}", is_valid_ecc);
    let is_valid_rsa = rsa::verify(signature_rsa, message, &public_key_rsa);
    println!("RSA Signature Valid: {}", is_valid_rsa);

    public_key_ecdsa.save("example_keys/public_key_ecdsa.txt");
    private_key_ecdsa.save("example_keys/private_key_ecdsa.txt");
    public_key_rsa.save("example_keys/public_key_rsa.txt");
    private_key_rsa.save("example_keys/private_key_rsa.txt");

    let public_key_ecdsa_test = ecdsa::ECDSAPublicKey::load("example_keys/public_key_ecdsa.txt");
    let private_key_ecdsa_test = ecdsa::ECDSAPrivateKey::load("example_keys/private_key_ecdsa.txt");
    let public_key_rsa_test = rsa::RSAPublicKey::load("example_keys/public_key_rsa.txt");
    let private_key_rsa_test = rsa::RSAPrivateKey::load("example_keys/private_key_rsa.txt");

    assert_eq!(public_key_ecdsa, public_key_ecdsa_test);
    assert_eq!(private_key_ecdsa, private_key_ecdsa_test);
    assert_eq!(public_key_rsa, public_key_rsa_test);
    assert_eq!(private_key_rsa, private_key_rsa_test);
}