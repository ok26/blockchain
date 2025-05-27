use prime_gen::generate_primes;
use crate::math::{algorithms::{gcd, get_qr, lcm, mod_inverse}, big_int::{BigInt, BigIntMod}};

mod prime_gen;

const KEY_SIZE: usize = 50;
const MILLER_ROUND: usize = 16;

pub struct RSAPublicKey {
    pub n: BigInt<KEY_SIZE>,
    pub e: BigInt<KEY_SIZE>,
}
pub struct RSAPrivateKey {
    pub p: BigInt<KEY_SIZE>,
    pub q: BigInt<KEY_SIZE>,
    pub dp: BigInt<KEY_SIZE>,
    pub dq: BigInt<KEY_SIZE>,
    pub qinv: BigInt<KEY_SIZE>,
    pub qpp: BigInt<KEY_SIZE>,
}

pub fn generate_keys() -> (RSAPublicKey, RSAPrivateKey) {
    let primes = generate_primes(2);
    let p = primes[0].clone();
    let q = primes[1].clone();

    let n: BigInt<{2 * KEY_SIZE}> = p.resize() * q.resize();
    let phi: BigInt<{2 * KEY_SIZE}> = lcm(p.resize() - BigInt::from_num(1), q.resize() - BigInt::from_num(1));

    let e = BigInt::from_num(65537);
    if e >= phi || gcd(e, phi) != BigInt::from_num(1) {
        println!("e must be less than phi and coprime to phi, restarting RSA key generation");
        return generate_keys();
    }

    let d = mod_inverse(e, phi);
    let dp = BigIntMod::new(d, p.resize() - BigInt::from_num(1)).slow_reduce();
    let dq = BigIntMod::new(d, q.resize() - BigInt::from_num(1)).slow_reduce();
    let qinv = mod_inverse(q, p);
    
    let qr = get_qr(q, p);
    let mut qpp = qr.0;
    if qr.1 != BigInt::from_num(0) {
        qpp = qpp + BigInt::from_num(1);
    }
    qpp = qpp * p;

    return (
        RSAPublicKey { n: n.resize(), e: e.resize() },
        RSAPrivateKey { p, q, dp: dp.integer.resize(), dq: dq.integer.resize(), qinv, qpp },
    );
}

pub fn encrypt(message: BigInt<KEY_SIZE>, public_key: &RSAPublicKey) -> BigInt<KEY_SIZE> {
    let m = BigIntMod::<{2 * KEY_SIZE}>::new(message.resize(), public_key.n.resize());
    m.pow(public_key.e.resize()).integer.resize()
}

pub fn decrypt(ciphertext: BigInt<KEY_SIZE>, private_key: &RSAPrivateKey) -> BigInt<KEY_SIZE> {
    let mut m1 = BigIntMod::<{2 * KEY_SIZE}>::new(ciphertext.resize(), private_key.p.resize()).slow_reduce();
    let mut m2 = BigIntMod::<{2 * KEY_SIZE}>::new(ciphertext.resize(), private_key.q.resize()).slow_reduce();
    m1 = m1.pow(private_key.dp.resize());
    m2 = m2.pow(private_key.dq.resize());
    if m1.integer < m2.integer {
        m1.integer = m1.integer + private_key.qpp.resize();
    }
    let h = BigIntMod::<{2 * KEY_SIZE}>::new((m1.integer - m2.integer) * private_key.qinv.resize(), private_key.p.resize()).slow_reduce();
    (m2.integer + (h.integer * private_key.q.resize())).resize()
}

pub fn sign(data: BigInt<KEY_SIZE>, private_key: &RSAPrivateKey) -> BigInt<KEY_SIZE> {
    return decrypt(data, private_key);
}

pub fn verify(signature: BigInt<KEY_SIZE>, data: BigInt<KEY_SIZE>, public_key: &RSAPublicKey) -> bool {
    return encrypt(signature, public_key) == data;
}