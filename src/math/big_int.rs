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

use super::random::get_nrandom_u64;
use crate::util;

#[derive(Debug, Copy, Clone)]
pub struct BigInt<const T: usize = 128> {
    bytes: [u64; T]
}

impl<const T: usize> BigInt<T> {
    pub fn new() -> BigInt<T> {
        Self { bytes: [0; T] }
    }

    pub const fn from_num(num: u128) -> Self {
        let mut bytes = [0; T];
        bytes[0] = (num as u128 % (u64::MAX as u128 + 1)) as u64;
        bytes[1] = (num >> 64) as u64;
        BigInt { bytes }
    }

    pub const fn from_parts(parts: [u64; T]) -> Self {
        BigInt { bytes: parts }
    }

    pub fn from_hex_string(hex_string: &str) -> Self {
        let mut hex_string = hex_string.to_string();
        if hex_string.len() < T * 16 {
            let padding = "0".repeat(T * 16 - hex_string.len());
            hex_string = padding + &hex_string;
        } else if hex_string.len() > T * 16 {
            hex_string = hex_string[hex_string.len() - T * 16..].to_string();
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
        if high < low || low > T || high > T {
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

    pub fn set_part(&mut self, index: usize, value: u64) {
        if index < T {
            self.bytes[index] = value;
        }
    }

    pub fn get_part(&self, index: usize) -> u64 {
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

    pub fn to_bits(&self) -> Vec<bool> {
        let mut bits = Vec::with_capacity(T * 64);
        for &part in self.bytes.iter() {
            for i in 0..64 {
                bits.push((part >> i) & 1 != 0);
            }
        }
        bits
    }

    pub fn get_base64(&self) -> String {
        util::base64_encode(&self.to_bytes_be())
    }

    pub fn get_hex(&self) -> String {
        self.bytes.iter()
            .rev()
            .map(|&part| format!("{:016x}", part))
            .collect::<String>()
            .trim_start_matches('0')
            .to_string()
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
        let parts_shift = (rhs / 64) as usize;
        let bits_shift = (rhs % 64) as u32;

        if parts_shift >= T {
            return res;
        }

        // Shift parts
        for i in 0..(T - parts_shift) {
            res.bytes[i] = self.bytes[i + parts_shift];
        }

        // Shift bits within parts
        if bits_shift != 0 {
            let mut carry = 0u64;
            for i in (0..(T - parts_shift)).rev() {
                let val = res.bytes[i];
                res.bytes[i] = (val >> bits_shift) | carry;
                carry = if bits_shift < 64 { val << (64 - bits_shift) } else { 0 };
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
        let parts_shift = (rhs / 64) as usize;
        let bits_shift = (rhs % 64) as u32;

        if parts_shift >= T {
            return res;
        }

        // Shift parts
        for i in (0..T - parts_shift).rev() {
            res.bytes[i + parts_shift] = self.bytes[i];
        }

        // Shift bits within parts
        if bits_shift != 0 {
            let mut carry = 0u64;
            for i in parts_shift..T {
                let val = res.bytes[i];
                res.bytes[i] = (val << bits_shift) | carry;
                carry = if bits_shift < 64 { val >> (64 - bits_shift) } else { 0 };
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
    pub const fn new(integer: BigInt<T>, modulo: BigInt<T>) -> Self {
        let barret_mu = None;
        Self { integer, modulo, barret_mu }
    }

    pub fn new_with_mu(integer: BigInt<T>, modulo: BigInt<T>, mu: BigInt<T>) -> Self {
        let mut result = Self::new(integer, modulo);
        result.barret_mu = Some(mu);
        result
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

    pub fn square(&self) -> Self {
        *self * *self
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

        let r1 = self.integer.mod_parts(k as usize + 1);
        let r2 = (q3 * self.modulo).mod_parts(1 + k as usize);
        let mut r = r1 - r2;
        if r.is_negative() {
            r = r + (BigInt::<T>::from_num(1) << (64 * (k + 1)));
        }
        let mut m = 2;
        while r >= self.modulo && m != 0 {
            r = r - self.modulo;
            m = m - 1;
        }
        if r >= self.modulo {
            panic!("Barret reduction failed: result is greater than or equal to modulo");
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
        if let Some(mu) = self.barret_mu {
            BigIntMod::new_with_mu(result, self.modulo, mu)
        }
        else if let Some(mu) = rhs.barret_mu {
            BigIntMod::new_with_mu(result, self.modulo, mu)
        }
        else {
            BigIntMod::new(result, self.modulo)
        }
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
        if let Some(mu) = self.barret_mu {
            BigIntMod::new_with_mu(result, self.modulo, mu)
        }
        else if let Some(mu) = rhs.barret_mu {
            BigIntMod::new_with_mu(result, self.modulo, mu)
        }
        else {
            BigIntMod::new(result, self.modulo)
        }
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

impl<const FROM: usize> BigIntMod<FROM> {
    pub fn resize<const TO: usize>(self) -> BigIntMod<TO> {
        BigIntMod::<TO>::new(self.integer.resize(), self.modulo.resize())
    }
}

impl<const T: usize> std::fmt::Display for BigIntMod<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.integer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bigint_addition() {
        let a = BigInt::<10>::from_hex_string("aaabbbb12398017506123123cb12b3bbcbbdeb1beeb1bebbcB123");
        let b = BigInt::<10>::from_hex_string("9836123bbcbefff123cb12b3b23123bb123c000eff12b1bebbcB2");
        let c = a + b;
        assert_eq!(c, BigInt::<10>::from_hex_string("142e1cdece057016629dd43d77d43d776ddf9eb2aedc4707a86dd5"));
    }

    #[test]
    fn test_bigint_addition_negative() {
        let a = BigInt::<5>::from_hex_string("80cdef1234567890fedcba98765432100123456789abcdef0123456789abcdef1234567890abcdef");
        let b = -a;
        let c = a + b;
        assert_eq!(c, BigInt::from_num(0));
    }

    #[test]
    fn test_bigint_subtraction() {
        let a = BigInt::<10>::from_num(30);
        let b = BigInt::<10>::from_num(20);
        let c = a - b;
        assert_eq!(c.get_part(0), 10);
    }

    #[test]
    fn test_bigint_subtraction_negative() {
        let a = BigInt::<5>::from_hex_string("aaabbbb12398017506123123cb12b3bbcbbdeb1beeb1bebbcB123");
        let b = BigInt::<5>::from_hex_string("80cdef1234567890fedcba98765432100123456789abcdef0123456789abcdef1234567890abcdef");
        let c = a - b;
        assert_eq!(c, BigInt::from_hex_string("7f3210edcba9876f0123456789b678abb9eef4188da4933411196bc3b210edef9f8a94a35b10e334"));
    }

    #[test]
    fn test_bigint_multiplication() {
        let a = BigInt::<10>::from_hex_string("aaabbbb12398017506123123cb12b3bbcbbdeb1beeb1bebbcB123");
        let b = BigInt::<10>::from_hex_string("9836123bbcbefff123cb12b3b23123bb123c000eff12b1bebbcB2");
        let c = a * b;
        assert_eq!(c, BigInt::from_hex_string("657a03d2ab1bed2ee586b2d22a0a7449253c2f5cdb3324ef029d0bbc9f093e51b68ae5f6050748b0e44ec5f7742b06fb4ec769de56"));
    }

    #[test]
    fn test_bigint_from_num_and_get_part() {
        let n = 0x123456789abcdef0123456789abcdef0u128;
        let a = BigInt::<2>::from_num(n);
        assert_eq!(a.get_part(0), 0x123456789abcdef0u64);
        assert_eq!(a.get_part(1), 0x123456789abcdef0u64);
    }

    #[test]
    fn test_bigint_from_parts_and_to_bytes_be() {
        let parts = [0x1122334455667788, 0x99aabbccddeeff00];
        let a = BigInt::<2>::from_parts(parts);
        let bytes = a.to_bytes_be();
        assert_eq!(
            bytes,
            [
                0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, 0x00,
                0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88
            ]
        );
    }

    #[test]
    fn test_bigint_from_bytes_be() {
        let bytes = [
            0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08,
            0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f, 0x10
        ];
        let a = BigInt::<2>::from_bytes_be(&bytes);
        assert_eq!(a.get_part(1), 0x0102030405060708);
        assert_eq!(a.get_part(0), 0x090a0b0c0d0e0f10);
    }

    #[test]
    fn test_bigint_is_negative_and_neg() {
        let a = BigInt::<2>::from_parts([0x187123, 0x8000000000000000]);
        assert!(a.is_negative());
        let b = -a;
        assert!(!b.is_negative());
    }

    #[test]
    fn test_bigint_is_odd() {
        let a = BigInt::<2>::from_parts([3, 0]);
        assert!(a.is_odd());
        let b = BigInt::<2>::from_parts([2, 0]);
        assert!(!b.is_odd());
    }

    #[test]
    fn test_bigint_log2() {
        let a = BigInt::<2>::from_parts([0, 0x8000000000000000]);
        assert_eq!(a.log2(), 128);
        let b = BigInt::<2>::from_parts([0x8000000000000000, 0]);
        assert_eq!(b.log2(), 64);
        let c = BigInt::<2>::from_parts([0, 0]);
        assert_eq!(c.log2(), 0);
    }

    #[test]
    fn test_bigint_mod_u64() {
        let a = BigInt::<2>::from_parts([0x123456789abcdef0, 0x0fedcba987654321]);
        let m = 123456789u64;
        let r = a.mod_u64(m);
        let expected = ((0x0fedcba987654321u128 << 64) + 0x123456789abcdef0u128) % m as u128;
        assert_eq!(r, expected);
    }

    #[test]
    fn test_bigint_shl_and_shr() {
        let a = BigInt::<2>::from_parts([1, 0]);
        let b = a << 65;
        println!("{}, {}", a.get_hex(), b.get_hex());
        assert_eq!(b.get_part(0), 0);
        assert_eq!(b.get_part(1), 2);

        let c = b >> 65;
        assert_eq!(c, a);
    }

    #[test]
    fn test_bigint_not() {
        let a = BigInt::<2>::from_parts([0x0, 0xffffffffffffffff]);
        let b = !a;
        assert_eq!(b.get_part(0), 0xffffffffffffffff);
        assert_eq!(b.get_part(1), 0x0);
    }

    #[test]
    fn test_bigint_to_bits() {
        let a = BigInt::<1>::from_parts([0b1011]);
        let bits = a.to_bits();
        assert_eq!(bits[0], true);
        assert_eq!(bits[1], true);
        assert_eq!(bits[2], false);
        assert_eq!(bits[3], true);
    }

    #[test]
    fn test_bigint_get_hex_and_base64() {
        let a = BigInt::<2>::from_parts([0x123456789abcdef0, 0x0fedcba987654321]);
        let hex = a.get_hex();
        assert!(hex.contains("fedcba987654321123456789abcdef0"));
        let base64 = a.get_base64();
        assert!(!base64.is_empty());
    }

    #[test]
    fn test_bigint_resize() {
        let a = BigInt::<2>::from_parts([1, 2]);
        let b: BigInt<4> = a.resize();
        assert_eq!(b.get_part(0), 1);
        assert_eq!(b.get_part(1), 2);
        assert_eq!(b.get_part(2), 0);
        assert_eq!(b.get_part(3), 0);
    }

    // BigIntMod Tests
}