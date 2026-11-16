//! Arbitrary-precision arithmetic where every digit is an enum variant, every
//! number is a `Vec<Digit>`, and every digit operation is a lookup table.
//!
//! This crate is a joke with a straight face. It implements schoolbook
//! decimal arithmetic on [`BigNum`] without using primitive arithmetic to
//! compute digit values. Addition, subtraction, multiplication, division,
//! remainders, comparisons, parsing, and formatting all work.
//!
//! # No primitive arithmetic
//!
//! [`Digit::carrying_add`], [`Digit::borrowing_sub`], and
//! [`Digit::carrying_mul`] are lookup tables. Big-number multiplication adds
//! table-computed partial products; division uses table-driven subtraction and
//! [`Digit::successor`]. `u128` conversion walks bits and uses a one-bit
//! full-adder table.
//!
//! Structural index and length arithmetic still uses `usize`, and primitive
//! integer types remain at the API boundary. Tests use primitive arithmetic as
//! an independent oracle.
//!
//! ```
//! use cursed_arithmetic::{BigNum, Digit};
//!
//! let a: BigNum = "12345678901234567890".parse().unwrap();
//! let b: BigNum = "98765432109876543210".parse().unwrap();
//! assert_eq!(
//!     (a * b).to_string(),
//!     "1219326311370217952237463801111263526900"
//! );
//!
//! let n = BigNum::from_digits([Digit::Four, Digit::Two]);
//! assert_eq!(n.to_string(), "42");
//! ```
//!
//! The [`Digit`] type also has cursed operator implementations: adding two
//! digits returns `(result, carry)`, and subtracting returns `(result,
//! borrow)`.
//!
//! ```
//! use std::ops::Add;
//! use cursed_arithmetic::Digit;
//!
//! assert_eq!(Digit::Nine + Digit::One, (Digit::Zero, true));
//! assert_eq!(Digit::Three + Digit::Four, (Digit::Seven, false));
//! ```
//!
//! # Panics
//!
//! Subtraction panics on underflow, and division or remainder panics when the
//! divisor is zero. Use [`BigNum::checked_sub`], [`BigNum::checked_div`], or
//! [`BigNum::checked_rem`] when you would rather get `None`.
//!
//! # Representation
//!
//! [`BigNum`] stores digits most significant first, with no leading zeroes.
//! Zero is `[Digit::Zero]`. Arithmetic is done exactly the way you did it on
//! paper, carries and borrows included.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod arith;
mod big_num;
mod digit;
mod extras;

pub use big_num::{BigNum, ParseBigNumError};
pub use digit::{Digit, InvalidDigit, ParseDigitError};
