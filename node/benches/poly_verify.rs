use blstrs::Scalar;
use melon::kzg::polynomial::Polynomial;
use melon::kzg::{
    setup, KZGParams, {KZGProver, KZGVerifier},
};
use pairing::group::ff::Field;
use rand::{rngs::SmallRng, Rng, SeedableRng};

use criterion::{black_box, criterion_group, criterion_main, Criterion};

pub fn csprng_setup<const MAX_COEFFS: usize>() -> KZGParams {
    let s: Scalar = rand::random::<u64>().into();
    setup(s, MAX_COEFFS)
}

fn poly_verify<const NUM_COEFFS: usize>(c: &mut Criterion) {
    let params = csprng_setup::<NUM_COEFFS>();
    let mut rng = SmallRng::from_seed([42; 32]);
    let mut coeffs = vec![Scalar::zero(); NUM_COEFFS];
    for coeff in coeffs.iter_mut().take(NUM_COEFFS) {
        *coeff = rng.gen::<u64>().into();
    }
    let polynomial = Polynomial::new_from_coeffs(coeffs, NUM_COEFFS - 1);
    let prover = KZGProver::new(&params);
    let verifier = KZGVerifier::new(&params);
    let commitment = prover.commit(&polynomial);

    c.bench_function(
        format!("verify_poly, degree {}", NUM_COEFFS - 1).as_str(),
        |b| {
            b.iter(|| {
                verifier.verify_poly(black_box(&commitment), black_box(&polynomial));
            })
        },
    );
}

criterion_group!(
    name = bench_poly_verify;
    config = Criterion::default();
    targets = poly_verify<16>, poly_verify<64>, poly_verify<128>, poly_verify<256>, poly_verify<512>, poly_verify<1024>, poly_verify<2048>, poly_verify<5096>
);
criterion_main!(bench_poly_verify);
