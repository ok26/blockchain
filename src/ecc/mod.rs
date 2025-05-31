pub mod point;
pub mod secp256k1;

use point::AffinePoint;
use secp256k1::BARRET_MU_N;
use crate::{math::{algorithms::mod_inverse, big_int::{BigInt, BigIntMod}}, sha256::Sha256};

pub fn generate_keypair() -> (AffinePoint, BigInt<4>) {
    let mut private_key = BigInt::rand(4, 4);
    while private_key >= secp256k1::N {
        private_key = BigInt::rand(4, 4);
    }
    (secp256k1::G.scalar_multiply(private_key).to_affine(), private_key)
}

pub fn sign(message: &[u8], private_key: BigInt<4>) -> (BigInt<4>, BigInt<4>) {
    let z = BigIntMod::<12>::new_with_mu(Sha256::hash(message).to_bigint().resize(), secp256k1::N.resize(), BARRET_MU_N);

    loop {

        let mut k = BigInt::rand(4, 4);
        while k.resize() >= secp256k1::N {
            k = BigInt::rand(4, 4);
        }

        let p = secp256k1::G.scalar_multiply(k);
        let mut r = BigIntMod::<12>::new_with_mu(p.x.resize(), secp256k1::N.resize(), BARRET_MU_N);
        r.barret_reduce();
        if r.integer == BigInt::from_num(0) {
            continue;
        }

        let d = BigIntMod::<12>::new_with_mu(private_key.resize(), secp256k1::N.resize(), BARRET_MU_N);
        let k_inv = BigIntMod::new_with_mu(mod_inverse(k.resize(), secp256k1::N.resize()), secp256k1::N.resize(), BARRET_MU_N);
        let s = k_inv * (z + r * d);
        if s.integer == BigInt::from_num(0) {
            continue;
        }

        return (r.integer.resize(), s.integer.resize());
    }
}

pub fn verify(signature: (BigInt<4>, BigInt<4>), message: &[u8], public_key: AffinePoint) -> bool {
    let r = signature.0;
    let s = signature.1;
    if r == BigInt::from_num(0) || s == BigInt::from_num(0) || r >= secp256k1::N || s >= secp256k1::N {
        return false;
    }

    let z = BigIntMod::<12>::new_with_mu(Sha256::hash(message).to_bigint().resize(), secp256k1::N.resize(), BARRET_MU_N);
    let w = BigIntMod::<12>::new_with_mu(mod_inverse(s.resize(), secp256k1::N.resize()), secp256k1::N.resize(), BARRET_MU_N);

    let u1 = w * z;
    let u2 = w * BigIntMod::new_with_mu(r.resize(), secp256k1::N.resize(), BARRET_MU_N);

    let p1 = secp256k1::G.scalar_multiply(u1.integer.resize());
    let p2 = public_key.scalar_multiply(u2.integer.resize());
    let p = p1 + p2.to_affine();
    p.x == r.resize() && p.y != BigInt::from_num(0) && !p.is_infinity()
}