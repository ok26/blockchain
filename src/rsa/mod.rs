use prime_gen::generate_primes;
use crate::math::{algorithms::{self, gcd, get_qr, lcm, mod_inverse}, big_int::{BigInt, BigIntMod}};

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

impl RSAPrivateKey {
    pub fn load(file: &str) -> Self {
        let base64 = std::fs::read_to_string(file).expect("Failed to read file");
        let bytes = algorithms::base64_decode(&base64);
        let mut index = 0;
        let mut fields = Vec::with_capacity(6);

        for _ in 0..6 {
            let len = u16::from_be_bytes([
                bytes[index],
                bytes[index + 1],
            ]) as usize;
            index += 2;
            let field = BigInt::<KEY_SIZE>::from_bytes_be(&bytes[index..index + len]);
            fields.push(field);
            index += len;
        }

        RSAPrivateKey {
            p: fields[0].clone(),
            q: fields[1].clone(),
            dp: fields[2].clone(),
            dq: fields[3].clone(),
            qinv: fields[4].clone(),
            qpp: fields[5].clone(),
        }
    }

    pub fn save(&self, file: &str) {
        let fields = [
            &self.p,
            &self.q,
            &self.dp,
            &self.dq,
            &self.qinv,
            &self.qpp,
        ];

        let mut bytes: Vec<u8> = Vec::new();
        for field in fields.iter() {
            let field_bytes = field.to_bytes_be();
            let len = field_bytes.len() as u16;
            bytes.extend_from_slice(&len.to_be_bytes());
            bytes.extend_from_slice(&field_bytes);
        }

        let out = algorithms::base64_encode(&bytes);
        std::fs::write(file, &out).expect("Failed to write file");
    }
}

impl RSAPublicKey {
    pub fn load(file: &str) -> Self {
        let base64 = std::fs::read_to_string(file).expect("Failed to read file");
        let bytes = algorithms::base64_decode(&base64);
        let mut index = 0;
        let mut fields = Vec::with_capacity(2);

        for _ in 0..2 {
            let len = u16::from_be_bytes([
                bytes[index],
                bytes[index + 1],
            ]) as usize;
            index += 2;
            let field = BigInt::<KEY_SIZE>::from_bytes_be(&bytes[index..index + len]);
            fields.push(field);
            index += len;
        }

        RSAPublicKey {
            n: fields[0].clone(),
            e: fields[1].clone(),
        }
    }

    pub fn save(&self, file: &str) {
        let fields = [
            &self.n,
            &self.e,
        ];

        let mut bytes: Vec<u8> = Vec::new();
        for field in fields.iter() {
            let field_bytes = field.to_bytes_be();
            let len = field_bytes.len() as u16;
            bytes.extend_from_slice(&len.to_be_bytes());
            bytes.extend_from_slice(&field_bytes);
        }

        let out = algorithms::base64_encode(&bytes);
        std::fs::write(file, &out).expect("Failed to write file");
    }
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