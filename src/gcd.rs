use crate::u256::U256;
use crunchy::unroll;

fn euclid_step(
    a: U256,
    b: U256,
    data: (U256, U256, U256, U256),
) -> (U256, U256, (U256, U256, U256, U256)) {
    let (q, rem) = a.divrem(&b).unwrap();
    let hold1 = &data.0 + &data.1 * &q;
    let hold2 = &data.2 + &data.3 * q;
    (b, rem, (data.1, hold1, data.3, hold2))
}

pub fn gcd_euclid(a: &U256, b: &U256) -> (U256, U256, U256, bool) {
    let mut a_prime;
    let mut b_prime;

    if b > a {
        //Note : Alg assumes a >= b, and gcd(a,b) = gcd(b,a)
        a_prime = b.clone(); //Gets correct ordering of mutable data
        b_prime = a.clone();
    } else {
        a_prime = a.clone(); //Gets correct ordering of mutable data
        b_prime = b.clone();
    }

    let mut consquences = (U256::ONE, U256::ZERO, U256::ZERO, U256::ONE);
    let mut even = true;

    while b_prime != U256::ZERO {
        let (hold1, hold2, hold3) = euclid_step(a_prime, b_prime, consquences);
        a_prime = hold1;
        b_prime = hold2;
        consquences = hold3;
        even = !even;
    }
    (a_prime, consquences.0, consquences.2, even)
}

/// Division optimized for small values
/// Requires a > b > 0. Returns a / b.
#[inline(always)]
#[allow(clippy::cognitive_complexity)]
fn div1(a: u64, b: u64) -> u64 {
    debug_assert!(a > b);
    let mut r = a;
    unroll! {
        for i in 1..20 {
            r -= b;
            if r < b {
                return i as u64
            }
        }
    }
    a / b
}

/// Division optimized for small values
/// Requires a > b > 0. Returns a / b.
#[inline(always)]
#[allow(clippy::cognitive_complexity)]
fn div_update(r0: &mut u64, r1: u64, u0: &mut u64, u1: u64, v0: &mut u64, v1: u64) -> u64 {
    unroll! {
        for q in 1..10 {
            *r0 -= r1;
            if *r0 < r1 {
                *u0 += (q as u64) * u1;
                *v0 += (q as u64) * v1;
                return (q as u64);
            }
        }
    }
    let mut q = *r0 / r1;
    *r0 -= q * r1;
    q += 9;
    *u0 += q * u1;
    *v0 += q * v1;
    q
}

/// Compute the Lehmer update matrix for small values.
/// This is essentialy Euclids extended GCD algorithm for 64 bits.
/// OPT: Would this be faster using extended binary gcd?
#[inline(never)]
fn lehmer_small(mut r0: u64, mut r1: u64) -> (u64, u64, u64, u64, bool) {
    if r1 == 0u64 {
        return (1, 0, 0, 1, true);
    }
    let mut q00 = 1u64;
    let mut q01 = 0u64;
    let mut q10 = 0u64;
    let mut q11 = 1u64;
    if r0 >= r1 {
        div_update(&mut r0, r1, &mut q00, q10, &mut q01, q11);
    }
    loop {
        // Loop is unrolled once to avoid swapping variables and tracking parity.
        if r0 == 0u64 {
            return (q10, q11, q00, q01, false);
        }
        div_update(&mut r1, r0, &mut q10, q00, &mut q11, q01);
        if r1 == 0u64 {
            return (q00, q01, q10, q11, true);
        }
        div_update(&mut r0, r1, &mut q00, q10, &mut q01, q11);
    }
}

#[inline(always)]
#[allow(clippy::cognitive_complexity)]
fn lehmer_unroll(a2: u64, a3: &mut u64, k2: u64, k3: &mut u64) {
    unroll! {
        for _i in 0..15 {
            if *a3 < a2 {
                return;
            }
            *a3 -= a2;
            *k3 += k2;
        }
    }
    if *a3 >= a2 {
        let q = *a3 / a2;
        *a3 -= q * a2;
        *k3 += q * k2;
    }
}

/// Compute the Lehmer update matrix for the most significant 64-bits of r0 and r1.
#[rustfmt::skip]
#[allow(clippy::cognitive_complexity)]
fn lehmer_loop(a0: u64, mut a1: u64) -> (u64, u64, u64, u64, bool) {
    const LIMIT: u64 = 1u64 << 32;

    // The cofactors u and v never exceed 32 bit. We can pack them in a single
    // 64 bit variable.
    let mut k0 = 1u64 << 32; // u0 = 1, v0 = 0
    let mut k1 = 1u64; // u1 = 0, v1 = 1
    let mut even = true;
    if a1 < LIMIT {
        return (1, 0, 0, 1, true)
    }

    // Compute a2
    let q = div1(a0, a1);
    let mut a2 = a0 - q * a1;
    let mut k2 = k0 + q * k1;
    if a2 < LIMIT {
        let u2 = k2 >> 32;
        let v2 = k2 % LIMIT;

        // Test i + 1 (odd)
        if a2 >= v2 && a1 - a2 >= u2 {
            return (0, 1, u2, v2, false);
        } else {
            return (1, 0, 0, 1, true);
        }
    }

    // Compute a3
    let q = div1(a1, a2);
    let mut a3 = a1 - q * a2;
    let mut k3 = k1 + q * k2;

    // Loop until a3 < LIMIT, maintaing the last three values
    // of a and the last four values of k.
    while a3 >= LIMIT {
        a1 = a2;
        a2 = a3;
        a3 = a1 - a2;
        k0 = k1;
        k1 = k2;
        k2 = k3;
        k3 += k1;
        lehmer_unroll(a2, &mut a3, k2, &mut k3);
        if a3 < LIMIT {
            even = false;
            break;
        }
        a1 = a2;
        a2 = a3;
        a3 = a1 - a2;
        k0 = k1;
        k1 = k2;
        k2 = k3;
        k3 += k1;
        lehmer_unroll(a2, &mut a3, k2, &mut k3);
    }
    // Unpack k into cofactors u and v
    let u0 = k0 >> 32;
    let u1 = k1 >> 32;
    let u2 = k2 >> 32;
    let u3 = k3 >> 32;
    let v0 = k0 % LIMIT;
    let v1 = k1 % LIMIT;
    let v2 = k2 % LIMIT;
    let v3 = k3 % LIMIT;
    debug_assert!(a2 >= LIMIT);
    debug_assert!(a3 < LIMIT);

    // Use Jebelean's exact condition to determine which outputs are correct.
    // Statistically, i + 2 should be correct about two-thirds of the time.
    if even {
        // Test i + 1 (odd)
        debug_assert!(a2 >= v2);
        if a1 - a2 >= u2 + u1 {
            // Test i + 2 (even)
            if a3 >= u3 && a2 - a3 >= v3 + v2 {
                // Correct value is i + 2
                (u2, v2, u3, v3, true)
            } else {
                // Correct value is i + 1
                (u1, v1, u2, v2, false)
            }
        } else {
            // Correct value is i
            (u0, v0, u1, v1, true)
        }
    } else {
        // Test i + 1 (even)
        debug_assert!(a2 >= u2);
        if a1 - a2 >= v2 + v1 {
            // Test i + 2 (odd)
            if a3 >= v3 && a2 - a3 >= u3 + u2 {
                // Correct value is i + 2
                (u2, v2, u3, v3, false)
            } else {
                // Correct value is i + 1
                (u1, v1, u2, v2, true)
            }
        } else {
            // Correct value is i
            (u0, v0, u1, v1, false)
        }
    }
}

/// Compute the Lehmer update matrix using double words
/// See https://github.com/ryepdx/gmp/blob/090b098806bc1a8f3af777b862369f58be465dd9/mpn/generic/hgcd2.c#L226
fn lehmer_double(mut r0: U256, mut r1: U256) -> (u64, u64, u64, u64, bool) {
    debug_assert!(r0 >= r1);
    if r0.bits() < 64 {
        return lehmer_small(r0.c0, r1.c0);
    }
    let s = r0.leading_zeros();
    let r0s = r0.clone() << s;
    let r1s = r1.clone() << s;
    let q = lehmer_loop(r0s.c3, r1s.c3);
    //println!("({:?}, {:?}, {:?}, {:?}) (first word)", q.0, q.1, q.2, q.3);
    if q.2 == 0u64 {
        return q;
    }
    // We can return q here and have a perfectly valid single-word Lehmer GCD.
    // return q;

    // Recompute r0 and r1 and take the high bits.
    // OPT: This does not need full precision.
    // OPT: Can we reuse the shifted variables here?
    // TODO: Should we use lehmer_small here when r0 is one word?
    lehmer_update(&mut r0, &mut r1, q);
    let s = r0.leading_zeros();
    let r0s = r0.clone() << s;
    let r1s = r1.clone() << s;
    let qn = lehmer_loop(r0s.c3, r1s.c3);
    //println!("({:?}, {:?}, {:?}, {:?}) (second word)", qn.0, qn.1, qn.2, qn.3);
    debug_assert!(qn.0 < (1u64 << 32));
    debug_assert!(qn.1 < (1u64 << 32));
    debug_assert!(qn.2 < (1u64 << 32));
    debug_assert!(qn.3 < (1u64 << 32));

    // Multiply matrices qn * q
    (
        qn.0 * q.0 + qn.1 * q.2,
        qn.0 * q.1 + qn.1 * q.3,
        qn.2 * q.0 + qn.3 * q.2,
        qn.2 * q.1 + qn.3 * q.3,
        qn.4 ^ !q.4
    )
}

fn lehmer_update(
    a0: &mut U256,
    a1: &mut U256,
    (q00, q01, q10, q11, even): (u64, u64, u64, u64, bool),
) {
    if even {
        mat_mul(a0, a1, (q00, q01, q10, q11));
    } else {
        mat_mul(a0, a1, (q10, q11, q00, q01));
        std::mem::swap(a0, a1);
    }
}

// Simulataneously computes
//   a' = q00 a - q01 b
//   b' = q11 b - q10 a
fn mat_mul(
    a: &mut U256,
    b: &mut U256,
    (q00, q01, q10, q11): (u64, u64, u64, u64),
) {
    use crate::utils::{msb, mac};
    let (ai, ac) = mac( 0, q00, a.c0, 0);
    let (ai, ab) = msb(ai, q01, b.c0, 0);
    let (bi, bc) = mac( 0, q11, b.c0, 0);
    let (bi, bb) = msb(bi, q10, a.c0, 0);
    a.c0 = ai;
    b.c0 = bi;
    let (ai, ac) = mac( 0, q00, a.c1, ac);
    let (ai, ab) = msb(ai, q01, b.c1, ab);
    let (bi, bc) = mac( 0, q11, b.c1, bc);
    let (bi, bb) = msb(bi, q10, a.c1, bb);
    a.c1 = ai;
    b.c1 = bi;
    let (ai, ac) = mac( 0, q00, a.c2, ac);
    let (ai, ab) = msb(ai, q01, b.c2, ab);
    let (bi, bc) = mac( 0, q11, b.c2, bc);
    let (bi, bb) = msb(bi, q10, a.c2, bb);
    a.c2 = ai;
    b.c2 = bi;
    let (ai, _ac) = mac( 0, q00, a.c3, ac);
    let (ai, _ab) = msb(ai, q01, b.c3, ab);
    let (bi, _bc) = mac( 0, q11, b.c3, bc);
    let (bi, _bb) = msb(bi, q10, a.c3, bb);
    a.c3 = ai;
    b.c3 = bi;
}

#[rustfmt::skip]
pub fn gcd_lehmer(mut r0: U256, mut r1: U256) -> (U256, U256, U256, bool) {
    debug_assert!(r0 >= r1);
    // TODO: Support r1 >= r0
    let mut s0 = U256::ONE;
    let mut s1 = U256::ZERO;
    let mut t0 = U256::ZERO;
    let mut t1 = U256::ONE;
    let mut even = true;
    while r1 != U256::ZERO {
        let q = lehmer_double(r0.clone(), r1.clone());
        //println!("({:?}, {:?}, {:?}, {:?})", q.0, q.1, q.2, q.3);
        if q.2 != 0u64 {
            lehmer_update(&mut r0, &mut r1, q);
            lehmer_update(&mut s0, &mut s1, q);
            lehmer_update(&mut t0, &mut t1, q);
            even ^= !q.4;
        } else {
            // Do a full precision Euclid step. q is at least a halfword.
            // This should happen zero or one time, seldom more.
            // OPT: use single limb version when q is small enough?
            let q = &r0 / &r1;
            //println!("{:?} full precission", q);
            let t = r0 - &q * &r1; r0 = r1; r1 = t;
            let t = s0 - &q * &s1; s0 = s1; s1 = t;
            let t = t0 -  q * &t1; t0 = t1; t1 = t;
            even = !even;
        }
    }
    // TODO: Compute using absolute value instead of patching sign.
    if even {
        // t negative
        t0 = U256::ZERO - t0;
    } else {
        // s negative
        s0 = U256::ZERO - s0;
    }
    (r0, s0, t0, even)
}

#[rustfmt::skip]
pub fn inv_lehmer(modulus: &U256, num: &U256) -> Option<U256> {
    debug_assert!(modulus > num);
    let mut r0 = modulus.clone();
    let mut r1 = num.clone();
    let mut t0 = U256::ZERO;
    let mut t1 = U256::ONE;
    let mut even = true;
    while r1 != U256::ZERO {
        let q = lehmer_double(r0.clone(), r1.clone());
        if q.2 != 0u64 {
            lehmer_update(&mut r0, &mut r1, q);
            lehmer_update(&mut t0, &mut t1, q);
            even ^= !q.4;
        } else {
            // Do a full precision Euclid step. q is at least a halfword.
            // This should happen zero or one time, seldom more.
            let q = &r0 / &r1;
            let t = r0 - &q * &r1; r0 = r1; r1 = t;
            let t = t0 -  q * &t1; t0 = t1; t1 = t;
            even = !even;
        }
    }
    if r0 == U256::ONE {
        // When `even` t0 is negative and in twos-complement form
        Some(if even { modulus + t0 } else { t0 })
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unreadable_litteral)]

    use super::*;
    use crate::field::{FieldElement, MODULUS};
    use crate::u256h;
    use hex_literal::*;
    use quickcheck_macros::quickcheck;

    #[test]
    fn test_lehmer_small() {
        assert_eq!(lehmer_small(0, 0), (1, 0, 0, 1, true));
        assert_eq!(lehmer_small(0, 1), (0, 1, 1, 0, false));
        assert_eq!(
            lehmer_small(5818365597666026993, 14535145444257436950),
            (
                947685836737753349,
                379355176803460069,
                2076449349179633850,
                831195085380860999,
                true
            )
        );
        assert_eq!(
            lehmer_small(10841422679839906593, 15507080595343815048),
            (
                57434639988632077,
                40154122160696118,
                5169026865114605016,
                3613807559946635531,
                false
            )
        );
    }

    #[quickcheck]
    fn test_lehmer_loop(a: u64, b: u64) -> bool {
        let (u0, v0, u1, v1, even) = lehmer_loop(a, b);
        assert!(u0 < 1u64 << 32);
        assert!(v0 < 1u64 << 32);
        assert!(u1 < 1u64 << 32);
        assert!(v1 < 1u64 << 32);
        if even {
            false
        } else {
            false
        }
    }

    #[quickcheck]
    fn test_mat_mul(a: U256, b: U256, q00: u64, q01: u64, q10: u64, q11: u64) -> bool {
        let a_expected = q00 * a.clone() - q01 * b.clone();
        let b_expected = q11 * b.clone() - q10 * a.clone();
        let mut a_result = a;
        let mut b_result = b;
        mat_mul(&mut a_result, &mut b_result, (q00, q01, q10, q11));
        a_result == a_expected && b_result == b_expected
    }

    /*
    #[test]
    fn test_lehmer_loop() {
        assert_eq!(lehmer_loop(0, 0, 1, 0, 0, 1), (1, 0, 0, 1, true));
        assert_eq!(
            lehmer_loop(5818365597666026993, 14535145444257436950, 1, 0, 0, 1),
            (139667543, 55908407, 174687518, 69926775, false)
        );
        assert_eq!(
            lehmer_loop(
                6044159827974199924,
                6325623274722585764,
                4189569209,
                21585722,
                1706813914,
                1897815210
            ),
            (
                1130534579495951597,
                356413338079229448,
                1604599888673401540,
                505867589524443154,
                false
            )
        );
    }
    */

    #[test]
    fn test_lehmer_double() {
        assert_eq!(lehmer_double(U256::ZERO, U256::ZERO), (1, 0, 0, 1, true));
        assert_eq!(
            lehmer_double(
                u256h!("518a5cc4c55ac5b050a0831b65e827e5e39fd4515e4e094961c61509e7870814"),
                u256h!("018a5cc4c55ac5b050a0831b65e827e5e39fd4515e4e094961c61509e7870814")
            ),
            (
                24753544726280,
                1310252935479731,
                64710401929971,
                3425246566597885,
                false
            )
        );
    }

    #[test]
    fn test_lehmer_34() {
        assert_eq!(
            gcd_lehmer(
                u256h!("fea5a792d0a17b24827908e5524bcceec3ec6a92a7a42eac3b93e2bb351cf4f2"),
                u256h!("00028735553c6c798ed1ffb8b694f8f37b672b1bab7f80c4e6f4c0e710c79fb4")
            ),
            (
                u256h!("0000000000000000000000000000000000000000000000000000000000000002"),
                u256h!("00000b5a5ecb4dfc4ea08773d0593986592959a646b2f97655ed839928274ebb"),
                u256h!("0477865490d3994853934bf7eae7dad9afac55ccbf412a60c18fc9bea58ec8ba"),
                false
            )
        );
    }

    #[test]
    fn test_gcd_lehmer() {
        assert_eq!(
            gcd_lehmer(U256::ZERO, U256::ZERO),
            (U256::ZERO, U256::ONE, U256::ZERO, true)
        );
        assert_eq!(
            gcd_lehmer(
                u256h!("518a5cc4c55ac5b050a0831b65e827e5e39fd4515e4e094961c61509e7870814"),
                u256h!("018a5cc4c55ac5b050a0831b65e827e5e39fd4515e4e094961c61509e7870814")
            ),
            (
                U256::from(4u64),
                u256h!("002c851a0dddfaa03b9db2e39d48067d9b57fa0d238b70c7feddf8d267accc41"),
                u256h!("0934869c752ae9c7d2ed8aa55e7754e5492aaac49f8c9f3416156313a16c1174"),
                true
            )
        );
        assert_eq!(
            gcd_lehmer(
                u256h!("7dfd26515f3cd365ea32e1a43dbac87a25d0326fd834a889cb1e4c6c3c8d368c"),
                u256h!("3d341ef315cbe5b9f0ab79255f9684e153deaf5f460a8425819c84ec1e80a2f3")
            ),
            (
                u256h!("0000000000000000000000000000000000000000000000000000000000000001"),
                u256h!("0bbc35a0c1fd8f1ae85377ead5a901d4fbf0345fa303a87a4b4b68429cd69293"),
                u256h!("18283a24821b7de14cf22afb0e1a7efb4212b7f373988f5a0d75f6ee0b936347"),
                false
            )
        );
        assert_eq!(
            gcd_lehmer(
                u256h!("836fab5d425345751b3425e733e8150a17fdab2d5fb840ede5e0879f41497a4f"),
                u256h!("196e875b381eb95d9b5c6c3f198c5092b3ccc21279a7e68bc42cb6bca2d2644d")
            ),
            (
                u256h!("000000000000000000000000000000000000000000000000c59f8490536754fd"),
                u256h!("000000000000000006865401d85836d50a2bd608f152186fb24072a122d0dc5d"),
                u256h!("000000000000000021b8940f60792f546cbeb17f8b852d33a00b14b323d6de70"),
                false
            )
        );
        assert_eq!(
            gcd_lehmer(
                u256h!("00253222ed7b612113dbea0be0e1a0b88f2c0c16250f54bf1ec35d62671bf83a"),
                u256h!("0000000000025d4e064960ef2964b2170f1cd63ab931968621dde8a867079fd4")
            ),
            (
                u256h!("000000000000000000000000000505b22b0a9fd5a6e2166e3486f0109e6f60b2"),
                u256h!("0000000000000000000000000000000000000000000000001f16d40433587ae9"),
                u256h!("0000000000000000000000000000000000000001e91177fbec66b1233e79662e"),
                true
            )
        );
    }
}
