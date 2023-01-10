use ark_bn254::Fr as Scalar;
use ark_ff::{UniformRand, Zero};
use melon::kzg::polynomial::Polynomial;
use melon::kzg::{setup, KZGParams, KZGProver};
use rand::{rngs::SmallRng, Rng, SeedableRng};

use criterion::{black_box, criterion_group, criterion_main, Criterion};

pub fn csprng_setup<const MAX_COEFFS: usize>() -> KZGParams {
    let s: Scalar = rand::random::<u64>().into();
    setup(s, MAX_COEFFS)
}

fn bench_create_witness<const NUM_COEFFS: usize>(c: &mut Criterion) {
    let params = csprng_setup::<NUM_COEFFS>();
    let mut rng = SmallRng::from_seed([42; 32]);
    let mut coeffs = vec![Scalar::zero(); NUM_COEFFS];
    for coeff in coeffs.iter_mut().take(NUM_COEFFS) {
        *coeff = rng.gen::<u64>().into();
    }
    let polynomial = Polynomial::new_from_coeffs(coeffs, NUM_COEFFS - 1);
    let prover = KZGProver::new(&params);
    let _commitment = prover.commit(&polynomial);

    let x: Scalar = Scalar::rand(&mut rng);
    let y = polynomial.eval(x);

    c.bench_function(
        format!("create_witness, degree {}", NUM_COEFFS - 1).as_str(),
        |b| {
            b.iter(|| {
                black_box(&prover)
                    .create_witness(black_box(&polynomial), black_box((x, y)))
                    .unwrap()
            })
        },
    );
}

criterion_group!(
    name = create_witness;
    config = Criterion::default();
    targets = bench_create_witness<16>, bench_create_witness<64>, bench_create_witness<128>, bench_create_witness<256>, bench_create_witness<512>, bench_create_witness<1024>, bench_create_witness<2048>, bench_create_witness<5096>
);
criterion_main!(create_witness);
