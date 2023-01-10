use ark_bn254::{Fr as Scalar, G1Affine};
use ark_ff::{One, PrimeField, Zero};
use melon::kzg::polynomial::Polynomial;
use melon::kzg::{setup, KZGCommitment, KZGParams, KZGProver, KZGVerifier};
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
            x: format!("0x{}", point.x.into_repr()),
            y: format!("0x{}", point.y.into_repr()),
            i: "0x".to_string(),
            value: "0x".to_string(),
        }
    }
}

pub fn csprng_setup<const MAX_COEFFS: usize>() -> KZGParams {
    let s: Scalar = Scalar::one();
    setup(s, MAX_COEFFS)
}

fn create_commit<const NUM_COEFFS: usize>() -> (Polynomial, KZGCommitment, KZGParams) {
    let params = csprng_setup::<NUM_COEFFS>();
    let mut rng = SmallRng::from_seed([42; 32]);
    let mut coeffs = vec![Scalar::zero(); NUM_COEFFS];
    for coeff in coeffs.iter_mut().take(NUM_COEFFS) {
        *coeff = rng.gen::<u64>().into();
    }
    let polynomial = Polynomial::new_from_coeffs(coeffs, NUM_COEFFS - 1);
    let prover = KZGProver::new(&params);
    let commitment = prover.commit(&polynomial);

    let commitment_json: JSONG1Affine = commitment.into();

    serde_json::to_writer(&File::create("commitment.json").unwrap(), &commitment_json).unwrap();

    (polynomial, commitment, params)
}

fn create_witness<const NUM_COEFFS: usize>(
    polynomial: Polynomial,
    commitment: KZGCommitment,
    params: KZGParams,
) {
    let prover = KZGProver::new(&params);
    let mut rng = SmallRng::from_seed([42; 32]);
    let x: Scalar = rng.gen::<u64>().into();
    let y = polynomial.eval(x);

    let wit = prover.create_witness(&polynomial, (x, y)).unwrap();
    let verifier = KZGVerifier::new(&params);
    assert!(verifier.verify_eval((x, y), &commitment, &wit));

    let mut wit_json: JSONG1Affine = wit.into();
    wit_json.i = format!("0x{}", x.into_repr());
    wit_json.value = format!("0x{}", y.into_repr());

    serde_json::to_writer(&File::create("witness.json").unwrap(), &wit_json).unwrap();
}

fn main() {
    let (poly, commitment, params) = create_commit::<8>();
    create_witness::<8>(poly, commitment, params);
}
