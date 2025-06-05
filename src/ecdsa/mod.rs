pub mod point;
pub mod secp256k1;

use point::AffinePoint;
use secp256k1::BARRET_MU_N;
use crate::{math::{big_int::{BigInt, BigIntMod}, algorithms}, sha256::Sha256, util};

#[derive(PartialEq, Debug, Clone)]
pub struct ECDSAPrivateKey {
    pub key: BigInt<4>,
}

#[derive(PartialEq, Debug, Clone)]
pub struct ECDSAPublicKey {
    pub key: AffinePoint,
}

impl ECDSAPrivateKey {
    pub fn load(file: &str) -> Self {
        let base64_encoded = std::fs::read_to_string(file).expect("Unable to read file");
        let der_encoding = util::base64_decode(&base64_encoded);
        let mut bytes = der_encoding.as_slice();
        let fields = util::der_decode::<4>(&mut bytes);
        assert_eq!(fields.len(), 1, "Invalid DER encoding for ECDSA private key");
        ECDSAPrivateKey { key: fields[0].clone() }
    }

    pub fn save(&self, file: &str) {
        let der_encoding = self.get_der_encoding();
        let base64_encoded = util::base64_encode(&der_encoding);
        std::fs::write(file, base64_encoded).expect("Unable to write file");
    }

    pub fn get_der_encoding(&self) -> Vec<u8> {
        util::der_encode(&[&self.key])
    }
}

impl ECDSAPublicKey {
    pub fn load(file: &str) -> Self {
        let base64_encoded = std::fs::read_to_string(file).expect("Unable to read file");
        let der_encoding = util::base64_decode(&base64_encoded);
        let mut bytes = der_encoding.as_slice();
        let fields = util::der_decode::<4>(&mut bytes);
        assert_eq!(fields.len(), 2, "Invalid DER encoding for ECDSA public key");
        ECDSAPublicKey { key: AffinePoint::new(fields[0].clone(), fields[1].clone()) }
    }

    pub fn save(&self, file: &str) {
        let der_encoding = self.get_der_encoding();
        let base64_encoded = util::base64_encode(&der_encoding);
        std::fs::write(file, base64_encoded).expect("Unable to write file");
    }

    pub fn get_der_encoding(&self) -> Vec<u8> {
        util::der_encode(&[&self.key.x, &self.key.y])
    }
}

impl std::fmt::Display for ECDSAPublicKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.key)
    }
}

impl std::fmt::Display for ECDSAPrivateKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.key.get_hex())
    }
}

pub fn generate_keypair() -> (ECDSAPublicKey, ECDSAPrivateKey) {
    let mut private_key = BigInt::rand(4, 4);
    while private_key >= secp256k1::N {
        private_key = BigInt::rand(4, 4);
    }
    (
        ECDSAPublicKey { key: secp256k1::G.scalar_multiply(private_key).to_affine() }, 
        ECDSAPrivateKey { key: private_key }
    )
}

pub fn sign(message: &[u8], private_key: &ECDSAPrivateKey) -> AffinePoint {
    let z: BigInt<4> = Sha256::hash(message).to_bigint().resize();
    let z = BigIntMod::<12>::new_with_mu(z.resize(), secp256k1::N.resize(), BARRET_MU_N);

    loop {
        let mut k = BigInt::rand(4, 4);
        while k >= secp256k1::N {
            k = BigInt::rand(4, 4);
        }
        let p = secp256k1::G.scalar_multiply(k).to_affine();
        let x1 = p.x;
        let r = BigIntMod::<12>::new_reduce(x1.resize(), secp256k1::N.resize(), BARRET_MU_N);
        if r.integer == BigInt::from_num(0) {
            continue;
        }

        let k_inv = algorithms::mod_inverse(k.resize::<12>(), secp256k1::N.resize::<12>());
        let k_inv = BigIntMod::<12>::new_with_mu(k_inv, secp256k1::N.resize(), BARRET_MU_N);
        let da = BigIntMod::<12>::new_with_mu(private_key.key.resize(), secp256k1::N.resize(), BARRET_MU_N);
        let s = k_inv * (z + r * da);
        if s.integer == BigInt::from_num(0) {
            continue;
        }
        return AffinePoint::new(r.integer.resize(), s.integer.resize());
    }
}

pub fn verify(signature: AffinePoint, message: &[u8], public_key: &ECDSAPublicKey) -> bool {
    let r = signature.x;
    let s = signature.y;
    if r == BigInt::from_num(0) || s == BigInt::from_num(0) || r >= secp256k1::N || s >= secp256k1::N {
        return false;
    }

    let z: BigInt<4> = Sha256::hash(message).to_bigint().resize();
    let z = BigIntMod::<12>::new_with_mu(z.resize(), secp256k1::N.resize(), BARRET_MU_N);
    
    let s_inv = algorithms::mod_inverse(s.resize::<12>(), secp256k1::N.resize::<12>());
    let s_inv = BigIntMod::<12>::new_with_mu(s_inv, secp256k1::N.resize(), BARRET_MU_N);
    let u1 = z * s_inv;
    let u2 = BigIntMod::new_with_mu(r.resize(), secp256k1::N.resize(), BARRET_MU_N) * s_inv;

    let p1 = secp256k1::G.scalar_multiply(u1.integer.resize());
    let p2 = public_key.key.scalar_multiply(u2.integer.resize());
    let p = (p1 + p2).to_affine();
    let x1 = BigIntMod::<12>::new_reduce(p.x.resize(), secp256k1::N.resize(), BARRET_MU_N);
    x1.integer == r.resize()
}