#![warn(clippy::all)]
#![deny(warnings)]
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use ecc::{mul, private_to_public, sign, verify, Affine, Jacobian};
use hex_literal::*;
use primefield::{u256h, FieldElement, U256};

fn curve_add(crit: &mut Criterion) {
    let a = Affine::Point {
        x: FieldElement::new(&[
            0xca9b3b7a, 0xadf5b0d8, 0x4728f1b4, 0x7a5cbd79, 0x316a86d0, 0xb9aaaf56, 0x557c9ca9,
            0x0259dee2,
        ]),
        y: FieldElement::new(&[
            0x68173fdd, 0x25daa0d2, 0xcd94b717, 0x4f84a316, 0xd637a579, 0x236d898d, 0x787b7c9e,
            0x011cf020,
        ]),
    };
    let b = Affine::Point {
        x: FieldElement::new(&[
            0x55893510, 0x5985d659, 0xc0cda9ae, 0xfb1db2ec, 0xc78fe4ec, 0xe60f0d63, 0xfb0e0cf5,
            0x0449895d,
        ]),
        y: FieldElement::new(&[
            0x1b78e1cc, 0x86e1e27b, 0x80a13dd1, 0x157492ef, 0x8191f8ae, 0x7fb47371, 0x8d4ef0e6,
            0x07cfb4b0,
        ]),
    };
    crit.bench_function("Curve add", move |bench| {
        bench.iter(|| {
            black_box(black_box(&a) + black_box(&b));
        })
    });
}

fn curve_dbl(crit: &mut Criterion) {
    let a = Affine::Point {
        x: FieldElement::new(&[
            0xa19caf1f, 0x9764694b, 0xd49d26e1, 0xc2d21cea, 0x9d37cc5b, 0xce13e7e3, 0x787be6e0,
            0x00ea1dff,
        ]),
        y: FieldElement::new(&[
            0xce7296f0, 0xd1f6f7df, 0xc9c5b41c, 0x6b889413, 0xc9449f06, 0xf44da1a6, 0x302e9f91,
            0x011b6c17,
        ]),
    };
    crit.bench_function("Curve dbl", move |bench| {
        bench.iter(|| {
            black_box(black_box(&a).double());
        })
    });
}

fn curve_mul(crit: &mut Criterion) {
    let a = Affine::Point {
        x: FieldElement::new(&[
            0x5bf31eb0, 0xfe50a889, 0x2d1a8a21, 0x3242e28e, 0x0d13fe66, 0xcf63e064, 0x9426e2c3,
            0x0040ffd5,
        ]),
        y: FieldElement::new(&[
            0xe29859d2, 0xd21b931a, 0xea34d27d, 0x296f19b9, 0x6487ae5b, 0x524260f9, 0x069092ca,
            0x060c2257,
        ]),
    };
    let b = u256h!("014023b44fbb1e6f2a79c929c6da775be3c4b9e043d439385b5050fdc69177e3");
    crit.bench_function("Curve mul", move |bench| {
        bench.iter(|| {
            black_box(black_box(&a) * black_box(&b));
        })
    });
}

fn jacobian_to_affine(crit: &mut Criterion) {
    let a = Jacobian::from(Affine::Point {
        x: FieldElement::new(&[
            0xca9b3b7a, 0xadf5b0d8, 0x4728f1b4, 0x7a5cbd79, 0x316a86d0, 0xb9aaaf56, 0x557c9ca9,
            0x0259dee2,
        ]),
        y: FieldElement::new(&[
            0x68173fdd, 0x25daa0d2, 0xcd94b717, 0x4f84a316, 0xd637a579, 0x236d898d, 0x787b7c9e,
            0x011cf020,
        ]),
    });
    crit.bench_function("Jacobian to Affine", move |bench| {
        bench.iter(|| {
            black_box(Affine::from(black_box(&a)));
        })
    });
}

fn jacobian_add(crit: &mut Criterion) {
    let a = Jacobian::from(Affine::Point {
        x: FieldElement::new(&[
            0xca9b3b7a, 0xadf5b0d8, 0x4728f1b4, 0x7a5cbd79, 0x316a86d0, 0xb9aaaf56, 0x557c9ca9,
            0x0259dee2,
        ]),
        y: FieldElement::new(&[
            0x68173fdd, 0x25daa0d2, 0xcd94b717, 0x4f84a316, 0xd637a579, 0x236d898d, 0x787b7c9e,
            0x011cf020,
        ]),
    });
    let b = Jacobian::from(Affine::Point {
        x: FieldElement::new(&[
            0x55893510, 0x5985d659, 0xc0cda9ae, 0xfb1db2ec, 0xc78fe4ec, 0xe60f0d63, 0xfb0e0cf5,
            0x0449895d,
        ]),
        y: FieldElement::new(&[
            0x1b78e1cc, 0x86e1e27b, 0x80a13dd1, 0x157492ef, 0x8191f8ae, 0x7fb47371, 0x8d4ef0e6,
            0x07cfb4b0,
        ]),
    });
    crit.bench_function("Jacobian add", move |bench| {
        bench.iter(|| {
            black_box(black_box(&a) + black_box(&b));
        })
    });
}

fn jacobian_add_affine(crit: &mut Criterion) {
    let a = Jacobian::from(Affine::Point {
        x: FieldElement::new(&[
            0xca9b3b7a, 0xadf5b0d8, 0x4728f1b4, 0x7a5cbd79, 0x316a86d0, 0xb9aaaf56, 0x557c9ca9,
            0x0259dee2,
        ]),
        y: FieldElement::new(&[
            0x68173fdd, 0x25daa0d2, 0xcd94b717, 0x4f84a316, 0xd637a579, 0x236d898d, 0x787b7c9e,
            0x011cf020,
        ]),
    });
    let b = Affine::Point {
        x: FieldElement::new(&[
            0x55893510, 0x5985d659, 0xc0cda9ae, 0xfb1db2ec, 0xc78fe4ec, 0xe60f0d63, 0xfb0e0cf5,
            0x0449895d,
        ]),
        y: FieldElement::new(&[
            0x1b78e1cc, 0x86e1e27b, 0x80a13dd1, 0x157492ef, 0x8191f8ae, 0x7fb47371, 0x8d4ef0e6,
            0x07cfb4b0,
        ]),
    };
    crit.bench_function("Jacobian add affine", move |bench| {
        bench.iter(|| {
            black_box(black_box(&a) + black_box(&b));
        })
    });
}

fn jacobian_dbl(crit: &mut Criterion) {
    let a = Jacobian::from(Affine::Point {
        x: FieldElement::new(&[
            0x5bf31eb0, 0xfe50a889, 0x2d1a8a21, 0x3242e28e, 0x0d13fe66, 0xcf63e064, 0x9426e2c3,
            0x0040ffd5,
        ]),
        y: FieldElement::new(&[
            0xe29859d2, 0xd21b931a, 0xea34d27d, 0x296f19b9, 0x6487ae5b, 0x524260f9, 0x069092ca,
            0x060c2257,
        ]),
    });
    crit.bench_function("Jacobian dbl", move |bench| {
        bench.iter(|| {
            black_box(black_box(&a).double());
        })
    });
}

fn jacobian_mul(crit: &mut Criterion) {
    let a = Jacobian::from(Affine::Point {
        x: FieldElement::new(&[
            0x5bf31eb0, 0xfe50a889, 0x2d1a8a21, 0x3242e28e, 0x0d13fe66, 0xcf63e064, 0x9426e2c3,
            0x0040ffd5,
        ]),
        y: FieldElement::new(&[
            0xe29859d2, 0xd21b931a, 0xea34d27d, 0x296f19b9, 0x6487ae5b, 0x524260f9, 0x069092ca,
            0x060c2257,
        ]),
    });
    let b = u256h!("014023b44fbb1e6f2a79c929c6da775be3c4b9e043d439385b5050fdc69177e3");
    crit.bench_function("Jacobian mul", move |bench| {
        bench.iter(|| {
            black_box(black_box(&a) * black_box(&b));
        })
    });
}

fn jacobian_mul_affine(crit: &mut Criterion) {
    let a = Affine::Point {
        x: FieldElement::new(&[
            0x5bf31eb0, 0xfe50a889, 0x2d1a8a21, 0x3242e28e, 0x0d13fe66, 0xcf63e064, 0x9426e2c3,
            0x0040ffd5,
        ]),
        y: FieldElement::new(&[
            0xe29859d2, 0xd21b931a, 0xea34d27d, 0x296f19b9, 0x6487ae5b, 0x524260f9, 0x069092ca,
            0x060c2257,
        ]),
    };
    let b = u256h!("014023b44fbb1e6f2a79c929c6da775be3c4b9e043d439385b5050fdc69177e3");
    crit.bench_function("Jacobian mul affine", move |bench| {
        bench.iter(|| {
            black_box(Jacobian::mul(black_box(&a), black_box(&b)));
        })
    });
}

fn wnaf_mul_affine(crit: &mut Criterion) {
    let a = Affine::Point {
        x: FieldElement::new(&[
            0x5bf31eb0, 0xfe50a889, 0x2d1a8a21, 0x3242e28e, 0x0d13fe66, 0xcf63e064, 0x9426e2c3,
            0x0040ffd5,
        ]),
        y: FieldElement::new(&[
            0xe29859d2, 0xd21b931a, 0xea34d27d, 0x296f19b9, 0x6487ae5b, 0x524260f9, 0x069092ca,
            0x060c2257,
        ]),
    };
    let b = u256h!("014023b44fbb1e6f2a79c929c6da775be3c4b9e043d439385b5050fdc69177e3");
    crit.bench_function("Wnaf mul", move |bench| {
        bench.iter(|| {
            black_box(mul(black_box(&a), black_box(&b)));
        })
    });
}

fn ecdsa_sign(crit: &mut Criterion) {
    let message_hash = u256h!("03d937c035c878245caf64531a5756109c53068da139362728feb561405371cb");
    let private_key = u256h!("0208a0a10250e382e1e4bbe2880906c2791bf6275695e02fbbc6aeff9cd8b31a");
    crit.bench_function("Ecdsa sign", move |bench| {
        bench.iter(|| black_box(sign(&message_hash, &private_key)))
    });
}

fn ecdsa_verify(crit: &mut Criterion) {
    let message_hash = u256h!("03d937c035c878245caf64531a5756109c53068da139362728feb561405371cb");
    let private_key = u256h!("0208a0a10250e382e1e4bbe2880906c2791bf6275695e02fbbc6aeff9cd8b31a");
    let public = private_to_public(&private_key);
    let (r, w) = sign(&message_hash, &private_key);
    crit.bench_function("Ecdsa verify", move |bench| {
        bench.iter(|| black_box(verify(&message_hash, &r, &w, &public)))
    });
}

fn criterion_benchmark(c: &mut Criterion) {
    curve_add(c);
    curve_dbl(c);
    curve_mul(c);
    jacobian_add(c);
    jacobian_add_affine(c);
    jacobian_dbl(c);
    jacobian_mul(c);
    jacobian_mul_affine(c);
    jacobian_to_affine(c);
    wnaf_mul_affine(c);
    ecdsa_sign(c);
    ecdsa_verify(c);
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
