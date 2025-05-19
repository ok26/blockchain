use crate::{math::{gcd, lcm, mod_inverse}, BigInt, BigIntMod};

const KEY_SIZE: usize = 50;

fn check_prime(num: BigInt<KEY_SIZE>, round: usize) -> bool {
    let mut d = num - BigInt::<KEY_SIZE>::from_num(1);
    let mut r = 0;
    while !d.is_odd() {
        d = d >> 1;
        r += 1;
    }
    let mu = BigIntMod::<KEY_SIZE>::calculate_mu(num.clone());
    for _ in 0..round {
        let a = BigIntMod::new_reduce(BigInt::<KEY_SIZE>::rand(1, KEY_SIZE / 2 - 2), num.clone(), mu.clone());
        let mut x = a.pow(d.clone());
        if !(x.integer == BigInt::<KEY_SIZE>::from_num(1) || x.integer == num - BigInt::<KEY_SIZE>::from_num(1)) {
            return false;
        }
        for _ in 0..(r - 1) {
            x = x * x;
            if x.integer != num - BigInt::<KEY_SIZE>::from_num(1) {
                return false;
            }
        }
    }

    true
}

pub fn generate_prime() -> BigInt<{2 * KEY_SIZE}> {
    let mut prime = BigInt::<KEY_SIZE>::rand(16, KEY_SIZE / 2 - 2);
    let mut i = 0;
    if !prime.is_odd() {
        
        prime = prime + BigInt::<KEY_SIZE>::from_num(1);
    }
    while !check_prime(prime, 25) {
        println!("{}", i);
        i += 1;
        prime = prime + BigInt::<KEY_SIZE>::from_num(2);
    }
    prime.resize()
}

pub fn generate_keys() {
    let p = generate_prime();
    println!("p: {}", p);
    let q = generate_prime();
    println!("q: {}", q);

    let n = p * q;
    println!("n: {}", n);
    let phi = lcm(p - BigInt::from_num(1), q - BigInt::from_num(1));
    println!("phi: {}", phi);

    let e = BigInt::from_num(65537);
    if e >= phi || gcd(e, phi) != BigInt::from_num(1) {
        println!("e must be less than phi and coprime to phi, restarting RSA key generation");
        generate_keys();
        return;
    }

    let d = mod_inverse(e, phi);
    println!("d: {}", d);
}