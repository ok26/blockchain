use prime_gen::generate_primes;
use crate::{math::{algorithms, big_int::{BigInt, BigIntMod}}, sha256::Sha256, util};

mod prime_gen;

const KEY_SIZE: usize = 50;
const MILLER_ROUND: usize = 16;

#[derive(PartialEq, Debug)]
pub struct RSAPublicKey {
    pub n: BigInt<KEY_SIZE>,
    pub e: BigInt<KEY_SIZE>,
}

#[derive(PartialEq, Debug)]
pub struct RSAPrivateKey {
    pub n: BigInt<KEY_SIZE>,
    pub e: BigInt<KEY_SIZE>,
    pub d: BigInt<KEY_SIZE>,
    pub p: BigInt<KEY_SIZE>,
    pub q: BigInt<KEY_SIZE>,
    pub dp: BigInt<KEY_SIZE>,
    pub dq: BigInt<KEY_SIZE>,
    pub qinv: BigInt<KEY_SIZE>,
}

impl RSAPrivateKey {
    pub fn load(file: &str) -> Self {
        let base64_encoded = std::fs::read_to_string(file).expect("Unable to read file");
        let der_encoding = util::base64_decode(&base64_encoded);
        let mut bytes = der_encoding.as_slice();
        let fields = util::der_decode::<KEY_SIZE>(&mut bytes);
        assert_eq!(fields.len(), 8, "Invalid DER encoding for RSA private key");
        RSAPrivateKey {
            n: fields[0].clone(),
            e: fields[1].clone(),
            d: fields[2].clone(),
            p: fields[3].clone(),
            q: fields[4].clone(),
            dp: fields[5].clone(),
            dq: fields[6].clone(),
            qinv: fields[7].clone(),
        }
    }

    pub fn save(&self, file: &str) {
        let der_encoding = self.get_der_encoding();
        let base64_encoded = util::base64_encode(&der_encoding);
        std::fs::write(file, base64_encoded).expect("Unable to write file");
    }

    pub fn get_der_encoding(&self) -> Vec<u8> {
        let fields = vec![
            &self.n,
            &self.e,
            &self.d,
            &self.p,
            &self.q,
            &self.dp,
            &self.dq,
            &self.qinv,
        ];
        util::der_encode(&fields)
    }
}

impl std::fmt::Display for RSAPublicKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let der_encoding = self.get_der_encoding();
        let base64_encoded = util::base64_encode(&der_encoding);
        write!(f, "{}", base64_encoded)
    }
}

impl RSAPublicKey {
    pub fn load(file: &str) -> Self {
        let base64_encoded = std::fs::read_to_string(file).expect("Unable to read file");
        let der_encoding = util::base64_decode(&base64_encoded);
        let mut bytes = der_encoding.as_slice();
        let fields = util::der_decode::<KEY_SIZE>(&mut bytes);
        assert_eq!(fields.len(), 2, "Invalid DER encoding for RSA public key");
        RSAPublicKey {
            n: fields[0].clone(),
            e: fields[1].clone(),
        }
    }

    pub fn save(&self, file: &str) {
        let der_encoding = self.get_der_encoding();
        let base64_encoded = util::base64_encode(&der_encoding);
        std::fs::write(file, base64_encoded).expect("Unable to write file");
    }

    pub fn get_der_encoding(&self) -> Vec<u8> {
        let fields = vec![&self.n, &self.e];
        util::der_encode(&fields)
    }
}

impl std::fmt::Display for RSAPrivateKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let der_encoding = self.get_der_encoding();
        let base64_encoded = util::base64_encode(&der_encoding);
        write!(f, "{}", base64_encoded)
    }
}

pub fn generate_keys() -> (RSAPublicKey, RSAPrivateKey) {
    let primes = generate_primes(2);
    let p = primes[0].clone();
    let q = primes[1].clone();

    let n: BigInt<{2 * KEY_SIZE}> = p.resize() * q.resize();
    let phi: BigInt<{2 * KEY_SIZE}> = algorithms::lcm(p.resize() - BigInt::from_num(1), q.resize() - BigInt::from_num(1));

    let e = BigInt::from_num(65537);
    if e >= phi || algorithms::gcd(e, phi) != BigInt::from_num(1) {
        println!("e must be less than phi and coprime to phi, restarting RSA key generation");
        return generate_keys();
    }

    let d = algorithms::mod_inverse(e, phi);
    let dp = BigIntMod::new(d, p.resize() - BigInt::from_num(1)).slow_reduce();
    let dq = BigIntMod::new(d, q.resize() - BigInt::from_num(1)).slow_reduce();
    let qinv = algorithms::mod_inverse(q, p);

    return (
        RSAPublicKey { n: n.resize(), e: e.resize() },
        RSAPrivateKey { 
            n: n.resize(),
            e: e.resize(),
            d: d.resize(),
            p: p.resize(),
            q: q.resize(),
            dp: dp.integer.resize(),
            dq: dq.integer.resize(),
            qinv: qinv.resize(),
        },
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
        let qr = algorithms::get_qr(private_key.q, private_key.p);
        let mut qpp = qr.0;
        if qr.1 != BigInt::from_num(0) {
            qpp = qpp + BigInt::from_num(1);
        }
        qpp = qpp * private_key.p;
        m1.integer = m1.integer + qpp.resize();
    }
    let h = BigIntMod::<{2 * KEY_SIZE}>::new((m1.integer - m2.integer) * private_key.qinv.resize(), private_key.p.resize()).slow_reduce();
    (m2.integer + (h.integer * private_key.q.resize())).resize()
}

pub fn sign(data: &[u8], private_key: &RSAPrivateKey) -> BigInt<KEY_SIZE> {
    let data = Sha256::hash(data).to_bigint().resize();
    return decrypt(data, private_key);
}

pub fn verify(signature: BigInt<KEY_SIZE>, data: &[u8], public_key: &RSAPublicKey) -> bool {
    let data = Sha256::hash(data).to_bigint().resize();
    return encrypt(signature, public_key) == data;
}