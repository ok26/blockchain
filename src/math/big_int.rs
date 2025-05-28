use std::ops::{
    Mul,
    Add,
    Sub,
    Div,
    Shl,
    Shr,
    Not,
    Neg,
};

use super::{algorithms, random::get_nrandom_u64};

#[derive(Debug, Copy, Clone)]
pub struct BigInt<const T: usize = 128> {
    bytes: [u64; T]
}

impl<const T: usize> BigInt<T> {
    pub fn new() -> BigInt<T> {
        Self { bytes: [0; T] }
    }

    pub fn from_num(num: u128) -> Self {
        let mut bytes = [0; T];
        bytes[0] = (num as u128 % (u64::MAX as u128 + 1)) as u64;
        bytes[1] = (num >> 64) as u64;
        BigInt { bytes }
    }

    pub fn from_hex_string(hex_string: &str) -> Self {
        if hex_string.len() != T * 16 {
            panic!("Hex string length must be {} characters", T * 16);
        }
        let mut bytes = [0; T];
        let mut index = 0;
        while index < hex_string.len() {
            let part = u64::from_str_radix(&hex_string[index..index + 16], 16).unwrap();
            bytes[T - index / 16 - 1] = part;
            index += 16;
        }
        BigInt { bytes }
    }

    pub fn rand(low: usize, high: usize) -> Self {
        if high < low || low >= T || high >= T {
            panic!("Invalid range for random number generation");
        }

        let rbytes =  get_nrandom_u64(high + 1);
        let mut bytes: [u64; T] = [0; T];

        let high_part = rbytes[0] as usize % (high - low + 1);
        for i in 0..(low + high_part) {
            bytes[i] = rbytes[i + 1];
        }

        BigInt { bytes }
    }

    fn set_part(&mut self, index: usize, value: u64) {
        if index < T {
            self.bytes[index] = value;
        }
    }

    fn get_part(&self, index: usize) -> u64 {
        if index < T { self.bytes[index] } 
        else { 0 }
    }

    pub fn is_negative(&self) -> bool {
        self.bytes[T - 1] & 0x8000000000000000 != 0
    }

    pub fn is_odd(&self) -> bool {
        self.bytes[0] & 1 != 0
    }

    pub fn log2(&self) -> u64 {
        let mut result = 0;
        for i in (0..T).rev() {
            if self.bytes[i] != 0 {
                result += (i as u64) * 64;
                let mut j = 63;
                while j > 0 && (self.bytes[i] >> j) == 0 {
                    j -= 1;
                }
                result += j + 1;
                break;
            }
        }
        result
    }

    fn mod_parts(&self, k: usize) -> BigInt<T> {
        let mut result = self.clone();
        for i in k..T {
            result.set_part(i, 0);
        }
        result
    }

    pub fn mod_u64(&self, other: u64) -> u128 {
        let mut result = 0;
        for i in (0..T).rev() {
            result = ((result << 64) + self.get_part(i) as u128) % other as u128;
        }
        result
    }

    fn single_part_mul(&self, other: u64) -> Self {
        let mut result = BigInt::<T>::new();
        let mut carry = 0;
        for i in 0..T {
            let prod = self.bytes[i] as u128 * other as u128 + carry as u128;
            result.set_part(i, prod as u64);
            carry = (prod >> 64) as u64;
        }
        result
    }

    pub fn to_bytes_be(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(T * 8);

        for &part in self.bytes.iter().rev() {
            bytes.extend_from_slice(&part.to_be_bytes());
        }

        while bytes.first() == Some(&0) && bytes.len() > 1 {
            bytes.remove(0);
        }
        bytes
    }

    pub fn from_bytes_be(bytes: &[u8]) -> Self {
        let mut parts = [0u64; T];
        let mut byte_index = bytes.len();

        for limb_i in 0..T {
            if byte_index == 0 {
                break;
            }

            let start = if byte_index >= 8 { byte_index - 8 } else { 0 };
            let len = byte_index - start;
            let mut part_bytes = [0u8; 8];

            part_bytes[8 - len..].copy_from_slice(&bytes[start..byte_index]);

            parts[limb_i] = u64::from_be_bytes(part_bytes);
            byte_index = start;
        }

        Self { bytes: parts }
    }

    pub fn get_base64(&self) -> String {
        algorithms::base64_encode(&self.to_bytes_be())
    }
}

impl<const T: usize> PartialEq for BigInt<T> {
    fn eq(&self, other: &Self) -> bool {
        self.bytes == other.bytes
    }
}
impl<const T: usize> Eq for BigInt<T> {}

impl<const T: usize> PartialOrd for BigInt<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        for i in (0..T).rev() {
            if self.bytes[i] > other.bytes[i] {
                return Some(std::cmp::Ordering::Greater);
            } else if self.bytes[i] < other.bytes[i] {
                return Some(std::cmp::Ordering::Less);
            }
        }
        Some(std::cmp::Ordering::Equal)
    }
}

impl<const T: usize> Add<BigInt<T>> for BigInt<T> {
    type Output = BigInt<T>;

    fn add(self, rhs: BigInt<T>) -> BigInt<T> {
        let mut result = BigInt::<T>::new();
        let mut carry = 0;
        for i in 0..T {
            let sum = self.bytes[i] as u128 + rhs.bytes[i] as u128 + carry as u128;
            result.set_part(i, sum as u64);
            carry = (sum - result.get_part(i) as u128) >> 64;
        }
        result
    }
}

impl<const T: usize> Sub<BigInt<T>> for BigInt<T> {
    type Output = BigInt<T>;

    fn sub(self, rhs: BigInt<T>) -> BigInt<T> {
        let mut result = BigInt::<T>::new();
        let mut borrow = 0;

        for i in 0..T {
            let (sub1, overflow1) = self.bytes[i].overflowing_sub(rhs.bytes[i]);
            let (sub2, overflow2) = sub1.overflowing_sub(borrow);
            result.bytes[i] = sub2;
            borrow = (overflow1 as u64) + (overflow2 as u64);
        }

        result
    }
}

impl<const T: usize> Mul<BigInt<T>> for BigInt<T> {
    type Output = BigInt<T>;

    fn mul(self, rhs: BigInt<T>) -> BigInt<T> {
        let a = self <= BigInt::<T>::from_num(u64::MAX as u128);
        let b = rhs <= BigInt::<T>::from_num(u64::MAX as u128);
        if a && b {
            return BigInt::<T>::from_num(self.get_part(0) as u128 * rhs.get_part(0) as u128);
        }
        else if a {
            return rhs.single_part_mul(self.get_part(0))
        }
        else if b {
            return self.single_part_mul(rhs.get_part(0))
        }
        let mut n1 = 1;
        let mut n2 = 1;
        for i in 0..T {
            if self.get_part(i) != 0 {
                n1 = i + 1;
            }
            if rhs.get_part(i) != 0 {
                n2 = i + 1;
            }
        }
        let n = if n1 > n2 { n1 } else { n2 };

        let m = (n + 1) / 2;
        let mut x0 = BigInt::<T>::new();
        let mut y0 = BigInt::<T>::new();
        let mut x1 = BigInt::<T>::new();
        let mut y1 = BigInt::<T>::new();
        for i in 0..m {
            x0.set_part(i, self.get_part(i));
            y0.set_part(i, rhs.get_part(i));
            x1.set_part(i, self.get_part(i + m));
            y1.set_part(i, rhs.get_part(i + m));
        }

        let z2 = x1 * y1;
        let z0 = x0 * y0;
        let z1 = (x1 + x0) * (y1 + y0) - z2 - z0;
        
        (z2 << (2 * m * 64) as u64) + (z1 << (m * 64) as u64) + z0
    }
}

// Naive implementation
impl<const T: usize> Div<BigInt<T>> for BigInt<T> {
    type Output = BigInt<T>;

    fn div(self, rhs: Self) -> BigInt<T> {
        let mut q = BigInt::<T>::new();
        let mut r = BigInt::<T>::new();

        for i in (0..64 * T).rev() {
            r = r << 1;
            r.set_part(0, r.get_part(0) | ((self.get_part(i / 64) >> (i % 64)) & 1));
            if r >= rhs {
                r = r - rhs;
                q.set_part(i / 64, q.get_part(i / 64) | (1 << (i % 64)));
            }
        }
        q
    }
}

impl<const T: usize> Not for BigInt<T> {
    type Output = BigInt<T>;

    fn not(self) -> Self::Output {
        let mut res = Self::new();
        for i in 0..T {
            res.set_part(i, !self.get_part(i));
        }
        res
    }
}

impl<const T: usize> Neg for BigInt<T> {
    type Output = BigInt<T>;

    fn neg(self) -> Self::Output {
        !self + BigInt::<T>::from_num(1)
    }
}

impl<const T: usize> Shr<u64> for BigInt<T> {
    type Output = BigInt<T>;

    fn shr(self, rhs: u64) -> Self::Output {
        if self.is_negative() {
            return -((-self) >> rhs);
        }
        let mut res = Self::new();
        let parts_shift = rhs as usize / 64;
        let bits_shift = rhs as usize % 64;
        for i in 0..(T - parts_shift as usize) {
            res.set_part(i, self.get_part(i + parts_shift as usize));
        }
        if bits_shift != 0 {
            for i in (0..(T - parts_shift as usize)).rev() {
                let next_part = self.get_part(i + 1);
                res.set_part(i, (self.get_part(i) >> bits_shift) | (next_part << (64 - bits_shift)));
            }
        }
        res
    }
}

impl<const T: usize> Shl<u64> for BigInt<T> {
    type Output = BigInt<T>;

    fn shl(self, rhs: u64) -> Self::Output {
        if self.is_negative() {
            return -((-self) << rhs);
        }
        let mut res = Self::new();
        let parts_shift = rhs as usize / 64;
        let bits_shift = rhs as usize % 64;
        if parts_shift > T {
            return res;
        }
        for i in 0..(T - parts_shift as usize) {
            res.set_part(i + parts_shift as usize, self.get_part(i));
        }
        if bits_shift != 0 {
            for i in (0..(T - parts_shift as usize)).rev() {
                let next_part = if i == 0 { 0 } else { self.get_part(i - 1) };
                res.set_part(i, (self.get_part(i) << bits_shift) | (next_part >> (64 - bits_shift)));
            }
        }
        res
    }
}

impl<const FROM: usize> BigInt<FROM> {
    pub fn resize<const TO: usize>(self) -> BigInt<TO> {
        let mut result = BigInt::<TO>::new();
        for i in 0..FROM {
            result.set_part(i, self.get_part(i));
        }
        result
    }
}

impl<const T: usize> std::fmt::Display for BigInt<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.get_base64())
     }
}


#[derive(Debug, Copy, Clone)]
pub struct BigIntMod<const T: usize> {
    pub integer: BigInt<T>,
    modulo: BigInt<T>,
    barret_mu: Option<BigInt<T>>,
}

impl<const T: usize> BigIntMod<T> {
    pub fn new(integer: BigInt<T>, modulo: BigInt<T>) -> Self {
        let barret_mu = None;
        Self { integer, modulo, barret_mu }
    }

    pub fn new_reduce(integer: BigInt<T>, modulo: BigInt<T>, mu: BigInt<T>) -> Self {
        let mut result = Self::new(integer, modulo);
        result.barret_mu = Some(mu);
        result.barret_reduce();
        result
    }

    pub fn from_num(num: u128, modulo: BigInt<T>) -> Self {
        let integer = BigInt::<T>::from_num(num);
        let barret_mu = None;
        Self { integer, modulo, barret_mu }
    }

    pub fn pow(&self, exponent: BigInt<T>) -> Self {
        let mut result = BigIntMod::new(BigInt::<T>::from_num(1), self.modulo);
        let mut base = self.clone();
        let mut exp = exponent;

        while exp > BigInt::from_num(0) {
            if exp.is_odd() {
                result = result * base;
            }
            base = base * base;
            exp = exp >> 1;
        }
        result
    }

    pub fn calculate_mu(modulo: BigInt<T>) -> BigInt<T> {
        let k = modulo.log2() / 64 + 1;
        let mu = (BigInt::<T>::from_num(1) << (2 * k * 64)) / modulo;
        mu
    }

    pub fn slow_reduce(&mut self) -> BigIntMod<T> {
        let q = self.integer / self.modulo;
        let r = self.integer - (q * self.modulo);
        BigIntMod::new(r, self.modulo.clone())
    }

    pub fn barret_reduce(&mut self) {
        let k = self.modulo.log2() / 64 + 1;
        if self.barret_mu.is_none() {
            self.barret_mu = Some(Self::calculate_mu(self.modulo));
        }

        let mu = self.barret_mu.unwrap();
        let q1 = self.integer >> (64 * (k - 1));
        let q2 = q1 * mu;
        let q3 = q2 >> (64 * (k + 1));

        let r1 = self.integer.mod_parts(1 + k as usize);
        let r2 = (q3 * self.modulo).mod_parts(1 + k as usize);
        let mut r = r1 - r2;
        if r.is_negative() {
            r = r + (BigInt::<T>::from_num(1) << (64 * (k + 1)));
        }
        while r >= self.modulo {
            r = r - self.modulo;
        }
        self.integer = r;
    }
}


impl<const T: usize> Add<BigIntMod<T>> for BigIntMod<T> {
    type Output = BigIntMod<T>;

    fn add(self, rhs: BigIntMod<T>) -> BigIntMod<T> {
        if self.modulo != rhs.modulo {
            panic!("Cannot add BigIntMod with different modulos");
        }

        let mut result = self.integer + rhs.integer;
        if result >= self.modulo {
            result = result - self.modulo;
        }
        BigIntMod::new(result, self.modulo)
    }
}

impl<const T: usize> Sub<BigIntMod<T>> for BigIntMod<T> {
    type Output = BigIntMod<T>;

    fn sub(self, rhs: BigIntMod<T>) -> BigIntMod<T> {
        if self.modulo != rhs.modulo {
            panic!("Cannot subtract BigIntMod with different modulos");
        }

        let mut result = self.integer - rhs.integer;
        if result.is_negative() {
            result = result + self.modulo;
        }
        BigIntMod::new(result, self.modulo)
    }
}

impl<const T: usize> Mul<BigIntMod<T>> for BigIntMod<T> {
    type Output = BigIntMod<T>;

    fn mul(self, rhs: BigIntMod<T>) -> BigIntMod<T> {
        if self.modulo != rhs.modulo {
            panic!("Cannot multiply BigIntMod with different modulos");
        }

        let mut result = BigIntMod::new(self.integer * rhs.integer, self.modulo);
        result.barret_mu = if let Some(mu) = self.barret_mu { Some(mu) }
        else if let Some(mu) = rhs.barret_mu { Some(mu) }
        else { None };
        result.barret_reduce();
        result
    }
}

impl<const T: usize> std::fmt::Display for BigIntMod<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.integer)
    }
}