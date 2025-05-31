use crate::math::big_int::BigInt;
use crate::ecc::point::AffinePoint;

pub const P: BigInt<4> = BigInt::from_parts([
    0xFFFFFFFEFFFFFC2F, 0xFFFFFFFFFFFFFFFF, 0xFFFFFFFFFFFFFFFF, 0xFFFFFFFFFFFFFFFF
]);
pub const GX: BigInt<4> = BigInt::from_parts([
    0x59F2815B16F81798, 0x029BFCDB2DCE28D9, 0x55A06295CE870B07, 0x79BE667EF9DCBBAC
]);
pub const GY: BigInt<4> = BigInt::from_parts([
    0x9C47D08FFB10D4B8, 0xFD17B448A6855419, 0x5DA4FBFC0E1108A8, 0x483ADA7726A3C465
]);
pub const G: AffinePoint = AffinePoint::new(GX, GY);
pub const N: BigInt<4> = BigInt::from_parts([
    0xBFD25E8CD0364141, 0xBAAEDCE6AF48A03B, 0xFFFFFFFFFFFFFFFE, 0xFFFFFFFFFFFFFFFF
]);

pub const BARRET_MU_P: BigInt<12> = BigInt::from_parts([
    0x0, 0x0, 0x00000001000003d1, 0x0, 0x0, 0x0, 0x1, 0x0, 0x0, 0x0, 0x0, 0x0
]);

pub const BARRET_MU_N: BigInt<12> = BigInt::from_parts([
    0xe697f5e45bcd07c7, 0x9d671cd581c69bc5, 0x402da1732fc9bec0, 
    0x4551231950b75fc4, 0x1, 0x0, 0x1, 0x0, 0x0, 0x0, 0x0, 0x0
]);