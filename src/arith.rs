//! Schoolbook arithmetic on decimal digits.

use crate::big_num::{BigNum, cmp_digits, normalize_vec, trimmed};
use crate::digit::Digit;
use std::cmp::Ordering;
use std::iter::{Product, Sum};
use std::ops::{
    Add, AddAssign, Div, DivAssign, Mul, MulAssign, Rem, RemAssign, Shl, ShlAssign, Shr, ShrAssign,
    Sub, SubAssign,
};

pub(crate) fn add_digits(lhs: &[Digit], rhs: &[Digit]) -> Vec<Digit> {
    let mut out = Vec::with_capacity(lhs.len().max(rhs.len()) + 1);
    let mut lhs_iter = lhs.iter().rev();
    let mut rhs_iter = rhs.iter().rev();
    let mut carry = false;

    loop {
        let lhs_digit = lhs_iter.next().copied();
        let rhs_digit = rhs_iter.next().copied();
        if lhs_digit.is_none() && rhs_digit.is_none() && !carry {
            break;
        }

        let (digit, next_carry) = lhs_digit
            .unwrap_or(Digit::Zero)
            .carrying_add(rhs_digit.unwrap_or(Digit::Zero), carry);
        out.push(digit);
        carry = next_carry;
    }

    out.reverse();
    out
}

pub(crate) fn checked_sub_digits(lhs: &[Digit], rhs: &[Digit]) -> Option<Vec<Digit>> {
    if cmp_digits(lhs, rhs) == Ordering::Less {
        return None;
    }

    let mut out = Vec::with_capacity(lhs.len());
    let mut rhs_iter = rhs.iter().rev();
    let mut borrow = false;

    for lhs_digit in lhs.iter().rev() {
        let rhs_digit = rhs_iter.next().copied().unwrap_or(Digit::Zero);
        let (digit, next_borrow) = lhs_digit.borrowing_sub(rhs_digit, borrow);
        out.push(digit);
        borrow = next_borrow;
    }

    debug_assert!(!borrow);
    out.reverse();
    Some(normalize_vec(out))
}

pub(crate) fn sub_digits(lhs: &[Digit], rhs: &[Digit]) -> Vec<Digit> {
    checked_sub_digits(lhs, rhs).unwrap_or_else(|| panic!("attempt to subtract with overflow"))
}

fn add_digit_at(digits: &mut [Digit], mut index: usize, digit: Digit) {
    let mut carry = digit;
    loop {
        let (sum, overflow) = digits[index].carrying_add(carry, false);
        digits[index] = sum;
        if !overflow {
            break;
        }
        carry = Digit::One;
        index += 1;
        assert!(
            index < digits.len(),
            "multiplication overflowed its result buffer"
        );
    }
}

pub(crate) fn mul_digits(lhs: &[Digit], rhs: &[Digit]) -> Vec<Digit> {
    if trimmed(lhs).is_empty() || trimmed(rhs).is_empty() {
        return vec![Digit::Zero];
    }

    let mut result = vec![Digit::Zero; lhs.len() + rhs.len() + 1];
    for (i, lhs_digit) in lhs.iter().rev().enumerate() {
        for (j, rhs_digit) in rhs.iter().rev().enumerate() {
            let (units, carry) = lhs_digit.carrying_mul(*rhs_digit);
            add_digit_at(&mut result, i + j, units);
            add_digit_at(&mut result, i + j + 1, carry);
        }
    }

    result.reverse();
    normalize_vec(result)
}

pub(crate) fn div_rem_digits(dividend: &[Digit], divisor: &[Digit]) -> (Vec<Digit>, Vec<Digit>) {
    assert!(!trimmed(divisor).is_empty(), "attempt to divide by zero");

    let mut quotient = Vec::with_capacity(dividend.len());
    let mut remainder = vec![Digit::Zero];

    for &digit in dividend {
        if remainder.len() == 1 && remainder[0] == Digit::Zero {
            remainder[0] = digit;
        } else {
            remainder.push(digit);
        }

        let mut quotient_digit = Digit::Zero;
        while cmp_digits(&remainder, divisor) != Ordering::Less {
            remainder = checked_sub_digits(&remainder, divisor)
                .expect("the remainder is at least the divisor");
            quotient_digit = quotient_digit.successor();
        }

        quotient.push(quotient_digit);
    }

    (normalize_vec(quotient), normalize_vec(remainder))
}

impl BigNum {
    /// Returns the quotient and remainder of `self / rhs`.
    ///
    /// # Panics
    ///
    /// Panics if `rhs` is zero.
    pub fn div_rem(&self, rhs: &Self) -> (Self, Self) {
        let (quotient, remainder) = div_rem_digits(self.digits(), rhs.digits());
        (
            BigNum::from_digits(quotient),
            BigNum::from_digits(remainder),
        )
    }

    /// Checked subtraction. Returns `None` on underflow.
    pub fn checked_sub(&self, rhs: &Self) -> Option<Self> {
        checked_sub_digits(self.digits(), rhs.digits()).map(BigNum::from_digits)
    }

    /// Checked division. Returns `None` when `rhs` is zero.
    pub fn checked_div(&self, rhs: &Self) -> Option<Self> {
        if rhs.is_zero() {
            None
        } else {
            Some(self.div_rem(rhs).0)
        }
    }

    /// Checked remainder. Returns `None` when `rhs` is zero.
    pub fn checked_rem(&self, rhs: &Self) -> Option<Self> {
        if rhs.is_zero() {
            None
        } else {
            Some(self.div_rem(rhs).1)
        }
    }

    /// Multiplies by `10^n` by appending `n` zero digits.
    pub fn mul_pow10(&self, n: usize) -> Self {
        if self.is_zero() || n == 0 {
            return self.clone();
        }

        let mut digits = self.digits().to_vec();
        digits.resize(digits.len() + n, Digit::Zero);
        BigNum::from_digits(digits)
    }

    /// Divides by `10^n` by dropping `n` least-significant digits.
    pub fn div_pow10(&self, n: usize) -> Self {
        let len = self.digits().len();
        if n >= len {
            BigNum::zero()
        } else {
            BigNum::from_digits(self.digits()[..len - n].iter().copied())
        }
    }
}

impl Add for BigNum {
    type Output = BigNum;

    fn add(self, rhs: BigNum) -> BigNum {
        BigNum::from_digits(add_digits(self.digits(), rhs.digits()))
    }
}

impl Sub for BigNum {
    type Output = BigNum;

    /// # Panics
    ///
    /// Panics if `rhs` is larger than `self`.
    fn sub(self, rhs: BigNum) -> BigNum {
        BigNum::from_digits(sub_digits(self.digits(), rhs.digits()))
    }
}

impl Mul for BigNum {
    type Output = BigNum;

    fn mul(self, rhs: BigNum) -> BigNum {
        BigNum::from_digits(mul_digits(self.digits(), rhs.digits()))
    }
}

impl Div for BigNum {
    type Output = BigNum;

    /// # Panics
    ///
    /// Panics if `rhs` is zero.
    fn div(self, rhs: BigNum) -> BigNum {
        self.div_rem(&rhs).0
    }
}

impl Rem for BigNum {
    type Output = BigNum;

    /// # Panics
    ///
    /// Panics if `rhs` is zero.
    fn rem(self, rhs: BigNum) -> BigNum {
        self.div_rem(&rhs).1
    }
}

impl AddAssign for BigNum {
    fn add_assign(&mut self, rhs: BigNum) {
        *self = BigNum::from_digits(add_digits(self.digits(), rhs.digits()));
    }
}

impl SubAssign for BigNum {
    fn sub_assign(&mut self, rhs: BigNum) {
        *self = BigNum::from_digits(sub_digits(self.digits(), rhs.digits()));
    }
}

impl MulAssign for BigNum {
    fn mul_assign(&mut self, rhs: BigNum) {
        *self = BigNum::from_digits(mul_digits(self.digits(), rhs.digits()));
    }
}

impl DivAssign for BigNum {
    fn div_assign(&mut self, rhs: BigNum) {
        *self = self.div_rem(&rhs).0;
    }
}

impl RemAssign for BigNum {
    fn rem_assign(&mut self, rhs: BigNum) {
        *self = self.div_rem(&rhs).1;
    }
}

macro_rules! impl_binop_ref_variants {
    ($trait:ident, $method:ident) => {
        impl $trait<&BigNum> for BigNum {
            type Output = BigNum;

            fn $method(self, rhs: &BigNum) -> BigNum {
                self.$method(rhs.clone())
            }
        }

        impl $trait<BigNum> for &BigNum {
            type Output = BigNum;

            fn $method(self, rhs: BigNum) -> BigNum {
                self.clone().$method(rhs)
            }
        }

        impl $trait<&BigNum> for &BigNum {
            type Output = BigNum;

            fn $method(self, rhs: &BigNum) -> BigNum {
                self.clone().$method(rhs.clone())
            }
        }
    };
}

impl_binop_ref_variants!(Add, add);
impl_binop_ref_variants!(Sub, sub);
impl_binop_ref_variants!(Mul, mul);
impl_binop_ref_variants!(Div, div);
impl_binop_ref_variants!(Rem, rem);

macro_rules! impl_assign_ref {
    ($assign:ident, $method:ident, $op:tt) => {
        impl $assign<&BigNum> for BigNum {
            fn $method(&mut self, rhs: &BigNum) {
                *self = self.clone() $op rhs;
            }
        }
    };
}

impl_assign_ref!(AddAssign, add_assign, +);
impl_assign_ref!(SubAssign, sub_assign, -);
impl_assign_ref!(MulAssign, mul_assign, *);
impl_assign_ref!(DivAssign, div_assign, /);
impl_assign_ref!(RemAssign, rem_assign, %);

macro_rules! impl_primitive_ops {
    ($($ty:ty),* $(,)?) => {
        $(
            impl Add<$ty> for BigNum {
                type Output = BigNum;

                fn add(self, rhs: $ty) -> BigNum {
                    self + BigNum::from(rhs)
                }
            }

            impl Sub<$ty> for BigNum {
                type Output = BigNum;

                fn sub(self, rhs: $ty) -> BigNum {
                    self - BigNum::from(rhs)
                }
            }

            impl Mul<$ty> for BigNum {
                type Output = BigNum;

                fn mul(self, rhs: $ty) -> BigNum {
                    self * BigNum::from(rhs)
                }
            }

            impl Div<$ty> for BigNum {
                type Output = BigNum;

                fn div(self, rhs: $ty) -> BigNum {
                    self / BigNum::from(rhs)
                }
            }

            impl Rem<$ty> for BigNum {
                type Output = BigNum;

                fn rem(self, rhs: $ty) -> BigNum {
                    self % BigNum::from(rhs)
                }
            }

            impl AddAssign<$ty> for BigNum {
                fn add_assign(&mut self, rhs: $ty) {
                    *self = self.clone() + BigNum::from(rhs);
                }
            }

            impl SubAssign<$ty> for BigNum {
                fn sub_assign(&mut self, rhs: $ty) {
                    *self = self.clone() - BigNum::from(rhs);
                }
            }

            impl MulAssign<$ty> for BigNum {
                fn mul_assign(&mut self, rhs: $ty) {
                    *self = self.clone() * BigNum::from(rhs);
                }
            }

            impl DivAssign<$ty> for BigNum {
                fn div_assign(&mut self, rhs: $ty) {
                    *self = self.clone() / BigNum::from(rhs);
                }
            }

            impl RemAssign<$ty> for BigNum {
                fn rem_assign(&mut self, rhs: $ty) {
                    *self = self.clone() % BigNum::from(rhs);
                }
            }
        )*
    };
}

impl_primitive_ops!(u8, u16, u32, u64, u128, usize);

impl Shl<usize> for BigNum {
    type Output = BigNum;

    /// Decimal shift: `n << k` means `n * 10^k`.
    fn shl(self, rhs: usize) -> BigNum {
        self.mul_pow10(rhs)
    }
}

impl Shr<usize> for BigNum {
    type Output = BigNum;

    /// Decimal shift: `n >> k` means `n / 10^k`, truncated.
    fn shr(self, rhs: usize) -> BigNum {
        self.div_pow10(rhs)
    }
}

impl ShlAssign<usize> for BigNum {
    fn shl_assign(&mut self, rhs: usize) {
        *self = self.clone().mul_pow10(rhs);
    }
}

impl ShrAssign<usize> for BigNum {
    fn shr_assign(&mut self, rhs: usize) {
        *self = self.clone().div_pow10(rhs);
    }
}

impl Sum for BigNum {
    fn sum<I: Iterator<Item = BigNum>>(iter: I) -> BigNum {
        iter.fold(BigNum::zero(), |acc, n| acc + n)
    }
}

impl<'a> Sum<&'a BigNum> for BigNum {
    fn sum<I: Iterator<Item = &'a BigNum>>(iter: I) -> BigNum {
        iter.fold(BigNum::zero(), |acc, n| acc + n)
    }
}

impl Product for BigNum {
    fn product<I: Iterator<Item = BigNum>>(iter: I) -> BigNum {
        iter.fold(BigNum::one(), |acc, n| acc * n)
    }
}

impl<'a> Product<&'a BigNum> for BigNum {
    fn product<I: Iterator<Item = &'a BigNum>>(iter: I) -> BigNum {
        iter.fold(BigNum::one(), |acc, n| acc * n)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn n(s: &str) -> BigNum {
        s.parse().unwrap()
    }

    #[test]
    fn addition_handles_carries() {
        assert_eq!((n("0") + n("0")).to_string(), "0");
        assert_eq!((n("999") + n("1")).to_string(), "1000");
        assert_eq!((n("123") + n("456")).to_string(), "579");
        assert_eq!(
            (n("99999999999999999999") + n("1")).to_string(),
            "100000000000000000000"
        );
    }

    #[test]
    fn subtraction_handles_borrows() {
        assert_eq!((n("1000") - n("1")).to_string(), "999");
        assert_eq!((n("123") - n("123")).to_string(), "0");
        assert_eq!(
            (n("100000000000000000000") - n("1")).to_string(),
            "99999999999999999999"
        );
        assert_eq!(n("3").checked_sub(&n("4")), None);
    }

    #[test]
    #[should_panic(expected = "attempt to subtract with overflow")]
    fn subtraction_underflow_panics() {
        let _ = n("3") - n("4");
    }

    #[test]
    fn multiplication_is_schoolbook() {
        assert_eq!((n("0") * n("999")).to_string(), "0");
        assert_eq!((n("999") * n("999")).to_string(), "998001");
        assert_eq!((n("12345") * n("67890")).to_string(), "838102050");
        assert_eq!(
            (n("12345678901234567890") * n("98765432109876543210")).to_string(),
            "1219326311370217952237463801111263526900"
        );
    }

    #[test]
    fn division_and_remainder_are_correct() {
        let (q, r) = n("1000").div_rem(&n("10"));
        assert_eq!(
            (q.to_string(), r.to_string()),
            ("100".to_string(), "0".to_string())
        );

        let (q, r) = n("1000").div_rem(&n("3"));
        assert_eq!(
            (q.to_string(), r.to_string()),
            ("333".to_string(), "1".to_string())
        );

        let (q, r) = n("7").div_rem(&n("10"));
        assert_eq!(
            (q.to_string(), r.to_string()),
            ("0".to_string(), "7".to_string())
        );

        let (q, r) = n("123456789012345678901234567890").div_rem(&n("987654321"));
        assert_eq!(q.to_string(), "124999998873437499901");
        assert_eq!(r.to_string(), "574845669");
    }

    #[test]
    #[should_panic(expected = "attempt to divide by zero")]
    fn division_by_zero_panics() {
        let _ = n("1") / n("0");
    }

    #[test]
    fn shifts_are_decimal() {
        assert_eq!((n("123") << 3).to_string(), "123000");
        assert_eq!((n("123000") >> 3).to_string(), "123");
        assert_eq!((n("123") >> 10).to_string(), "0");
        assert_eq!((n("0") << 5).to_string(), "0");
    }

    #[test]
    fn primitive_and_reference_ops_work() {
        let a = n("12");
        let b = n("30");
        assert_eq!((a.clone() + 8u32).to_string(), "20");
        assert_eq!((&a + &b).to_string(), "42");
        assert_eq!((a.clone() + &b).to_string(), "42");
        assert_eq!((&a + b).to_string(), "42");

        let mut c = n("10");
        c += 5u64;
        c -= 3u8;
        c *= 2u16;
        c /= 4u32;
        c %= 5u128;
        assert_eq!(c.to_string(), "1");
    }

    #[test]
    fn iterator_sums_and_products_work() {
        let nums = [n("1"), n("2"), n("3")];
        assert_eq!(nums.iter().sum::<BigNum>().to_string(), "6");
        assert_eq!(nums.iter().product::<BigNum>().to_string(), "6");
        assert_eq!(nums.into_iter().sum::<BigNum>().to_string(), "6");
    }
}
