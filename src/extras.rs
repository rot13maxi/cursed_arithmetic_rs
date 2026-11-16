//! Number-theory party tricks.

use crate::big_num::BigNum;
use crate::digit::Digit;

impl BigNum {
    /// Raises this number to a non-negative power.
    ///
    /// `0.pow(0)` is `1`, matching the usual integer convention.
    pub fn pow(&self, mut exponent: u32) -> Self {
        let mut base = self.clone();
        let mut result = BigNum::one();

        while exponent > 0 {
            if exponent & 1 == 1 {
                result *= &base;
            }
            exponent >>= 1;
            if exponent > 0 {
                base = &base * &base;
            }
        }

        result
    }

    /// Computes `n!`.
    pub fn factorial(n: u32) -> Self {
        let mut result = BigNum::one();
        for factor in 2..=n {
            result *= BigNum::from(factor);
        }
        result
    }

    /// Greatest common divisor, via Euclid's algorithm.
    pub fn gcd(&self, other: &Self) -> Self {
        let mut a = self.clone();
        let mut b = other.clone();

        while !b.is_zero() {
            let remainder = a % &b;
            a = b;
            b = remainder;
        }

        a
    }

    /// The sum of this number's decimal digits.
    pub fn digit_sum(&self) -> Self {
        self.digits()
            .iter()
            .fold(BigNum::zero(), |acc, digit| acc + BigNum::from(*digit))
    }

    /// The repeated digit sum, a single digit.
    pub fn digital_root(&self) -> Digit {
        let mut value = self.clone();
        loop {
            let digits = value.digits();
            if digits.len() == 1 {
                return digits[0];
            }
            value = value.digit_sum();
        }
    }

    /// Whether this number is even.
    pub fn is_even(&self) -> bool {
        self.digits().last().is_some_and(|digit| digit.is_even())
    }

    /// Whether this number is odd.
    pub fn is_odd(&self) -> bool {
        !self.is_even()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn n(s: &str) -> BigNum {
        s.parse().unwrap()
    }

    #[test]
    fn powers_work() {
        assert_eq!(n("2").pow(0).to_string(), "1");
        assert_eq!(n("0").pow(0).to_string(), "1");
        assert_eq!(n("2").pow(10).to_string(), "1024");
        assert_eq!(
            n("2").pow(100).to_string(),
            "1267650600228229401496703205376"
        );
        assert_eq!(
            n("10").pow(30).to_string(),
            "1000000000000000000000000000000"
        );
    }

    #[test]
    fn factorial_works() {
        assert_eq!(BigNum::factorial(0).to_string(), "1");
        assert_eq!(BigNum::factorial(1).to_string(), "1");
        assert_eq!(BigNum::factorial(5).to_string(), "120");
        assert_eq!(BigNum::factorial(20).to_string(), "2432902008176640000");
        assert_eq!(
            BigNum::factorial(100).to_string(),
            "93326215443944152681699238856266700490715968264381621468592963895217599993229915608941463976156518286253697920827223758251185210916864000000000000000000000000"
        );
    }

    #[test]
    fn gcd_works() {
        assert_eq!(n("48").gcd(&n("18")).to_string(), "6");
        assert_eq!(n("17").gcd(&n("13")).to_string(), "1");
        assert_eq!(n("0").gcd(&n("5")).to_string(), "5");
        assert_eq!(n("5").gcd(&n("0")).to_string(), "5");
        assert_eq!(n("0").gcd(&n("0")).to_string(), "0");
    }

    #[test]
    fn digit_properties_work() {
        assert_eq!(n("123456789").digit_sum().to_string(), "45");
        assert_eq!(n("999999999999999999999999").digit_sum().to_string(), "216");
        assert_eq!(n("123456789").digital_root(), Digit::Nine);
        assert_eq!(n("0").digital_root(), Digit::Zero);
        assert_eq!(n("10").digital_root(), Digit::One);
        assert!(n("1234").is_even());
        assert!(n("1235").is_odd());
        assert!(n("0").is_even());
    }
}
