use ark_bn254::Fr as Scalar;
use ark_ff::{UniformRand, Zero};
use melon::kzg::polynomial::Polynomial;
use rand::{rngs::SmallRng, Rng, SeedableRng};

use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn random_polynomial(rng: &mut SmallRng, n: usize) -> Polynomial {
    let mut coeffs = vec![Scalar::zero(); n];
    for coeff in coeffs.iter_mut().take(n) {
        *coeff = rng.gen::<u64>().into();
    }
    Polynomial::new(coeffs)
}

fn poly_arithmetic<const NUM_COEFFS: usize>(c: &mut Criterion) {
    let mut rng = SmallRng::from_seed([NUM_COEFFS as u8; 32]);
    let f = random_polynomial(&mut rng, NUM_COEFFS);
    let g = random_polynomial(&mut rng, NUM_COEFFS);

    c.bench_function(format!("add, degree {}", NUM_COEFFS - 1).as_str(), |b| {
        b.iter(|| black_box(f.clone()) + black_box(g.clone()));
    });

    c.bench_function(
        format!("mul_naive, degree {}", NUM_COEFFS - 1).as_str(),
        |b| {
            b.iter(|| black_box(f.clone()) * black_box(g.clone()));
        },
    );

    let mut xs = vec![Scalar::zero(); NUM_COEFFS - 1];
    let mut ys = vec![Scalar::zero(); NUM_COEFFS - 1];
    for i in 0..xs.len() {
        xs[i] = Scalar::rand(&mut rng);
        ys[i] = Scalar::rand(&mut rng);
    }

    c.bench_function(
        format!("interpolation, degree {}", NUM_COEFFS - 1).as_str(),
        |b| b.iter(|| Polynomial::lagrange_interpolation(xs.as_slice(), ys.as_slice())),
    );
}

criterion_group!(
    name = bench_poly_arithmetic;
    config = Criterion::default();
    targets = poly_arithmetic<16>, poly_arithmetic<64>, poly_arithmetic<128>, poly_arithmetic<256>, poly_arithmetic<512>, poly_arithmetic<1024>, poly_arithmetic<2048>, poly_arithmetic<5098>
);
criterion_main!(bench_poly_arithmetic);
