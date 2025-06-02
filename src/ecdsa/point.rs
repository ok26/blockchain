use crate::math::{algorithms::mod_inverse, big_int::{BigInt, BigIntMod}};
use core::ops::Add;
use crate::ecdsa::secp256k1::*;

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct AffinePoint {
    pub x: BigInt<4>,
    pub y: BigInt<4>,
    infinity: bool,
}

#[derive(Clone, Copy)]
pub struct JacobianPoint {
    pub x: BigInt<4>,
    pub y: BigInt<4>,
    pub z: BigInt<4>,
}

impl AffinePoint {
    pub const fn new(x: BigInt<4>, y: BigInt<4>) -> Self {
        Self { x, y, infinity: false }
    }

    pub fn infinity() -> Self {
        let x = BigInt::from_num(0);
        let y = BigInt::from_num(0);
        Self { x, y, infinity: true, }
    }

    pub fn is_infinity(&self) -> bool {
        self.infinity
    }

    pub fn scalar_multiply(&self, scalar: BigInt<4>) -> JacobianPoint {
        if self.is_infinity() || scalar == BigInt::from_num(0) {
            return JacobianPoint::from_affine(&AffinePoint::infinity());
        }

        let mut result = JacobianPoint::from_affine(self);
        let bits = scalar.to_bits();
        let mut i = (scalar.log2() - 2) as i32;

        while i >= 0 {
            result = result.double();
            if bits[i as usize] {
                result = result + *self;
            }
            i = i - 1;
        }
        result
    }

    pub fn get_bytes(&self) -> Vec<u8> {
        if self.is_infinity() {
            return vec![0x00];
        }
        let mut bytes = vec![0x04]; // Uncompressed point prefix
        bytes.extend_from_slice(&self.x.to_bytes_be());
        bytes.extend_from_slice(&self.y.to_bytes_be());
        bytes
    }
}

impl JacobianPoint {
    pub fn new(x: BigInt<4>, y: BigInt<4>, z: BigInt<4>) -> Self {
        Self { x, y, z }
    }

    pub fn from_affine(affine: &AffinePoint) -> Self {
        if affine.is_infinity() {
            return Self::new(
                BigInt::from_num(1),
                BigInt::from_num(1),
                BigInt::from_num(0),
            );
        }
        Self::new(affine.x.clone(), affine.y.clone(), BigInt::from_num(1))
    }

    pub fn is_infinity(&self) -> bool {
        self.z == BigInt::from_num(0)
    }

    pub fn to_affine(&self) -> AffinePoint {
        if self.is_infinity() {
            return AffinePoint::new(BigInt::from_num(0), BigInt::from_num(0));
        }
        let px = BigIntMod::<12>::new_with_mu(self.x.resize(), P.resize(), BARRET_MU_P);
        let py = BigIntMod::<12>::new_with_mu(self.y.resize(), P.resize(), BARRET_MU_P);
        let pz = BigIntMod::<12>::new_with_mu(self.z.resize(), P.resize(), BARRET_MU_P);

        let z_inv: BigIntMod<12> = BigIntMod::new_with_mu(mod_inverse(pz.integer, P.resize()), P.resize(), BARRET_MU_P);
        let z_inv_2 = z_inv.square();
        let z_inv_3 = z_inv_2 * z_inv;
        let x = px * z_inv_2;
        let y = py * z_inv_3;
        AffinePoint::new(x.integer.resize(), y.integer.resize())
    }

    pub fn double(&self) -> Self {
        if self.is_infinity() || self.y == BigInt::from_num(0) {
            return Self::from_affine(&AffinePoint::infinity());
        }

        let px = BigIntMod::<12>::new_with_mu(self.x.resize(), P.resize(), BARRET_MU_P);
        let py = BigIntMod::<12>::new_with_mu(self.y.resize(), P.resize(), BARRET_MU_P);
        let pz = BigIntMod::<12>::new_with_mu(self.z.resize(), P.resize(), BARRET_MU_P);

        let y2 = py.square();
        let s = BigIntMod::<12>::from_num(4, P.resize()) * px * y2;
        let m = BigIntMod::<12>::from_num(3, P.resize()) * px.square();
        let x = m.square() - BigIntMod::<12>::from_num(2, P.resize()) * s;
        let y = m * (s - x) - BigIntMod::<12>::from_num(8, P.resize()) * y2.square();
        let z = BigIntMod::<12>::from_num(2, P.resize()) * py * pz;

        Self::new(x.integer.resize(), y.integer.resize(), z.integer.resize())
    }
}

impl Add<AffinePoint> for JacobianPoint {
    type Output = Self;

    fn add(self, other: AffinePoint) -> Self {
        if self.is_infinity() {
            return Self::from_affine(&other);
        }
        if other.is_infinity() {
            return self;
        }

        let x1 = BigIntMod::<12>::new_with_mu(self.x.resize(), P.resize(), BARRET_MU_P);
        let y1 = BigIntMod::<12>::new_with_mu(self.y.resize(), P.resize(), BARRET_MU_P);
        let z1 = BigIntMod::<12>::new_with_mu(self.z.resize(), P.resize(), BARRET_MU_P);
        let x2 = BigIntMod::<12>::new_with_mu(other.x.resize(), P.resize(), BARRET_MU_P);
        let y2 = BigIntMod::<12>::new_with_mu(other.y.resize(), P.resize(), BARRET_MU_P);

        let h = x2 * z1.square() - x1;
        let r = y2 * z1.square() * z1 - y1;
        if h.integer == BigInt::from_num(0) {
            if r.integer != BigInt::from_num(0) {
                return Self::from_affine(&AffinePoint::infinity());
            }
            return self.double();
        }
        let h2 = h.square();
        let h3 = h * h2;
        let x3 = r.square() - h3 - BigIntMod::<12>::from_num(2, P.resize()) * x1 * h2;
        let y3 = r * (x1 * h2 - x3) - y1 * h3;
        let z3 = h * z1;

        Self {
            x: x3.integer.resize(),
            y: y3.integer.resize(),
            z: z3.integer.resize(),
        }
    }
}

impl Add<JacobianPoint> for JacobianPoint {
    type Output = Self;

    fn add(self, other: JacobianPoint) -> Self {
        if self.is_infinity() {
            return other;
        }
        if other.is_infinity() {
            return self;
        }

        let x1 = BigIntMod::<12>::new_with_mu(self.x.resize(), P.resize(), BARRET_MU_P);
        let y1 = BigIntMod::<12>::new_with_mu(self.y.resize(), P.resize(), BARRET_MU_P);
        let z1 = BigIntMod::<12>::new_with_mu(self.z.resize(), P.resize(), BARRET_MU_P);
        let x2 = BigIntMod::<12>::new_with_mu(other.x.resize(), P.resize(), BARRET_MU_P);
        let y2 = BigIntMod::<12>::new_with_mu(other.y.resize(), P.resize(), BARRET_MU_P);
        let z2 = BigIntMod::<12>::new_with_mu(other.z.resize(), P.resize(), BARRET_MU_P);

        let z22 = z2.square();
        let z12 = z1.square();
        let u = x1 * z22;
        let h = x2 * z12 - u;
        let s = y1 * z22 * z2;
        let r = y2 * z12 * z1 - s;
        if h.integer == BigInt::from_num(0) {
            if r.integer != BigInt::from_num(0) {
                return Self::from_affine(&AffinePoint::infinity());
            }
            return self.double();
        }
        let h2 = h.square();
        let h3 = h * h2;
        let x3 = r.square() - h3 - BigIntMod::<12>::from_num(2, P.resize()) * u * h2;
        let y3 = r * (u * h2 - x3) - s * h3;
        let z3 = h * z1 * z2;

        Self {
            x: x3.integer.resize(),
            y: y3.integer.resize(),
            z: z3.integer.resize(),
        }
    }
}

impl std::fmt::Display for AffinePoint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_infinity() {
            write!(f, "AffinePoint(infinity)")
        } else {
            write!(f, "04{}{}", self.x.get_hex(), self.y.get_hex())
        }
    }
}

impl std::fmt::Display for JacobianPoint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_infinity() {
            write!(f, "JacobianPoint(infinity)")
        } else {
            write!(f, "JacobianPoint({}, {}, {})", self.x.get_hex(), self.y.get_hex(), self.z.get_hex())
        }
    }
}