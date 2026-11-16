//! Cursed structural counters.
//!
//! `Nat` is a [`BigNum`] used only for structural quantities: lengths, offsets,
//! exponents, padding, and loop counters. Its arithmetic is the same
//! table-driven arithmetic as everything else.

use crate::big_num::BigNum;

/// A non-negative structural count.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub(crate) struct Nat(BigNum);

impl Nat {
    /// Zero.
    pub(crate) fn zero() -> Self {
        Self(BigNum::zero())
    }

    /// Two.
    pub(crate) fn two() -> Self {
        Self(BigNum::from(2u8))
    }

    /// Whether this count is zero.
    pub(crate) fn is_zero(&self) -> bool {
        self.0.is_zero()
    }

    /// The next count.
    pub(crate) fn successor(&self) -> Self {
        Self(&self.0 + &BigNum::one())
    }

    /// The previous count. Panics if this count is zero.
    pub(crate) fn predecessor(&self) -> Self {
        Self(
            self.0
                .checked_sub(&BigNum::one())
                .expect("predecessor of zero"),
        )
    }

    /// Subtraction, saturating at zero.
    pub(crate) fn saturating_sub(&self, other: &Self) -> Self {
        if self < other {
            Self::zero()
        } else {
            Self(
                self.0
                    .checked_sub(&other.0)
                    .expect("self is at least other"),
            )
        }
    }

    /// Quotient and remainder.
    pub(crate) fn div_rem(&self, other: &Self) -> (Self, Self) {
        let (quotient, remainder) = self.0.div_rem(&other.0);
        (Self(quotient), Self(remainder))
    }

    /// Converts a `usize` through its decimal representation.
    pub(crate) fn from_usize(value: usize) -> Self {
        Self(
            value
                .to_string()
                .parse()
                .expect("every usize has a decimal representation"),
        )
    }
}
