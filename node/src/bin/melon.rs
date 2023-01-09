use ark_bn254::{Fr as Scalar, G1Affine};
use ark_ff::{BigInteger256, PrimeField, Zero};
use melon::kzg::polynomial::Polynomial;
use melon::kzg::{setup, KZGParams, KZGProver};
use rand::{rngs::SmallRng, Rng, SeedableRng};
use serde::{Deserialize, Serialize};
use std::fs::File;

#[derive(Serialize, Deserialize)]
struct JSONG1Affine {
    x: String,
    y: String,
    i: String,
    value: String,
}

impl From<G1Affine> for JSONG1Affine {
    fn from(point: G1Affine) -> Self {
        JSONG1Affine {
            x: format!("0x{}", hex::encode(point.x.into_repr().to_bytes_be())),
            y: format!("0x{}", hex::encode(point.y.into_repr().to_bytes_be())),
            i: "0x".to_string(),
            value: "0x".to_string(),
        }
    }
}

pub fn csprng_setup<const MAX_COEFFS: usize>() -> KZGParams {
    let s: Scalar = rand::random::<u64>().into();
    setup(s, MAX_COEFFS)
}

fn create_commit<const NUM_COEFFS: usize>() -> (Polynomial, KZGParams) {
    let params = csprng_setup::<NUM_COEFFS>();
    let mut rng = SmallRng::from_seed([42; 32]);
    let mut coeffs = vec![Scalar::zero(); NUM_COEFFS];
    for coeff in coeffs.iter_mut().take(NUM_COEFFS) {
        *coeff = rng.gen::<u64>().into();
    }
    let polynomial = Polynomial::new_from_coeffs(coeffs, NUM_COEFFS - 1);
    let prover = KZGProver::new(&params);
    let commitment: JSONG1Affine = prover.commit(&polynomial).into();

    serde_json::to_writer(&File::create("commitment.json").unwrap(), &commitment).unwrap();

    (polynomial, params)
}

fn create_witness<const NUM_COEFFS: usize>(polynomial: Polynomial, params: KZGParams) {
    let prover = KZGProver::new(&params);
    let mut rng = SmallRng::from_seed([42; 32]);
    let x: Scalar = rng.gen::<u64>().into();
    let y = polynomial.eval(x);

    let mut wit: JSONG1Affine = prover.create_witness(&polynomial, (x, y)).unwrap().into();
    wit.i = format!("0x{}", hex::encode(x.into_repr().to_bytes_be()));
    wit.value = format!("0x{}", hex::encode(y.into_repr().to_bytes_be()));

    serde_json::to_writer(&File::create("witness.json").unwrap(), &wit).unwrap();
}

fn main() {
    let (poly, params) = create_commit::<8>();
    create_witness::<8>(poly, params);
}
