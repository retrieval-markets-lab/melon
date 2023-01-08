use blstrs::{pairing, G1Affine, G1Projective, G2Projective, Scalar};
use pairing::group::Group;
use thiserror::Error;

pub mod polynomial;

use polynomial::Polynomial;

use pairing::group::{ff::Field, prime::PrimeCurveAffine, Curve};
use std::fmt::Debug;

#[derive(Debug, Clone)]
pub struct KZGProver<'params> {
    parameters: &'params KZGParams,
}

/// parameters from tested setup
#[derive(Clone, Debug)]
pub struct KZGParams {
    /// g, g^alpha^1, g^alpha^2, ...
    pub gs: Vec<G1Projective>,
    /// h, h^alpha^1, h^alpha^2, ...
    pub hs: Vec<G2Projective>,
}

pub type KZGCommitment = G1Affine;
pub type KZGWitness = G1Affine;

#[derive(Error, Debug)]
pub enum KZGError {
    #[error("no polynomial!")]
    NoPolynomial,
    #[error("point not on polynomial!")]
    PointNotOnPolynomial,
    #[error("batch opening remainder is zero!")]
    BatchOpeningZeroRemainder,
    #[error("polynomial degree too large")]
    PolynomialDegreeTooLarge,
}

pub fn setup(s: Scalar, num_coeffs: usize) -> KZGParams {
    let mut gs = vec![G1Projective::generator(); num_coeffs];
    let mut hs = vec![G2Projective::generator(); num_coeffs];

    let mut curr = gs[0];
    for g in gs.iter_mut().skip(1) {
        *g = curr * s;
        curr = *g;
    }

    let mut curr = hs[0];
    for h in hs.iter_mut().skip(1) {
        *h = curr * s;
        curr = *h;
    }

    KZGParams { gs, hs }
}

#[derive(Debug, Clone)]
pub struct KZGVerifier<'params> {
    parameters: &'params KZGParams,
}

impl<'params> KZGProver<'params> {
    /// initializes `polynomial` to zero polynomial
    pub fn new(parameters: &'params KZGParams) -> Self {
        Self { parameters }
    }

    pub fn parameters(&self) -> &'params KZGParams {
        self.parameters
    }

    pub fn commit(&self, polynomial: &Polynomial) -> KZGCommitment {
        let gs = &self.parameters.gs[..polynomial.num_coeffs()];
        let commitment = G1Projective::multi_exp(gs, polynomial.slice_coeffs());

        commitment.to_affine()
    }

    pub fn create_witness(
        &self,
        polynomial: &Polynomial,
        (x, y): (Scalar, Scalar),
    ) -> Result<KZGWitness, KZGError> {
        let mut dividend = polynomial.clone();
        let degree = dividend.degree();
        dividend.coeffs[0] -= y;

        let mut remainder = polynomial.clone();
        let mut divpoly = Polynomial::new_from_coeffs(vec![Scalar::zero(); degree], degree - 1);

        for i in (1..=degree).rev() {
            let factor = remainder.coeffs[i];
            divpoly.coeffs[i - 1] = factor;
            remainder.coeffs[i - 1] = remainder.coeffs[i - 1] + x * factor;
        }

        if divpoly.num_coeffs() == 1 {
            Ok((self.parameters.gs[0] * divpoly.coeffs[0]).to_affine())
        } else {
            Ok(self.commit(&divpoly))
        }
    }
}

impl<'params> KZGVerifier<'params> {
    pub fn new(parameters: &'params KZGParams) -> Self {
        KZGVerifier { parameters }
    }

    pub fn verify_poly(&self, commitment: &KZGCommitment, polynomial: &Polynomial) -> bool {
        let gs = &self.parameters.gs[..polynomial.num_coeffs()];
        let check = G1Projective::multi_exp(gs, polynomial.slice_coeffs());

        check.to_affine() == *commitment
    }

    pub fn verify_eval(
        &self,
        (x, y): (Scalar, Scalar),
        commitment: &KZGCommitment,
        witness: &KZGWitness,
    ) -> bool {
        let lhs = pairing(
            witness,
            &(self.parameters.hs[1] - self.parameters.hs[0] * x).to_affine(),
        );
        let rhs = pairing(
            &(commitment.to_curve() - self.parameters.gs[0] * y).to_affine(),
            &self.parameters.hs[0].to_affine(),
        );

        lhs == rhs
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kzg::setup;
    use pairing::group::ff::Field;
    use rand::{rngs::SmallRng, Rng, SeedableRng};

    const RNG_SEED: [u8; 32] = [69; 32];

    fn test_setup<const MAX_COEFFS: usize>(rng: &mut SmallRng) -> KZGParams {
        let s: Scalar = rng.gen::<u64>().into();
        setup(s, MAX_COEFFS)
    }

    fn test_participants(params: &'_ KZGParams) -> (KZGProver<'_>, KZGVerifier<'_>) {
        let prover = KZGProver::new(params);
        let verifier = KZGVerifier::new(params);

        (prover, verifier)
    }

    // never returns zero polynomial
    fn random_polynomial(rng: &mut SmallRng, min_coeffs: usize, max_coeffs: usize) -> Polynomial {
        let num_coeffs = rng.gen_range(min_coeffs..max_coeffs);
        let mut coeffs = vec![Scalar::zero(); max_coeffs];

        for coeff in coeffs.iter_mut().take(num_coeffs) {
            *coeff = rng.gen::<u64>().into();
        }

        let mut poly = Polynomial::new_from_coeffs(coeffs, num_coeffs - 1);
        poly.shrink_degree();
        poly
    }

    fn assert_verify_poly(
        verifier: &KZGVerifier,
        commitment: &KZGCommitment,
        polynomial: &Polynomial,
    ) {
        assert!(
            verifier.verify_poly(commitment, polynomial),
            "verify_poly failed for commitment {:#?} and polynomial {:#?}",
            commitment,
            polynomial
        );
    }

    fn assert_verify_poly_fails(
        verifier: &KZGVerifier,
        commitment: &KZGCommitment,
        polynomial: &Polynomial,
    ) {
        assert!(
            !verifier.verify_poly(commitment, polynomial),
            "expected verify_poly to fail for commitment {:#?} and polynomial {:#?} but it didn't",
            commitment,
            polynomial
        );
    }

    fn assert_verify_eval(
        verifier: &KZGVerifier,
        point: (Scalar, Scalar),
        commitment: &KZGCommitment,
        witness: &KZGWitness,
    ) {
        assert!(
            verifier.verify_eval(point, commitment, witness),
            "verify_eval failed for point {:#?}, commitment {:#?}, and witness {:#?}",
            point,
            commitment,
            witness
        );
    }

    fn assert_verify_eval_fails(
        verifier: &KZGVerifier,
        point: (Scalar, Scalar),
        commitment: &KZGCommitment,
        witness: &KZGWitness,
    ) {
        assert!(!verifier.verify_eval(point, commitment, witness), "expected verify_eval to fail for for point {:#?}, commitment {:#?}, and witness {:#?}, but it didn't", point, commitment, witness);
    }

    #[test]
    fn test_basic() {
        let mut rng = SmallRng::from_seed(RNG_SEED);
        let params = test_setup::<12>(&mut rng);

        let (prover, verifier) = test_participants(&params);

        let polynomial = random_polynomial(&mut rng, 2, 12);
        let commitment = prover.commit(&polynomial);

        assert_verify_poly(&verifier, &commitment, &polynomial);
        assert_verify_poly_fails(&verifier, &commitment, &random_polynomial(&mut rng, 2, 12));
    }

    fn random_field_elem_neq(val: Scalar) -> Scalar {
        let mut rng = SmallRng::from_seed(RNG_SEED);
        let mut v: Scalar = rng.gen::<u64>().into();
        while v == val {
            v = rng.gen::<u64>().into();
        }

        v
    }

    #[test]
    fn test_modify_single_coeff() {
        let mut rng = SmallRng::from_seed(RNG_SEED);
        let params = test_setup::<8>(&mut rng);

        let (prover, verifier) = test_participants(&params);

        let polynomial = random_polynomial(&mut rng, 3, 8);
        let commitment = prover.commit(&polynomial);

        let mut modified_polynomial = polynomial.clone();
        let new_coeff = random_field_elem_neq(modified_polynomial.coeffs[2]);
        modified_polynomial.coeffs[2] = new_coeff;

        assert_verify_poly(&verifier, &commitment, &polynomial);
        assert_verify_poly_fails(&verifier, &commitment, &modified_polynomial);
    }

    #[test]
    fn test_eval_basic() {
        let mut rng = SmallRng::from_seed(RNG_SEED);
        let params = test_setup::<13>(&mut rng);

        let (prover, verifier) = test_participants(&params);

        let polynomial = random_polynomial(&mut rng, 5, 13);
        let commitment = prover.commit(&polynomial);

        let x: Scalar = rng.gen::<u64>().into();
        let y = polynomial.eval(x);

        let witness = prover.create_witness(&polynomial, (x, y)).unwrap();
        assert_verify_eval(&verifier, (x, y), &commitment, &witness);

        let y_prime = random_field_elem_neq(y);
        assert_verify_eval_fails(&verifier, (x, y_prime), &commitment, &witness);

        // test degree 1 edge case
        let mut coeffs = vec![Scalar::zero(); 13];
        coeffs[0] = 3.into();
        coeffs[1] = 1.into();
        let polynomial = Polynomial::new(coeffs);

        let commitment = prover.commit(&polynomial);
        let witness = prover
            .create_witness(&polynomial, (1.into(), 4.into()))
            .unwrap();
        assert_verify_eval(&verifier, (1.into(), 4.into()), &commitment, &witness);
        assert_verify_eval_fails(&verifier, (1.into(), 5.into()), &commitment, &witness);
    }
}
