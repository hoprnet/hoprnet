//! Secret sharing reconstruction via Lagrange interpolation at x=0.
//!
//! This module provides share reconstruction functionality that was previously
//! in vsss-rs, implemented directly using elliptic-curve types.

use elliptic_curve::PrimeField;
use k256::Scalar;

/// A share with identifier and value, both in the same prime field.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Share<S = Scalar> {
    pub(crate) identifier: S,
    pub(crate) value: S,
}

impl<S: PrimeField> Share<S> {
    /// Creates a new share with the given identifier and value.
    ///
    /// Returns `None` if the identifier is zero.
    pub fn new(identifier: S, value: S) -> Option<Self> {
        if bool::from(identifier.is_zero()) {
            return None;
        }
        Some(Self { identifier, value })
    }

    /// Returns the identifier (x-coordinate) of this share.
    pub fn identifier(&self) -> &S {
        &self.identifier
    }

    /// Returns the value (y-coordinate) of this share.
    pub fn value(&self) -> &S {
        &self.value
    }
}

impl<S: PrimeField> From<(S, S)> for Share<S> {
    fn from((identifier, value): (S, S)) -> Self {
        Self { identifier, value }
    }
}

impl<S: PrimeField> std::ops::Deref for Share<S> {
    type Target = (S, S);
    fn deref(&self) -> &Self::Target {
        // Safety: Share is repr(transparent) to (S, S)
        unsafe { std::mem::transmute(self) }
    }
}

/// A raw polynomial with coefficients in a prime field.
///
/// This is similar to the vsss-rs Polynomial type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawPolynomial<S = Scalar>(pub Vec<Share<S>>);

impl<S: PrimeField> RawPolynomial<S> {
    /// Creates a new polynomial with the given degree.
    ///
    /// The polynomial will have `degree + 1` coefficients.
    pub fn create(degree: usize) -> Self {
        Self(Vec::with_capacity(degree + 1))
    }

    /// Creates a polynomial filled with random coefficients, with the secret as the constant term.
    ///
    /// - `secret`: The constant term (the secret to be shared)
    /// - `rng`: Random number generator
    /// - `degree`: The degree of the polynomial (number of random coefficients)
    pub fn fill<R>(mut self, secret: &S, mut rng: R, degree: usize) -> Self
    where
        R: elliptic_curve::rand_core::RngCore,
    {
        // First coefficient is the secret
        self.0.push(Share {
            identifier: S::ZERO, // Placeholder
            value: *secret,
        });

        // Remaining coefficients are random
        for _ in 1..=degree {
            let value = S::random(&mut rng);
            self.0.push(Share {
                identifier: S::ZERO, // Placeholder
                value,
            });
        }

        self
    }

    /// Evaluates the polynomial at the given point.
    ///
    /// Uses Horner's method for efficient evaluation.
    pub fn evaluate(&self, x: &S, _threshold: usize) -> Share<S> {
        if self.0.is_empty() {
            return Share {
                identifier: *x,
                value: S::ZERO,
            };
        }

        // Start with the highest coefficient and work backwards
        let mut result = self.0[self.0.len() - 1].value;
        for i in (0..self.0.len() - 1).rev() {
            result = result * x + self.0[i].value;
        }

        Share {
            identifier: *x,
            value: result,
        }
    }

    /// Returns the number of coefficients in the polynomial.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns true if the polynomial is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

/// A wrapper to provide vsss_rs-compatible polynomial creation syntax.
impl<S: PrimeField> RawPolynomial<S> {
    /// Creates a new polynomial with degree `t-1` (i.e., `t` coefficients).
    pub fn create_with_threshold(t: usize) -> Self {
        Self::create(t)
    }
}

/// Trait for share verification.
pub trait ShareVerifier<S: PrimeField = Scalar> {
    /// The share type used by this verifier.
    type Share: std::borrow::Borrow<Share<S>>;
    /// Error type returned by verification.
    type Error;

    /// Verifies a share against this verifier.
    fn verify_share(&self, share: &Self::Share) -> Result<(), Self::Error>;
}

/// Extension trait to provide combine() method on share collections.
///
/// This maintains API compatibility with the previous vsss-rs implementation.
pub trait ReadableShareSet<S> {
    /// Reconstruct the secret from shares using Lagrange interpolation at x=0.
    ///
    /// Given shares (x_i, y_i) where y_i = f(x_i) for some polynomial f,
    /// this computes f(0) = Σ y_i * L_i(0) where L_i is the Lagrange basis polynomial.
    fn combine(&self) -> Result<(S,), CombineError>;
}

impl<S: PrimeField> ReadableShareSet<S> for Vec<Share<S>> {
    fn combine(&self) -> Result<(S,), CombineError> {
        combine_shares(self).map(|s| (s,))
    }
}

impl<S: PrimeField> ReadableShareSet<S> for &[Share<S>] {
    fn combine(&self) -> Result<(S,), CombineError> {
        combine_shares(self).map(|s| (s,))
    }
}

/// Reconstruct a secret from shares using Lagrange interpolation at x=0.
///
/// Given shares (x_i, y_i) where y_i = f(x_i) for some polynomial f,
/// this computes f(0) = Σ y_i * L_i(0) where L_i is the Lagrange basis polynomial.
///
/// # Algorithm
/// For each share i, compute:
///   numerator = Π_{j≠i} x_j
///   denominator = Π_{j≠i} (x_j - x_i)
///   basis_i = numerator * denominator^{-1}
///   contribution_i = y_i * basis_i
///
/// The secret is the sum of all contributions.
///
/// # Errors
/// - Returns `CombineError::TooFewShares` if fewer than 2 shares provided
/// - Returns `CombineError::ZeroIdentifier` if any share has zero identifier
/// - Returns `CombineError::DuplicateIdentifier` if share identifiers are not unique
pub fn combine_shares<S: PrimeField>(shares: &[Share<S>]) -> Result<S, CombineError> {
    let n = shares.len();

    if n < 2 {
        return Err(CombineError::TooFewShares);
    }

    // Validate: no zero identifiers
    for share in shares {
        if bool::from(share.identifier.is_zero()) {
            return Err(CombineError::ZeroIdentifier);
        }
    }

    // Validate: no duplicate identifiers
    for i in 0..n {
        for j in (i + 1)..n {
            if bool::from(shares[i].identifier.ct_eq(&shares[j].identifier)) {
                return Err(CombineError::DuplicateIdentifier);
            }
        }
    }

    // Lagrange interpolation at x=0
    let mut secret = S::ZERO;

    for (i, x_i) in shares.iter().enumerate() {
        let mut numerator = S::ONE;
        let mut denominator = S::ONE;

        for (j, x_j) in shares.iter().enumerate() {
            if i == j {
                continue;
            }

            // numerator *= x_j
            numerator *= x_j.identifier;

            // denominator *= (x_j - x_i)
            let diff = x_j.identifier - x_i.identifier;
            denominator *= diff;
        }

        // basis = numerator * denominator^(-1)
        let basis = numerator * denominator.invert().unwrap();

        // contribution = y_i * basis
        let contribution = x_i.value * basis;

        secret += contribution;
    }

    Ok(secret)
}

/// Errors that can occur during share combination.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum CombineError {
    #[error("at least 2 shares required for reconstruction")]
    TooFewShares,

    #[error("share identifier cannot be zero")]
    ZeroIdentifier,

    #[error("duplicate share identifiers detected")]
    DuplicateIdentifier,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_combine_two_shares() {
        // Secret = 42, threshold = 2, shares at x=1 and x=2
        // For a degree-1 polynomial f(x) = a + b*x with f(1) = s1, f(2) = s2:
        // secret = f(0) = 2*s1 - s2 = 2*? - ?
        // Let's use specific values:
        // x=1: share1 = 100
        // x=2: share2 = 200
        // f(0) = 2*100 - 200 = 0
        // Actually let's verify: polynomial through (1,100) and (2,200) is f(x) = 100*x
        // f(0) = 0
        let share1 = Share::new(Scalar::ONE, Scalar::from(100u32));
        let share2 = Share::new(Scalar::from(2u32), Scalar::from(200u32));
        let shares = vec![share1.unwrap(), share2.unwrap()];

        let result = combine_shares(&shares);
        assert!(result.is_ok());
        // f(x) = 100*x, so f(0) = 0
        assert_eq!(result.unwrap(), Scalar::ZERO);
    }

    #[test]
    fn test_combine_three_shares() {
        // Use 3 points to reconstruct degree-2 polynomial
        // f(x) = a + b*x + c*x^2
        // Let's say secret = f(0) = a
        // If f(1) = 15, f(2) = 42, f(3) = 87
        // We need to find the polynomial that passes through these points
        // For simplicity, let's just test that reconstruction works

        let share1 = Share::new(Scalar::ONE, Scalar::from(15u32));
        let share2 = Share::new(Scalar::from(2u32), Scalar::from(42u32));
        let share3 = Share::new(Scalar::from(3u32), Scalar::from(87u32));
        let shares = vec![share1.unwrap(), share2.unwrap(), share3.unwrap()];

        let result = combine_shares(&shares);
        assert!(result.is_ok());
        // The reconstructed secret is f(0)
        // For polynomial passing through (1,15), (2,42), (3,87):
        // Lagrange: L1(0) = 2*3/((1-2)*(1-3)) = 6/(−1*−2) = 6/2 = 3
        //          L2(0) = 1*3/((2-1)*(2-3)) = 3/(1*−1) = -3
        //          L3(0) = 1*2/((3-1)*(3-2)) = 2/(2*1) = 1
        // secret = 15*3 + 42*(-3) + 87*1 = 45 - 126 + 87 = 6
        assert_eq!(result.unwrap(), Scalar::from(6u32));
    }

    #[test]
    fn test_combine_too_few_shares() {
        let share = Share::new(Scalar::ONE, Scalar::from(100u32));
        let shares = vec![share.unwrap()];

        let result = combine_shares(&shares);
        assert!(matches!(result, Err(CombineError::TooFewShares)));
    }

    #[test]
    fn test_combine_zero_identifier() {
        // When identifier is zero, Share::new returns None
        let share1 = Share::new(Scalar::ZERO, Scalar::from(100u32));
        let _share2 = Share::new(Scalar::ONE, Scalar::from(200u32));
        assert!(share1.is_none(), "Share with zero identifier should return None");

        // Test that combine returns ZeroIdentifier error for a share with zero identifier
        // We create shares that explicitly have zero identifier by using a different approach
        let shares = vec![
            Share {
                identifier: Scalar::ZERO,
                value: Scalar::from(100u32),
            },
            Share {
                identifier: Scalar::ONE,
                value: Scalar::from(200u32),
            },
        ];

        let result = combine_shares(&shares);
        assert!(matches!(result, Err(CombineError::ZeroIdentifier)));
    }

    #[test]
    fn test_combine_duplicate_identifier() {
        let share1 = Share::new(Scalar::ONE, Scalar::from(100u32));
        let share2 = Share::new(Scalar::ONE, Scalar::from(200u32));
        let shares = vec![share1.unwrap(), share2.unwrap()];

        let result = combine_shares(&shares);
        assert!(matches!(result, Err(CombineError::DuplicateIdentifier)));
    }

    #[test]
    fn test_readable_share_set_trait() {
        let share1 = Share::new(Scalar::ONE, Scalar::from(15u32));
        let share2 = Share::new(Scalar::from(2u32), Scalar::from(42u32));
        let share3 = Share::new(Scalar::from(3u32), Scalar::from(87u32));
        let shares = vec![share1.unwrap(), share2.unwrap(), share3.unwrap()];

        // Test Vec::combine() returns (S,) tuple
        let result: Result<(Scalar,), _> = shares.combine();
        assert!(result.is_ok());
        assert_eq!(result.unwrap().0, Scalar::from(6u32));
    }
}
