//! The arbitrary-precision decimal number type.
//!
//! Digits live in a singly linked list, most significant first. There are no
//! slices, no indices, and no length arithmetic: every traversal is a pointer
//! walk, and every structural count is a cursed [`Nat`].

use crate::digit::Digit;
use crate::nat::Nat;
use std::cmp::Ordering;
use std::fmt::{self, Write as _};
use std::str::FromStr;

/// An arbitrary-precision non-negative integer stored as a linked list of
/// decimal digits, most significant first.
///
/// The representation is normalized: there are no leading zeroes, and zero is
/// a single [`Digit::Zero`] node.
///
/// ```
/// use cursed_arithmetic::{BigNum, Digit};
///
/// let n = BigNum::from_digits([Digit::One, Digit::Two, Digit::Three]);
/// assert_eq!(n.to_string(), "123");
/// ```
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct BigNum(Option<Box<Node>>);

#[derive(Clone, PartialEq, Eq, Hash)]
struct Node {
    digit: Digit,
    next: Option<Box<Node>>,
}

fn cons(digit: Digit, next: Option<Box<Node>>) -> Option<Box<Node>> {
    Some(Box::new(Node { digit, next }))
}

fn zero_cons() -> Option<Box<Node>> {
    cons(Digit::Zero, None)
}

fn is_zero_list(head: &Option<Box<Node>>) -> bool {
    match head {
        Some(node) => node.digit == Digit::Zero && node.next.is_none(),
        None => true,
    }
}

fn normalize(head: Option<Box<Node>>) -> BigNum {
    let mut head = head;
    loop {
        match head {
            Some(node) if node.digit == Digit::Zero && node.next.is_some() => {
                let Node { next, .. } = *node;
                head = next;
            }
            Some(node) => return BigNum(Some(node)),
            None => return BigNum(zero_cons()),
        }
    }
}

fn reverse(mut head: Option<Box<Node>>) -> Option<Box<Node>> {
    let mut previous = None;
    while let Some(mut node) = head {
        head = node.next.take();
        node.next = previous;
        previous = Some(node);
    }
    previous
}

fn append(head: Option<Box<Node>>, digit: Digit) -> Option<Box<Node>> {
    match head {
        None => cons(digit, None),
        Some(mut head) => {
            let mut cursor: &mut Node = &mut head;
            loop {
                match cursor.next {
                    Some(ref mut next) => cursor = next,
                    None => {
                        cursor.next = cons(digit, None);
                        break;
                    }
                }
            }
            Some(head)
        }
    }
}

fn append_list(head: Option<Box<Node>>, tail: Option<Box<Node>>) -> Option<Box<Node>> {
    match head {
        None => tail,
        Some(mut head) => {
            let mut cursor: &mut Node = &mut head;
            loop {
                match cursor.next {
                    Some(ref mut next) => cursor = next,
                    None => {
                        cursor.next = tail;
                        break;
                    }
                }
            }
            Some(head)
        }
    }
}

fn cmp_lists(lhs: &Option<Box<Node>>, rhs: &Option<Box<Node>>) -> Ordering {
    let mut lhs = lhs.as_deref();
    let mut rhs = rhs.as_deref();
    let mut ordering = Ordering::Equal;

    loop {
        match (lhs, rhs) {
            (None, None) => return ordering,
            (None, Some(_)) => return Ordering::Less,
            (Some(_), None) => return Ordering::Greater,
            (Some(lhs_node), Some(rhs_node)) => {
                if ordering == Ordering::Equal {
                    ordering = lhs_node.digit.cmp(&rhs_node.digit);
                }
                lhs = lhs_node.next.as_deref();
                rhs = rhs_node.next.as_deref();
            }
        }
    }
}

fn add_lsd(mut lhs: Option<Box<Node>>, mut rhs: Option<Box<Node>>) -> Option<Box<Node>> {
    let mut carry = false;
    let mut out = None;

    loop {
        if lhs.is_none() && rhs.is_none() && !carry {
            break;
        }

        let (lhs_digit, lhs_next) = match lhs {
            Some(node) => {
                let Node { digit, next } = *node;
                (digit, next)
            }
            None => (Digit::Zero, None),
        };
        lhs = lhs_next;

        let (rhs_digit, rhs_next) = match rhs {
            Some(node) => {
                let Node { digit, next } = *node;
                (digit, next)
            }
            None => (Digit::Zero, None),
        };
        rhs = rhs_next;

        let (sum, next_carry) = lhs_digit.carrying_add(rhs_digit, carry);
        out = cons(sum, out);
        carry = next_carry;
    }

    out
}

pub(crate) fn add_owned(lhs: BigNum, rhs: BigNum) -> BigNum {
    let sum = add_lsd(reverse(lhs.0), reverse(rhs.0));
    normalize(sum)
}

fn sub_lsd(mut lhs: Option<Box<Node>>, mut rhs: Option<Box<Node>>) -> Option<Option<Box<Node>>> {
    let mut borrow = false;
    let mut out = None;

    loop {
        if lhs.is_none() && rhs.is_none() && !borrow {
            break;
        }

        let (lhs_digit, lhs_next) = match lhs {
            Some(node) => {
                let Node { digit, next } = *node;
                (digit, next)
            }
            None => (Digit::Zero, None),
        };
        lhs = lhs_next;

        let (rhs_digit, rhs_next) = match rhs {
            Some(node) => {
                let Node { digit, next } = *node;
                (digit, next)
            }
            None => (Digit::Zero, None),
        };
        rhs = rhs_next;

        let (difference, next_borrow) = lhs_digit.borrowing_sub(rhs_digit, borrow);
        out = cons(difference, out);
        borrow = next_borrow;
    }

    if borrow { None } else { Some(out) }
}

pub(crate) fn sub_owned(lhs: BigNum, rhs: BigNum) -> Option<BigNum> {
    if cmp_lists(&lhs.0, &rhs.0) == Ordering::Less {
        return None;
    }

    let difference = sub_lsd(reverse(lhs.0), reverse(rhs.0))?;
    Some(normalize(difference))
}

fn mul_by_digit(lhs: &Option<Box<Node>>, digit: Digit) -> Option<Box<Node>> {
    let mut cursor = lhs.as_deref();
    let mut out = None;
    let mut carry = Digit::Zero;

    while let Some(node) = cursor {
        let (units, mul_carry) = node.digit.carrying_mul(digit);
        let (sum, add_carry) = units.carrying_add(carry, false);
        let add_digit = if add_carry { Digit::One } else { Digit::Zero };
        let (next_carry, overflow) = mul_carry.carrying_add(add_digit, false);
        debug_assert!(!overflow);
        out = cons(sum, out);
        carry = next_carry;
        cursor = node.next.as_deref();
    }

    cons(carry, out)
}

fn append_zeros(head: Option<Box<Node>>, count: &Nat) -> Option<Box<Node>> {
    let mut zeros = None;
    let mut remaining = count.clone();
    while !remaining.is_zero() {
        zeros = cons(Digit::Zero, zeros);
        remaining = remaining.predecessor();
    }
    append_list(head, zeros)
}

pub(crate) fn mul_owned(lhs: BigNum, rhs: BigNum) -> BigNum {
    if lhs.is_zero() || rhs.is_zero() {
        return BigNum::zero();
    }

    let lhs_lsd = reverse(lhs.0);
    let mut rhs_lsd = reverse(rhs.0);
    let mut accumulator = zero_cons();
    let mut shift = Nat::zero();

    while let Some(node) = rhs_lsd {
        let Node { digit, next } = *node;
        rhs_lsd = next;

        let partial = mul_by_digit(&lhs_lsd, digit);
        let shifted = append_zeros(partial, &shift);
        accumulator = add_lsd(reverse(accumulator), reverse(shifted));
        shift = shift.successor();
    }

    normalize(accumulator)
}

pub(crate) fn div_rem_owned(dividend: BigNum, divisor: BigNum) -> (BigNum, BigNum) {
    assert!(!divisor.is_zero(), "attempt to divide by zero");

    let mut quotient = None;
    let mut remainder = zero_cons();
    let mut cursor = dividend.0.as_deref();

    while let Some(node) = cursor {
        let digit = node.digit;
        cursor = node.next.as_deref();

        if is_zero_list(&remainder) {
            remainder = cons(digit, None);
        } else {
            remainder = append(remainder, digit);
        }

        let mut quotient_digit = Digit::Zero;
        while cmp_lists(&remainder, &divisor.0) != Ordering::Less {
            let reduced = sub_owned(BigNum(remainder.clone()), divisor.clone())
                .expect("the remainder is at least the divisor");
            remainder = reduced.0;
            quotient_digit = quotient_digit.successor();
        }

        quotient = append(quotient, quotient_digit);
    }

    (normalize(quotient), normalize(remainder))
}

pub(crate) fn checked_sub(lhs: &BigNum, rhs: &BigNum) -> Option<BigNum> {
    sub_owned(lhs.clone(), rhs.clone())
}

pub(crate) fn div_rem(lhs: &BigNum, rhs: &BigNum) -> (BigNum, BigNum) {
    div_rem_owned(lhs.clone(), rhs.clone())
}

impl BigNum {
    /// Zero.
    pub fn zero() -> Self {
        Self(zero_cons())
    }

    /// One.
    pub fn one() -> Self {
        Self(cons(Digit::One, None))
    }

    /// Builds a number from decimal digits, most significant first.
    ///
    /// Leading zeroes are discarded. An empty iterator produces zero.
    pub fn from_digits(digits: impl IntoIterator<Item = Digit>) -> Self {
        let mut head = None;
        for digit in digits {
            head = cons(digit, head);
        }
        normalize(reverse(head))
    }

    /// Builds a number from a `u128` through its decimal representation.
    pub fn from_u128(value: u128) -> Self {
        value
            .to_string()
            .parse()
            .expect("every u128 has a decimal representation")
    }

    /// The digits, most significant first.
    pub fn digits(&self) -> Digits<'_> {
        Digits {
            next: self.0.as_deref(),
        }
    }

    /// Consumes the number and returns its digits, most significant first.
    pub fn into_digits(self) -> Vec<Digit> {
        self.into_iter().collect()
    }

    /// Whether this number is zero.
    pub fn is_zero(&self) -> bool {
        is_zero_list(&self.0)
    }

    /// Converts to a `u128`, if the value fits.
    pub fn to_u128(&self) -> Option<u128> {
        self.to_string().parse().ok()
    }

    /// Converts to a `u64`, if the value fits.
    pub fn to_u64(&self) -> Option<u64> {
        self.to_string().parse().ok()
    }

    pub(crate) fn last_digit(&self) -> Option<Digit> {
        let mut cursor = self.0.as_deref();
        let mut last = None;
        while let Some(node) = cursor {
            last = Some(node.digit);
            cursor = node.next.as_deref();
        }
        last
    }

    fn len_nat(&self) -> Nat {
        let mut count = Nat::zero();
        let mut cursor = self.0.as_deref();
        while let Some(node) = cursor {
            count = count.successor();
            cursor = node.next.as_deref();
        }
        count
    }

    pub(crate) fn mul_pow10_nat(&self, count: &Nat) -> Self {
        if self.is_zero() || count.is_zero() {
            return self.clone();
        }

        let mut zeros = None;
        let mut remaining = count.clone();
        while !remaining.is_zero() {
            zeros = cons(Digit::Zero, zeros);
            remaining = remaining.predecessor();
        }

        Self(append_list(self.0.clone(), zeros))
    }

    pub(crate) fn div_pow10_nat(&self, count: &Nat) -> Self {
        let mut head = reverse(self.0.clone());
        let mut remaining = count.clone();

        while !remaining.is_zero() {
            match head {
                Some(node) => {
                    let Node { next, .. } = *node;
                    head = next;
                }
                None => return Self::zero(),
            }
            remaining = remaining.predecessor();
        }

        normalize(reverse(head))
    }
}

/// A borrowing iterator over a [`BigNum`]'s digits, most significant first.
#[derive(Clone, Copy)]
pub struct Digits<'a> {
    next: Option<&'a Node>,
}

impl Iterator for Digits<'_> {
    type Item = Digit;

    fn next(&mut self) -> Option<Digit> {
        let node = self.next?;
        self.next = node.next.as_deref();
        Some(node.digit)
    }
}

/// An owning iterator over a [`BigNum`]'s digits, most significant first.
#[derive(Clone)]
pub struct IntoDigits {
    next: Option<Box<Node>>,
}

impl Iterator for IntoDigits {
    type Item = Digit;

    fn next(&mut self) -> Option<Digit> {
        let node = self.next.take()?;
        let Node { digit, next } = *node;
        self.next = next;
        Some(digit)
    }
}

fn write_repeated(f: &mut fmt::Formatter<'_>, character: char, count: &Nat) -> fmt::Result {
    let mut remaining = count.clone();
    while !remaining.is_zero() {
        f.write_char(character)?;
        remaining = remaining.predecessor();
    }
    Ok(())
}

impl fmt::Display for BigNum {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let width = f.width().map(Nat::from_usize).unwrap_or_else(Nat::zero);
        let length = self.len_nat();
        let padding = width.saturating_sub(&length);

        if f.sign_aware_zero_pad() {
            write_repeated(f, '0', &padding)?;
            for digit in self.digits() {
                f.write_char(digit.to_char())?;
            }
            return Ok(());
        }

        let (before, after) = match f.align().unwrap_or(fmt::Alignment::Right) {
            fmt::Alignment::Left => (Nat::zero(), padding.clone()),
            fmt::Alignment::Right => (padding.clone(), Nat::zero()),
            fmt::Alignment::Center => {
                let (half, _) = padding.div_rem(&Nat::two());
                let rest = padding.saturating_sub(&half);
                (half, rest)
            }
        };

        write_repeated(f, f.fill(), &before)?;
        for digit in self.digits() {
            f.write_char(digit.to_char())?;
        }
        write_repeated(f, f.fill(), &after)
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

        let mut head = None;
        for c in s.chars() {
            let digit = Digit::from_char(c).ok_or(ParseBigNumError::InvalidCharacter(c))?;
            head = cons(digit, head);
        }
        Ok(normalize(reverse(head)))
    }
}

impl Ord for BigNum {
    fn cmp(&self, other: &Self) -> Ordering {
        cmp_lists(&self.0, &other.0)
    }
}

impl PartialOrd for BigNum {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl From<Digit> for BigNum {
    fn from(digit: Digit) -> Self {
        Self(cons(digit, None))
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
                    value
                        .to_string()
                        .parse()
                        .expect("every unsigned integer has a decimal representation")
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
        for digit in iter {
            self.0 = append(self.0.take(), digit);
        }
        self.0 = normalize(self.0.take()).0;
    }
}

impl IntoIterator for BigNum {
    type Item = Digit;
    type IntoIter = IntoDigits;

    fn into_iter(self) -> Self::IntoIter {
        IntoDigits { next: self.0 }
    }
}

impl<'a> IntoIterator for &'a BigNum {
    type Item = Digit;
    type IntoIter = Digits<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.digits()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalization_removes_leading_zeroes() {
        assert_eq!(
            BigNum::from_digits([Digit::Zero, Digit::Zero, Digit::One, Digit::Two])
                .digits()
                .collect::<Vec<_>>(),
            vec![Digit::One, Digit::Two]
        );
        assert_eq!(
            BigNum::from_digits([]).digits().collect::<Vec<_>>(),
            vec![Digit::Zero]
        );
        assert_eq!(
            BigNum::from_digits([Digit::Zero, Digit::Zero])
                .digits()
                .collect::<Vec<_>>(),
            vec![Digit::Zero]
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
            vec![Digit::Nine, Digit::Eight, Digit::Seven]
        );
        assert_eq!(
            "12".parse::<BigNum>().unwrap().digits().collect::<Vec<_>>(),
            vec![Digit::One, Digit::Two]
        );

        let n: BigNum = "123".parse().unwrap();
        assert_eq!(n.digits().count(), 3);
        assert_eq!(n.digits().next(), Some(Digit::One));
        assert_eq!(n.digits().last(), Some(Digit::Three));
    }
}
