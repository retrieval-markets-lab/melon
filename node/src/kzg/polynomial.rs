use blstrs::Scalar;
use pairing::group::ff::Field;
use std::cmp::{Eq, PartialEq};
use std::iter::Iterator;
use std::ops::{Add, Mul, MulAssign, Sub};

#[derive(Clone, Debug)]
pub struct Polynomial {
    pub degree: usize,
    pub coeffs: Vec<Scalar>,
}

impl PartialEq<Polynomial> for Polynomial {
    fn eq(&self, other: &Self) -> bool {
        if self.degree() != other.degree() {
            false
        } else {
            self.coeffs
                .iter()
                .zip(other.coeffs.iter())
                .all(|(l, r)| l == r)
        }
    }
}

impl Eq for Polynomial {}

impl Polynomial {
    pub fn is_zero(&self) -> bool {
        self.degree() == 0 && self.coeffs[0] == Scalar::zero()
    }

    pub fn new_zero() -> Polynomial {
        Polynomial {
            degree: 0,
            coeffs: vec![Scalar::zero()],
        }
    }

    pub fn from_scalar(scalar: Scalar) -> Polynomial {
        Polynomial {
            degree: 0,
            coeffs: vec![scalar],
        }
    }

    pub fn new_monic_of_degree(degree: usize) -> Polynomial {
        Polynomial {
            degree,
            coeffs: vec![Scalar::one(); degree + 1],
        }
    }

    pub fn new_zero_with_size(cap: usize) -> Polynomial {
        Polynomial {
            degree: 0,
            coeffs: vec![Scalar::zero(); cap],
        }
    }

    pub fn new(coeffs: Vec<Scalar>) -> Polynomial {
        // figure out what the initial degree is
        let degree = Self::compute_degree(&coeffs, coeffs.len() - 1);
        Polynomial { degree, coeffs }
    }

    /// note: use this carefully, as setting the degree incorrect can lead to the degree being inconsistent
    pub fn new_from_coeffs(coeffs: Vec<Scalar>, degree: usize) -> Polynomial {
        Polynomial { degree, coeffs }
    }

    pub fn compute_degree(coeffs: &[Scalar], upper_bound: usize) -> usize {
        let mut i = upper_bound;
        loop {
            if i == 0 {
                break 0;
            } else if coeffs[i] != Scalar::zero() {
                break i;
            }

            i -= 1;
        }
    }

    pub fn shrink_degree(&mut self) {
        let degree = Self::compute_degree(&self.coeffs, self.degree);
        self.degree = degree;
    }

    pub fn lead(&self) -> Scalar {
        self.coeffs[self.degree]
    }

    pub fn num_coeffs(&self) -> usize {
        self.degree + 1
    }

    pub fn degree(&self) -> usize {
        self.degree
    }

    pub fn coeffs(mut self) -> Vec<Scalar> {
        self.coeffs.truncate(self.num_coeffs());
        self.coeffs
    }

    pub fn slice_coeffs(&self) -> &[Scalar] {
        &self.coeffs[..self.num_coeffs()]
    }

    pub fn iter_coeffs(&self) -> impl Iterator<Item = &Scalar> {
        self.coeffs.iter().take(self.num_coeffs())
    }

    pub fn eval(&self, x: Scalar) -> Scalar {
        let mut res = self.coeffs[self.degree()];

        for i in (0..self.degree()).rev() {
            res *= x;
            res += self.coeffs[i];
        }

        res
    }

    pub fn lagrange_interpolation(xs: &[Scalar], ys: &[Scalar]) -> Polynomial {
        assert_eq!(xs.len(), ys.len());

        // Interpolates on the first `i` samples.
        let mut poly = Polynomial::new_from_coeffs(vec![ys[0]], 0);
        // Is zero on the first `i` samples.
        let mut base = Polynomial::new_from_coeffs(vec![-xs[0], Scalar::one()], 1);

        // We update `base` so that it is always zero on all previous samples, and `poly` so that
        // it has the correct values on the previous samples.
        for (x, y) in xs[1..].iter().zip(ys[1..].iter()) {
            // Scale `base` so that its value at `x` is the difference between `y` and `poly`'s
            // current value at `x`: Adding it to `poly` will then make it correct for `x`.
            let diff = (*y - poly.eval(*x)) * base.eval(*x).invert().unwrap();
            base = base * &diff;
            poly = poly + base.clone();

            // Finally, multiply `base` by X - x, so that it is zero at `x`, too, now.
            base = base * Polynomial::new_from_coeffs(vec![-(*x), Scalar::one()], 1);
        }
        poly
    }
}

impl Add for Polynomial {
    type Output = Polynomial;

    fn add(self, rhs: Self) -> Self::Output {
        let (mut res, shorter) = if rhs.degree() > self.degree() {
            (rhs, self)
        } else {
            (self, rhs)
        };

        for i in 0..shorter.num_coeffs() {
            res.coeffs[i] += shorter.coeffs[i];
        }

        res
    }
}

impl<'a> Mul<&'a Scalar> for Polynomial {
    type Output = Polynomial;

    fn mul(mut self, rhs: &Scalar) -> Self::Output {
        if rhs.is_zero().unwrap_u8() == 1 {
            return Polynomial::new_zero();
        } else {
            self.coeffs.iter_mut().for_each(|c| c.mul_assign(rhs));
        }
        self
    }
}

impl<'a> Sub for &'a Polynomial {
    type Output = Polynomial;

    fn sub(self, rhs: Self) -> Self::Output {
        let mut res = self.clone();
        if rhs.num_coeffs() > self.num_coeffs() {
            res.coeffs.resize(rhs.num_coeffs(), Scalar::zero());
            res.degree = rhs.degree();
        }

        for i in 0..rhs.num_coeffs() {
            res.coeffs[i] -= rhs.coeffs[i];
        }

        res.shrink_degree();
        res
    }
}

impl Mul<Polynomial> for Polynomial {
    type Output = Polynomial;

    fn mul(self, rhs: Self) -> Self::Output {
        let mut res = Polynomial::new_zero_with_size(self.degree() + rhs.degree() + 1);
        for i in 0..self.num_coeffs() {
            for j in 0..rhs.num_coeffs() {
                res.coeffs[i + j] += self.coeffs[i] * rhs.coeffs[j];
            }
        }

        res.degree = self.degree() + rhs.degree();
        res
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use blstrs::Scalar;

    #[test]
    fn test_eval_basic() {
        // y(x) = x^5 + 4x^3 + 7x^2 + 34
        let polynomial = Polynomial::new(vec![
            34.into(),
            Scalar::zero(),
            7.into(),
            4.into(),
            Scalar::zero(),
            Scalar::one(),
        ]);

        // y(0) = 34
        assert_eq!(polynomial.eval(Scalar::zero()), 34.into());
        // y(1) = 46
        assert_eq!(polynomial.eval(Scalar::one()), 46.into());
        // y(5) = 3834
        assert_eq!(polynomial.eval(5.into()), 3834.into());
    }

    #[test]
    fn test_interpolation() {
        let xs: Vec<Scalar> = vec![2].into_iter().map(|x| x.into()).collect();
        let ys: Vec<Scalar> = vec![8].into_iter().map(|x| x.into()).collect();

        let interpolation = Polynomial::lagrange_interpolation(xs.as_slice(), ys.as_slice());

        for (&x, &y) in xs.iter().zip(ys.iter()) {
            assert_eq!(interpolation.eval(x), y);
        }

        let xs: Vec<Scalar> = vec![2, 5, 7, 90, 111, 31, 29]
            .into_iter()
            .map(|x| x.into())
            .collect();
        let ys: Vec<Scalar> = vec![8, 1, 43, 2, 87, 122, 13]
            .into_iter()
            .map(|x| x.into())
            .collect();
        let interpolation = Polynomial::lagrange_interpolation(xs.as_slice(), ys.as_slice());

        for (&x, &y) in xs.iter().zip(ys.iter()) {
            assert_eq!(interpolation.eval(x), y);
        }
    }
}
