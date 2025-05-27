use crate::BigInt;

pub fn abs<const T: usize>(a: BigInt<T>) -> BigInt<T> {
    if a.is_negative() {
        -a
    } else {
        a
    }
}

// Nonzero a and b
pub fn gcd<const T: usize>(a: BigInt<T>, b: BigInt<T>) -> BigInt<T> {
    let mut g = BigInt::<T>::from_num(1);
    let mut a = a;
    let mut b = b;
    if a < b {
        std::mem::swap(&mut a, &mut b);
    }
    while !a.is_odd() && !b.is_odd() {
        a = a >> 1;
        b = b >> 1;
        g = g << 1;
    }
    while a != BigInt::<T>::from_num(0) {
        while !a.is_odd() {
            a = a >> 1;
        }
        while !b.is_odd() {
            b = b >> 1;
        }
        let t = abs(a - b) >> 1;
        if a >= b {
            a = t;
        } else {
            b = t;
        }
    }
    return g * b;
}

pub fn lcm<const T: usize>(a: BigInt<T>, b: BigInt<T>) -> BigInt<T> {
    let gcd = gcd(a, b);
    (a * b) / gcd
}

// x = m, y = a
pub fn mod_inverse<const T: usize>(a: BigInt<T>, m: BigInt<T>) -> BigInt<T> {
    let mut u = m;
    let mut v = a;
    let mut a0 = BigInt::<T>::from_num(1);
    let mut b0 = BigInt::<T>::from_num(0);
    let mut c0 = BigInt::<T>::from_num(0);
    let mut d0 = BigInt::<T>::from_num(1);

    while u != BigInt::<T>::from_num(0) {
        while !u.is_odd() {
            u = u >> 1;
            if !a0.is_odd() && !b0.is_odd() {
                a0 = a0 >> 1;
                b0 = b0 >> 1;
            } 
            else {
                a0 = (a0 + a) >> 1;
                b0 = (b0 - m) >> 1;
            }
        }
        while !v.is_odd() {
            v = v >> 1;
            if !c0.is_odd() && !d0.is_odd() {
                c0 = c0 >> 1;
                d0 = d0 >> 1;
            } 
            else {
                c0 = (c0 + a) >> 1;
                d0 = (d0 - m) >> 1;
            }
        }
        if u >= v {
            u = u - v;
            a0 = a0 - c0;
            b0 = b0 - d0;
        } else {
            v = v - u;
            c0 = c0 - a0;
            d0 = d0 - b0;
        }
    }

    if v != BigInt::<T>::from_num(1) {
        panic!("No inverse exists");
    }

    if d0.is_negative() {
        d0 = d0 + m;
    }
    return d0;
}

pub fn get_qr<const T: usize>(a: BigInt<T>, b: BigInt<T>) -> (BigInt<T>, BigInt<T>) {
    if a.is_negative() || b.is_negative() {
        panic!("a and b must be non-negative integers");
    }
    let q = a / b;
    let r = a - (q * b);
    return (q, r);
}