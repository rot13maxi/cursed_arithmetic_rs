//! The arbitrary-precision decimal number type.

use crate::digit::Digit;
use std::cmp::Ordering;
use std::fmt::{self, Write as _};
use std::str::FromStr;

/// An arbitrary-precision non-negative integer stored as a big-endian
/// `Vec<Digit>`.
///
/// The representation is normalized: there are no leading zeroes, and zero is
/// represented as `[Digit::Zero]`.
///
/// ```
/// use cursed_arithmetic::{BigNum, Digit};
///
/// let n = BigNum::from_digits([Digit::One, Digit::Two, Digit::Three]);
/// assert_eq!(n.to_string(), "123");
/// assert_eq!(n.digits(), &[Digit::One, Digit::Two, Digit::Three]);
/// ```
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct BigNum(Vec<Digit>);

pub(crate) fn normalize_vec(digits: Vec<Digit>) -> Vec<Digit> {
    let first_nonzero = digits.iter().position(|digit| *digit != Digit::Zero);
    match first_nonzero {
        Some(index) => {
            let mut digits = digits;
            digits.drain(..index);
            digits
        }
        None => vec![Digit::Zero],
    }
}

pub(crate) fn trimmed(digits: &[Digit]) -> &[Digit] {
    let start = digits
        .iter()
        .position(|digit| *digit != Digit::Zero)
        .unwrap_or(digits.len());
    &digits[start..]
}

pub(crate) fn cmp_digits(lhs: &[Digit], rhs: &[Digit]) -> Ordering {
    let lhs = trimmed(lhs);
    let rhs = trimmed(rhs);
    lhs.len().cmp(&rhs.len()).then_with(|| lhs.cmp(rhs))
}

/// One-bit addition, encoded as a lookup table.
const fn full_adder(lhs: bool, rhs: bool, carry: bool) -> (bool, bool) {
    match (lhs, rhs, carry) {
        (false, false, false) => (false, false),
        (false, false, true) => (true, false),
        (false, true, false) => (true, false),
        (false, true, true) => (false, true),
        (true, false, false) => (true, false),
        (true, false, true) => (false, true),
        (true, true, false) => (false, true),
        (true, true, true) => (true, true),
    }
}

/// Adds two `u128`s one bit at a time, without using primitive addition.
fn add_u128(lhs: u128, rhs: u128) -> (u128, bool) {
    let mut result = 0u128;
    let mut carry = false;

    for bit in 0..128 {
        let lhs_bit = (lhs >> bit) & 1 == 1;
        let rhs_bit = (rhs >> bit) & 1 == 1;
        let (sum, next_carry) = full_adder(lhs_bit, rhs_bit, carry);
        if sum {
            result |= 1u128 << bit;
        }
        carry = next_carry;
    }

    (result, carry)
}

/// Multiplies a `u128` by ten using shifts and the bitwise adder.
fn mul10(value: u128) -> Option<u128> {
    if value >> 125 != 0 {
        return None;
    }

    let (result, carry) = add_u128(value << 3, value << 1);
    if carry { None } else { Some(result) }
}

impl BigNum {
    /// Zero.
    pub fn zero() -> Self {
        Self(vec![Digit::Zero])
    }

    /// One.
    pub fn one() -> Self {
        Self(vec![Digit::One])
    }

    /// Builds a number from decimal digits, most significant first.
    ///
    /// Leading zeroes are discarded. An empty iterator produces zero.
    pub fn from_digits(digits: impl IntoIterator<Item = Digit>) -> Self {
        Self(normalize_vec(digits.into_iter().collect()))
    }

    /// Builds a number from a `u128`.
    ///
    /// The conversion walks the primitive's bits from most significant to
    /// least significant, doubling the decimal digit vector and adding each
    /// bit. All decimal operations use the digit lookup tables.
    pub fn from_u128(value: u128) -> Self {
        let mut digits = vec![Digit::Zero];

        for shift in (0..128).rev() {
            let mut carry = false;
            for digit in &mut digits {
                let (sum, next_carry) = digit.carrying_add(*digit, carry);
                *digit = sum;
                carry = next_carry;
            }
            if carry {
                digits.push(Digit::One);
            }

            if (value >> shift) & 1 == 1 {
                let mut carry = true;
                for digit in &mut digits {
                    let (sum, next_carry) = digit.carrying_add(Digit::Zero, carry);
                    *digit = sum;
                    carry = next_carry;
                    if !carry {
                        break;
                    }
                }
                if carry {
                    digits.push(Digit::One);
                }
            }
        }

        digits.reverse();
        Self(normalize_vec(digits))
    }

    /// The digits, most significant first.
    pub fn digits(&self) -> &[Digit] {
        &self.0
    }

    /// Consumes the number and returns its digits, most significant first.
    pub fn into_digits(self) -> Vec<Digit> {
        self.0
    }

    /// Whether this number is zero.
    pub fn is_zero(&self) -> bool {
        self.0 == [Digit::Zero]
    }

    /// Converts to a `u128`, if the value fits.
    ///
    /// The conversion uses shifts and a bit-level full adder; it does not use
    /// primitive multiplication or addition.
    pub fn to_u128(&self) -> Option<u128> {
        let mut value = 0u128;
        for digit in &self.0 {
            value = mul10(value)?;
            let (sum, carry) = add_u128(value, u128::from(digit.value()));
            if carry {
                return None;
            }
            value = sum;
        }
        Some(value)
    }

    /// Converts to a `u64`, if the value fits.
    pub fn to_u64(&self) -> Option<u64> {
        self.to_u128()?.try_into().ok()
    }
}

impl fmt::Display for BigNum {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let width = f.width().unwrap_or(0);
        let padding = width.saturating_sub(self.0.len());

        if f.sign_aware_zero_pad() {
            for _ in 0..padding {
                f.write_char('0')?;
            }
            for digit in &self.0 {
                f.write_char(digit.to_char())?;
            }
            return Ok(());
        }

        let (before, after) = match f.align().unwrap_or(fmt::Alignment::Right) {
            fmt::Alignment::Left => (0, padding),
            fmt::Alignment::Right => (padding, 0),
            fmt::Alignment::Center => (padding / 2, padding - padding / 2),
        };

        let fill = f.fill();
        for _ in 0..before {
            f.write_char(fill)?;
        }
        for digit in &self.0 {
            f.write_char(digit.to_char())?;
        }
        for _ in 0..after {
            f.write_char(fill)?;
        }
        Ok(())
    }
}

impl fmt::Debug for BigNum {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "BigNum({self})")
    }
}

/// The error returned when parsing a [`BigNum`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ParseBigNumError {
    /// The input contained no digits.
    Empty,
    /// The input contained a character that is not an ASCII decimal digit.
    InvalidCharacter(char),
}

impl fmt::Display for ParseBigNumError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => f.write_str("empty string is not a number"),
            Self::InvalidCharacter(c) => {
                write!(f, "invalid character {c:?} in decimal number")
            }
        }
    }
}

impl std::error::Error for ParseBigNumError {}

impl FromStr for BigNum {
    type Err = ParseBigNumError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.strip_prefix('+').unwrap_or(s);
        if s.is_empty() {
            return Err(ParseBigNumError::Empty);
        }

        let mut digits = Vec::with_capacity(s.len());
        for c in s.chars() {
            let digit = Digit::from_char(c).ok_or(ParseBigNumError::InvalidCharacter(c))?;
            digits.push(digit);
        }
        Ok(Self(normalize_vec(digits)))
    }
}

impl Ord for BigNum {
    fn cmp(&self, other: &Self) -> Ordering {
        cmp_digits(&self.0, &other.0)
    }
}

impl PartialOrd for BigNum {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl From<Digit> for BigNum {
    fn from(digit: Digit) -> Self {
        Self(vec![digit])
    }
}

impl From<Vec<Digit>> for BigNum {
    fn from(digits: Vec<Digit>) -> Self {
        Self::from_digits(digits)
    }
}

impl From<BigNum> for Vec<Digit> {
    fn from(number: BigNum) -> Self {
        number.into_digits()
    }
}

impl Default for BigNum {
    fn default() -> Self {
        Self::zero()
    }
}

macro_rules! impl_from_unsigned {
    ($($ty:ty),* $(,)?) => {
        $(
            impl From<$ty> for BigNum {
                fn from(value: $ty) -> Self {
                    Self::from_u128(value as u128)
                }
            }
        )*
    };
}

impl_from_unsigned!(u8, u16, u32, u64, u128, usize);

impl FromIterator<Digit> for BigNum {
    fn from_iter<I: IntoIterator<Item = Digit>>(iter: I) -> Self {
        Self::from_digits(iter)
    }
}

impl Extend<Digit> for BigNum {
    fn extend<I: IntoIterator<Item = Digit>>(&mut self, iter: I) {
        self.0.extend(iter);
        self.0 = normalize_vec(std::mem::take(&mut self.0));
    }
}

impl IntoIterator for BigNum {
    type Item = Digit;
    type IntoIter = std::vec::IntoIter<Digit>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> IntoIterator for &'a BigNum {
    type Item = &'a Digit;
    type IntoIter = std::slice::Iter<'a, Digit>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl AsRef<[Digit]> for BigNum {
    fn as_ref(&self) -> &[Digit] {
        &self.0
    }
}

impl std::ops::Deref for BigNum {
    type Target = [Digit];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalization_removes_leading_zeroes() {
        assert_eq!(
            BigNum::from_digits([Digit::Zero, Digit::Zero, Digit::One, Digit::Two]).digits(),
            [Digit::One, Digit::Two]
        );
        assert_eq!(BigNum::from_digits([]).digits(), [Digit::Zero]);
        assert_eq!(
            BigNum::from_digits([Digit::Zero, Digit::Zero]).digits(),
            [Digit::Zero]
        );
    }

    #[test]
    fn parsing_and_display_round_trip() {
        let cases = [
            ("0", "0"),
            ("1", "1"),
            ("9", "9"),
            ("10", "10"),
            (
                "123456789012345678901234567890",
                "123456789012345678901234567890",
            ),
            ("00042", "42"),
            ("+007", "7"),
        ];
        for (input, expected) in cases {
            let n: BigNum = input.parse().unwrap();
            assert_eq!(n.to_string(), expected);
        }
        assert_eq!("".parse::<BigNum>(), Err(ParseBigNumError::Empty));
        assert_eq!("+".parse::<BigNum>(), Err(ParseBigNumError::Empty));
        assert_eq!(
            "12a".parse::<BigNum>(),
            Err(ParseBigNumError::InvalidCharacter('a'))
        );
        assert_eq!(
            "-1".parse::<BigNum>(),
            Err(ParseBigNumError::InvalidCharacter('-'))
        );
    }

    #[test]
    fn conversion_round_trips() {
        for value in [0u128, 1, 9, 10, 12345, u128::MAX] {
            assert_eq!(BigNum::from_u128(value).to_u128(), Some(value));
        }
        assert_eq!(BigNum::from_u128(u128::MAX).to_u64(), None);
        assert_eq!(BigNum::from(42u32).to_u64(), Some(42));
    }

    #[test]
    fn ordering_is_numeric() {
        let nums: Vec<BigNum> = ["0", "1", "2", "9", "10", "11", "99", "100", "1000"]
            .into_iter()
            .map(|s| s.parse().unwrap())
            .collect();
        for pair in nums.windows(2) {
            assert!(pair[0] < pair[1]);
        }
        assert_eq!("0007".parse::<BigNum>().unwrap(), "7".parse().unwrap());
    }

    #[test]
    fn display_respects_width_alignment_and_zero_padding() {
        let n = BigNum::from(42u32);
        assert_eq!(format!("{n:>5}"), "   42");
        assert_eq!(format!("{n:<5}"), "42   ");
        assert_eq!(format!("{n:^5}"), " 42  ");
        assert_eq!(format!("{n:05}"), "00042");
        assert_eq!(format!("{n:x>5}"), "xxx42");
        assert_eq!(format!("{:>5}", BigNum::zero()), "    0");
    }

    #[test]
    fn iterators_and_extend_work() {
        let mut n: BigNum = "12".parse().unwrap();
        n.extend([Digit::Three, Digit::Four]);
        assert_eq!(n.to_string(), "1234");

        let collected: BigNum = "987".parse::<BigNum>().unwrap().into_iter().collect();
        assert_eq!(collected.to_string(), "987");
        assert_eq!(
            "987"
                .parse::<BigNum>()
                .unwrap()
                .into_iter()
                .collect::<Vec<_>>(),
            [Digit::Nine, Digit::Eight, Digit::Seven]
        );
        assert_eq!(
            "12".parse::<BigNum>().unwrap().as_ref(),
            [Digit::One, Digit::Two]
        );

        let n: BigNum = "123".parse().unwrap();
        assert_eq!(n.len(), 3);
        assert_eq!(n[0], Digit::One);
        assert_eq!(n.last(), Some(&Digit::Three));
    }
}
