// copypasta from zkcrypto/bellman

use std::ops::{AddAssign, MulAssign, SubAssign};

use super::polynomial::Polynomial;
use super::KZGError;
use blstrs::Scalar;
use pairing::group::ff::Field;
use pairing::group::ff::PrimeField;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvaluationDomain {
    pub(crate) coeffs: Vec<Scalar>,
    pub(crate) d: usize,
    pub(crate) exp: u32,
    pub(crate) omega: Scalar,
    pub(crate) omegainv: Scalar,
    pub(crate) geninv: Scalar,
    pub(crate) minv: Scalar,
}

impl From<EvaluationDomain> for Polynomial {
    fn from(domain: EvaluationDomain) -> Polynomial {
        Polynomial::new(domain.coeffs)
    }
}

impl AsRef<[Scalar]> for EvaluationDomain {
    fn as_ref(&self) -> &[Scalar] {
        &self.coeffs
    }
}

impl AsMut<[Scalar]> for EvaluationDomain {
    fn as_mut(&mut self) -> &mut [Scalar] {
        &mut self.coeffs
    }
}

impl EvaluationDomain {
    // returns m, exp, and omega
    pub fn compute_omega(d: usize) -> Result<(usize, u32, Scalar), KZGError> {
        // Compute the size of our evaluation domain
        let mut m = 1;
        let mut exp = 0;

        // TODO cache this in a lazy static
        while m < d {
            m *= 2;
            exp += 1;

            // The pairing-friendly curve may not be able to support
            // large enough (radix2) evaluation domains.
            if exp >= Scalar::S {
                return Err(KZGError::PolynomialDegreeTooLarge);
            }
        }

        // Compute omega, the 2^exp primitive root of unity
        let omega = Scalar::root_of_unity().pow_vartime([1 << (Scalar::S - exp)]);

        Ok((m, exp, omega))
    }

    pub fn new(coeffs: Vec<Scalar>, d: usize, exp: u32, omega: Scalar) -> Self {
        EvaluationDomain {
            coeffs,
            d,
            exp,
            omega,
            omegainv: omega.invert().unwrap(),
            geninv: Scalar::multiplicative_generator().invert().unwrap(),
            minv: Scalar::from(d as u64).invert().unwrap(),
        }
    }

    pub fn from_coeffs(mut coeffs: Vec<Scalar>) -> Result<EvaluationDomain, KZGError> {
        let (m, exp, omega) = Self::compute_omega(coeffs.len())?;

        // Extend the coeffs vector with zeroes if necessary
        coeffs.resize(m, Scalar::zero());

        Ok(EvaluationDomain {
            d: m,
            coeffs,
            exp,
            omega,
            omegainv: omega.invert().unwrap(),
            geninv: Scalar::multiplicative_generator().invert().unwrap(),
            minv: Scalar::from(m as u64).invert().unwrap(),
        })
    }

    pub fn fft(&mut self) {
        best_fft(&mut self.coeffs, &self.omega, self.exp);
    }

    pub fn ifft(&mut self) {
        best_fft(&mut self.coeffs, &self.omegainv, self.exp);
        {
            let minv = self.minv;
            for v in self.coeffs.iter_mut() {
                v.mul_assign(&minv);
            }
        }
    }

    /// Perform O(n) multiplication of two polynomials in the domain.
    pub fn mul_assign(&mut self, other: &EvaluationDomain) {
        assert_eq!(self.coeffs.len(), other.coeffs.len());

        for (a, b) in self.coeffs.iter_mut().zip(other.coeffs.iter()) {
            a.mul_assign(b);
        }
    }
}

fn best_fft(a: &mut [Scalar], omega: &Scalar, log_n: u32) {
    serial_fft(a, omega, log_n);
}

#[allow(clippy::many_single_char_names)]
fn serial_fft(a: &mut [Scalar], omega: &Scalar, log_n: u32) {
    fn bitreverse(mut n: u32, l: u32) -> u32 {
        let mut r = 0;
        for _ in 0..l {
            r = (r << 1) | (n & 1);
            n >>= 1;
        }
        r
    }

    let n = a.len() as u32;
    assert_eq!(n, 1 << log_n);

    for k in 0..n {
        let rk = bitreverse(k, log_n);
        if k < rk {
            a.swap(rk as usize, k as usize);
        }
    }

    let mut m = 1;
    for _ in 0..log_n {
        let w_m = omega.pow_vartime([u64::from(n / (2 * m))]);

        let mut k = 0;
        while k < n {
            let mut w = Scalar::one();
            for j in 0..m {
                let mut t = a[(k + j + m) as usize];
                t.mul_assign(&w);
                let mut tmp = a[(k + j) as usize];
                tmp.sub_assign(&t);
                a[(k + j + m) as usize] = tmp;
                a[(k + j) as usize].add_assign(&t);
                w.mul_assign(&w_m);
            }

            k += 2 * m;
        }

        m *= 2;
    }
}

#[cfg(test)]
use rand::{rngs::SmallRng, Rng, SeedableRng};

// Test multiplying various (low degree) polynomials together and
// comparing with naive evaluations.
#[test]
fn polynomial_arith() {
    fn test_mul<R: Rng>(mut rng: &mut R) {
        for coeffs_a in &[1, 5, 10, 50] {
            for coeffs_b in &[1, 5, 10, 50] {
                let a: Vec<_> = (0..*coeffs_a).map(|_| Scalar::random(&mut rng)).collect();
                let b: Vec<_> = (0..*coeffs_b).map(|_| Scalar::random(&mut rng)).collect();

                let a = Polynomial::new_from_coeffs(a, coeffs_a - 1);
                let b = Polynomial::new_from_coeffs(b, coeffs_b - 1);

                // naive evaluation
                let naive = a.clone() * b.clone();
                let fft = a.fft_mul(&b);

                assert!(naive == fft);
            }
        }
    }

    let rng = &mut SmallRng::from_seed([42; 32]);

    test_mul(rng);
}

#[test]
fn fft_composition() {
    use rand::RngCore;

    fn test_comp<R: RngCore>(mut rng: &mut R) {
        for coeffs in 0..10 {
            let coeffs = 1 << coeffs;

            let mut v = vec![];
            for _ in 0..coeffs {
                v.push(Scalar::random(&mut rng));
            }

            let mut domain = EvaluationDomain::from_coeffs(v.clone()).unwrap();
            domain.ifft();
            domain.fft();
            assert!(v == domain.coeffs);
            domain.fft();
            domain.ifft();
            assert!(v == domain.coeffs);
        }
    }

    let rng = &mut rand::thread_rng();

    test_comp(rng);
}
