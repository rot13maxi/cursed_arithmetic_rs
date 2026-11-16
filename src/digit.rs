//! A single decimal digit and its cursed little arithmetic helpers.

use std::fmt::{self, Write as _};
use std::ops::{Add, Sub};
use std::str::FromStr;

/// One decimal digit, `0` through `9`.
///
/// This is deliberately an enum rather than a `u8`: the whole point of the
/// crate is that even a single digit is a slightly unreasonable type.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum Digit {
    /// `0`
    Zero = 0,
    /// `1`
    One = 1,
    /// `2`
    Two = 2,
    /// `3`
    Three = 3,
    /// `4`
    Four = 4,
    /// `5`
    Five = 5,
    /// `6`
    Six = 6,
    /// `7`
    Seven = 7,
    /// `8`
    Eight = 8,
    /// `9`
    Nine = 9,
}

use Digit::*;

/// Addition without an incoming carry.
///
/// Indexed by the two digit values. Each entry is `(result, carry)`.
pub(crate) const ADD_TABLE: [[(Digit, bool); 10]; 10] = [
    [
        (Zero, false),
        (One, false),
        (Two, false),
        (Three, false),
        (Four, false),
        (Five, false),
        (Six, false),
        (Seven, false),
        (Eight, false),
        (Nine, false),
    ],
    [
        (One, false),
        (Two, false),
        (Three, false),
        (Four, false),
        (Five, false),
        (Six, false),
        (Seven, false),
        (Eight, false),
        (Nine, false),
        (Zero, true),
    ],
    [
        (Two, false),
        (Three, false),
        (Four, false),
        (Five, false),
        (Six, false),
        (Seven, false),
        (Eight, false),
        (Nine, false),
        (Zero, true),
        (One, true),
    ],
    [
        (Three, false),
        (Four, false),
        (Five, false),
        (Six, false),
        (Seven, false),
        (Eight, false),
        (Nine, false),
        (Zero, true),
        (One, true),
        (Two, true),
    ],
    [
        (Four, false),
        (Five, false),
        (Six, false),
        (Seven, false),
        (Eight, false),
        (Nine, false),
        (Zero, true),
        (One, true),
        (Two, true),
        (Three, true),
    ],
    [
        (Five, false),
        (Six, false),
        (Seven, false),
        (Eight, false),
        (Nine, false),
        (Zero, true),
        (One, true),
        (Two, true),
        (Three, true),
        (Four, true),
    ],
    [
        (Six, false),
        (Seven, false),
        (Eight, false),
        (Nine, false),
        (Zero, true),
        (One, true),
        (Two, true),
        (Three, true),
        (Four, true),
        (Five, true),
    ],
    [
        (Seven, false),
        (Eight, false),
        (Nine, false),
        (Zero, true),
        (One, true),
        (Two, true),
        (Three, true),
        (Four, true),
        (Five, true),
        (Six, true),
    ],
    [
        (Eight, false),
        (Nine, false),
        (Zero, true),
        (One, true),
        (Two, true),
        (Three, true),
        (Four, true),
        (Five, true),
        (Six, true),
        (Seven, true),
    ],
    [
        (Nine, false),
        (Zero, true),
        (One, true),
        (Two, true),
        (Three, true),
        (Four, true),
        (Five, true),
        (Six, true),
        (Seven, true),
        (Eight, true),
    ],
];

/// Subtraction without an incoming borrow.
///
/// Indexed by the two digit values. Each entry is `(result, borrow)`.
pub(crate) const SUB_TABLE: [[(Digit, bool); 10]; 10] = [
    [
        (Zero, false),
        (Nine, true),
        (Eight, true),
        (Seven, true),
        (Six, true),
        (Five, true),
        (Four, true),
        (Three, true),
        (Two, true),
        (One, true),
    ],
    [
        (One, false),
        (Zero, false),
        (Nine, true),
        (Eight, true),
        (Seven, true),
        (Six, true),
        (Five, true),
        (Four, true),
        (Three, true),
        (Two, true),
    ],
    [
        (Two, false),
        (One, false),
        (Zero, false),
        (Nine, true),
        (Eight, true),
        (Seven, true),
        (Six, true),
        (Five, true),
        (Four, true),
        (Three, true),
    ],
    [
        (Three, false),
        (Two, false),
        (One, false),
        (Zero, false),
        (Nine, true),
        (Eight, true),
        (Seven, true),
        (Six, true),
        (Five, true),
        (Four, true),
    ],
    [
        (Four, false),
        (Three, false),
        (Two, false),
        (One, false),
        (Zero, false),
        (Nine, true),
        (Eight, true),
        (Seven, true),
        (Six, true),
        (Five, true),
    ],
    [
        (Five, false),
        (Four, false),
        (Three, false),
        (Two, false),
        (One, false),
        (Zero, false),
        (Nine, true),
        (Eight, true),
        (Seven, true),
        (Six, true),
    ],
    [
        (Six, false),
        (Five, false),
        (Four, false),
        (Three, false),
        (Two, false),
        (One, false),
        (Zero, false),
        (Nine, true),
        (Eight, true),
        (Seven, true),
    ],
    [
        (Seven, false),
        (Six, false),
        (Five, false),
        (Four, false),
        (Three, false),
        (Two, false),
        (One, false),
        (Zero, false),
        (Nine, true),
        (Eight, true),
    ],
    [
        (Eight, false),
        (Seven, false),
        (Six, false),
        (Five, false),
        (Four, false),
        (Three, false),
        (Two, false),
        (One, false),
        (Zero, false),
        (Nine, true),
    ],
    [
        (Nine, false),
        (Eight, false),
        (Seven, false),
        (Six, false),
        (Five, false),
        (Four, false),
        (Three, false),
        (Two, false),
        (One, false),
        (Zero, false),
    ],
];

/// Multiplication without an incoming carry.
///
/// Indexed by the two digit values. Each entry is `(units, carry)`.
pub(crate) const MUL_TABLE: [[(Digit, Digit); 10]; 10] = [
    [
        (Zero, Zero),
        (Zero, Zero),
        (Zero, Zero),
        (Zero, Zero),
        (Zero, Zero),
        (Zero, Zero),
        (Zero, Zero),
        (Zero, Zero),
        (Zero, Zero),
        (Zero, Zero),
    ],
    [
        (Zero, Zero),
        (One, Zero),
        (Two, Zero),
        (Three, Zero),
        (Four, Zero),
        (Five, Zero),
        (Six, Zero),
        (Seven, Zero),
        (Eight, Zero),
        (Nine, Zero),
    ],
    [
        (Zero, Zero),
        (Two, Zero),
        (Four, Zero),
        (Six, Zero),
        (Eight, Zero),
        (Zero, One),
        (Two, One),
        (Four, One),
        (Six, One),
        (Eight, One),
    ],
    [
        (Zero, Zero),
        (Three, Zero),
        (Six, Zero),
        (Nine, Zero),
        (Two, One),
        (Five, One),
        (Eight, One),
        (One, Two),
        (Four, Two),
        (Seven, Two),
    ],
    [
        (Zero, Zero),
        (Four, Zero),
        (Eight, Zero),
        (Two, One),
        (Six, One),
        (Zero, Two),
        (Four, Two),
        (Eight, Two),
        (Two, Three),
        (Six, Three),
    ],
    [
        (Zero, Zero),
        (Five, Zero),
        (Zero, One),
        (Five, One),
        (Zero, Two),
        (Five, Two),
        (Zero, Three),
        (Five, Three),
        (Zero, Four),
        (Five, Four),
    ],
    [
        (Zero, Zero),
        (Six, Zero),
        (Two, One),
        (Eight, One),
        (Four, Two),
        (Zero, Three),
        (Six, Three),
        (Two, Four),
        (Eight, Four),
        (Four, Five),
    ],
    [
        (Zero, Zero),
        (Seven, Zero),
        (Four, One),
        (One, Two),
        (Eight, Two),
        (Five, Three),
        (Two, Four),
        (Nine, Four),
        (Six, Five),
        (Three, Six),
    ],
    [
        (Zero, Zero),
        (Eight, Zero),
        (Six, One),
        (Four, Two),
        (Two, Three),
        (Zero, Four),
        (Eight, Four),
        (Six, Five),
        (Four, Six),
        (Two, Seven),
    ],
    [
        (Zero, Zero),
        (Nine, Zero),
        (Eight, One),
        (Seven, Two),
        (Six, Three),
        (Five, Four),
        (Four, Five),
        (Three, Six),
        (Two, Seven),
        (One, Eight),
    ],
];

const NEXT_DIGIT: [Digit; 10] = [
    Digit::One,
    Digit::Two,
    Digit::Three,
    Digit::Four,
    Digit::Five,
    Digit::Six,
    Digit::Seven,
    Digit::Eight,
    Digit::Nine,
    Digit::Zero,
];

const DIGIT_CHARS: [char; 10] = ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9'];

const PARITY: [bool; 10] = [
    true, false, true, false, true, false, true, false, true, false,
];

impl Digit {
    /// Every digit, in ascending order.
    pub const ALL: [Digit; 10] = [
        Digit::Zero,
        Digit::One,
        Digit::Two,
        Digit::Three,
        Digit::Four,
        Digit::Five,
        Digit::Six,
        Digit::Seven,
        Digit::Eight,
        Digit::Nine,
    ];

    /// The numeric value of this digit.
    pub const fn value(self) -> u8 {
        self as u8
    }

    /// Returns the digit with the given numeric value, if it exists.
    pub const fn from_value(value: u8) -> Option<Self> {
        match value {
            0 => Some(Digit::Zero),
            1 => Some(Digit::One),
            2 => Some(Digit::Two),
            3 => Some(Digit::Three),
            4 => Some(Digit::Four),
            5 => Some(Digit::Five),
            6 => Some(Digit::Six),
            7 => Some(Digit::Seven),
            8 => Some(Digit::Eight),
            9 => Some(Digit::Nine),
            _ => None,
        }
    }

    /// Parses an ASCII decimal digit.
    pub const fn from_char(c: char) -> Option<Self> {
        match c {
            '0' => Some(Digit::Zero),
            '1' => Some(Digit::One),
            '2' => Some(Digit::Two),
            '3' => Some(Digit::Three),
            '4' => Some(Digit::Four),
            '5' => Some(Digit::Five),
            '6' => Some(Digit::Six),
            '7' => Some(Digit::Seven),
            '8' => Some(Digit::Eight),
            '9' => Some(Digit::Nine),
            _ => None,
        }
    }

    /// This digit as a `char`.
    pub const fn to_char(self) -> char {
        DIGIT_CHARS[self as usize]
    }

    /// Adds two digits plus an incoming carry, returning the result digit and
    /// the outgoing carry.
    ///
    /// The result comes from the addition lookup table; no primitive addition
    /// is performed.
    pub const fn carrying_add(self, rhs: Self, carry: bool) -> (Self, bool) {
        let (sum, carry_out) = ADD_TABLE[self as usize][rhs as usize];
        if carry {
            let (sum, carry_from_one) = ADD_TABLE[sum as usize][Digit::One as usize];
            (sum, carry_out || carry_from_one)
        } else {
            (sum, carry_out)
        }
    }

    /// Subtracts two digits and an incoming borrow, returning the result digit
    /// and the outgoing borrow.
    ///
    /// The result comes from the subtraction lookup table; no primitive
    /// subtraction is performed.
    pub const fn borrowing_sub(self, rhs: Self, borrow: bool) -> (Self, bool) {
        let (difference, borrow_out) = SUB_TABLE[self as usize][rhs as usize];
        if borrow {
            let (difference, borrow_from_one) = SUB_TABLE[difference as usize][Digit::One as usize];
            (difference, borrow_out || borrow_from_one)
        } else {
            (difference, borrow_out)
        }
    }

    /// Multiplies two digits, returning `(units, carry)`.
    ///
    /// The result comes from the multiplication lookup table; no primitive
    /// multiplication is performed.
    pub const fn carrying_mul(self, rhs: Self) -> (Self, Self) {
        MUL_TABLE[self as usize][rhs as usize]
    }

    /// Checked digit addition.
    pub const fn checked_add(self, rhs: Self) -> Option<Self> {
        let (sum, carry) = ADD_TABLE[self as usize][rhs as usize];
        if carry { None } else { Some(sum) }
    }

    /// Checked digit subtraction.
    pub const fn checked_sub(self, rhs: Self) -> Option<Self> {
        let (difference, borrow) = SUB_TABLE[self as usize][rhs as usize];
        if borrow { None } else { Some(difference) }
    }

    /// Checked digit multiplication. Returns `None` when the product exceeds
    /// nine.
    pub const fn checked_mul(self, rhs: Self) -> Option<Self> {
        let (units, carry) = MUL_TABLE[self as usize][rhs as usize];
        match carry {
            Digit::Zero => Some(units),
            _ => None,
        }
    }

    /// Digit addition modulo ten.
    pub const fn wrapping_add(self, rhs: Self) -> Self {
        ADD_TABLE[self as usize][rhs as usize].0
    }

    /// Digit subtraction modulo ten.
    pub const fn wrapping_sub(self, rhs: Self) -> Self {
        SUB_TABLE[self as usize][rhs as usize].0
    }

    /// Digit multiplication modulo ten.
    pub const fn wrapping_mul(self, rhs: Self) -> Self {
        MUL_TABLE[self as usize][rhs as usize].0
    }

    /// The next digit, wrapping from nine back to zero.
    pub const fn successor(self) -> Self {
        NEXT_DIGIT[self as usize]
    }

    /// Whether this digit is even.
    pub const fn is_even(self) -> bool {
        PARITY[self as usize]
    }

    /// Whether this digit is odd.
    pub const fn is_odd(self) -> bool {
        !self.is_even()
    }
}

/// The error returned when a value is not a decimal digit.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InvalidDigit {
    value: u32,
}

impl InvalidDigit {
    /// The code point or numeric value that was rejected.
    pub const fn value(self) -> u32 {
        self.value
    }
}

impl fmt::Display for InvalidDigit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "expected an ASCII decimal digit, got U+{:04X}",
            self.value
        )
    }
}

impl std::error::Error for InvalidDigit {}

/// The error returned when parsing a [`Digit`] from a string.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ParseDigitError;

impl fmt::Display for ParseDigitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("expected exactly one ASCII decimal digit")
    }
}

impl std::error::Error for ParseDigitError {}

impl TryFrom<u8> for Digit {
    type Error = InvalidDigit;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Self::from_value(value).ok_or(InvalidDigit {
            value: u32::from(value),
        })
    }
}

impl TryFrom<char> for Digit {
    type Error = InvalidDigit;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        Self::from_char(value).ok_or(InvalidDigit {
            value: value as u32,
        })
    }
}

impl From<Digit> for u8 {
    fn from(digit: Digit) -> Self {
        digit.value()
    }
}

impl From<Digit> for char {
    fn from(digit: Digit) -> Self {
        digit.to_char()
    }
}

impl From<Digit> for usize {
    fn from(digit: Digit) -> Self {
        usize::from(digit.value())
    }
}

impl fmt::Display for Digit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_char(self.to_char())
    }
}

impl FromStr for Digit {
    type Err = ParseDigitError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut chars = s.chars();
        match (chars.next(), chars.next()) {
            (Some(c), None) => Self::from_char(c).ok_or(ParseDigitError),
            _ => Err(ParseDigitError),
        }
    }
}

impl Add for Digit {
    /// Cursed but useful: digit addition returns `(result_digit, carry)`.
    type Output = (Digit, bool);

    fn add(self, rhs: Digit) -> Self::Output {
        self.carrying_add(rhs, false)
    }
}

impl Sub for Digit {
    /// Cursed but useful: digit subtraction returns `(result_digit, borrow)`.
    type Output = (Digit, bool);

    fn sub(self, rhs: Digit) -> Self::Output {
        self.borrowing_sub(rhs, false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_digits_round_trip() {
        for (value, digit) in Digit::ALL.into_iter().enumerate() {
            assert_eq!(digit.value(), value as u8);
            assert_eq!(Digit::from_value(value as u8), Some(digit));
            assert_eq!(Digit::from_char(digit.to_char()), Some(digit));
            assert_eq!(u8::from(digit), value as u8);
            assert_eq!(char::from(digit), digit.to_char());
            assert_eq!(digit.to_string(), digit.to_char().to_string());
        }
    }

    #[test]
    fn invalid_digits_are_rejected() {
        assert_eq!(Digit::from_value(10), None);
        assert_eq!(Digit::from_char('a'), None);
        assert_eq!(Digit::from_char('١'), None);
        assert_eq!(Digit::try_from(10), Err(InvalidDigit { value: 10 }));
        assert_eq!(
            Digit::try_from('x'),
            Err(InvalidDigit { value: 'x' as u32 })
        );
        assert_eq!("12".parse::<Digit>(), Err(ParseDigitError));
        assert_eq!("".parse::<Digit>(), Err(ParseDigitError));
        assert_eq!("7".parse::<Digit>(), Ok(Digit::Seven));
    }

    #[test]
    fn carrying_add_works() {
        assert_eq!(
            Digit::Three.carrying_add(Digit::Four, false),
            (Digit::Seven, false)
        );
        assert_eq!(
            Digit::Nine.carrying_add(Digit::One, false),
            (Digit::Zero, true)
        );
        assert_eq!(
            Digit::Nine.carrying_add(Digit::Nine, true),
            (Digit::Nine, true)
        );
        assert_eq!(
            Digit::Zero.carrying_add(Digit::Zero, false),
            (Digit::Zero, false)
        );
    }

    #[test]
    fn borrowing_sub_works() {
        assert_eq!(
            Digit::Seven.borrowing_sub(Digit::Four, false),
            (Digit::Three, false)
        );
        assert_eq!(
            Digit::Zero.borrowing_sub(Digit::One, false),
            (Digit::Nine, true)
        );
        assert_eq!(
            Digit::Zero.borrowing_sub(Digit::Zero, true),
            (Digit::Nine, true)
        );
        assert_eq!(
            Digit::Nine.borrowing_sub(Digit::Nine, false),
            (Digit::Zero, false)
        );
    }

    #[test]
    fn carrying_mul_works() {
        for lhs in Digit::ALL {
            for rhs in Digit::ALL {
                let product = u16::from(lhs.value()) * u16::from(rhs.value());
                let (units, carry) = lhs.carrying_mul(rhs);
                assert_eq!(u16::from(units.value()), product % 10);
                assert_eq!(u16::from(carry.value()), product / 10);
                assert_eq!(
                    lhs.checked_mul(rhs),
                    if product <= 9 { Some(units) } else { None }
                );
                assert_eq!(lhs.wrapping_mul(rhs), units);
            }
        }
    }

    #[test]
    fn successor_and_parity_work() {
        for (value, digit) in Digit::ALL.into_iter().enumerate() {
            let expected = Digit::from_value(((value + 1) % 10) as u8).unwrap();
            assert_eq!(digit.successor(), expected);
            assert_eq!(digit.is_even(), value % 2 == 0);
            assert_eq!(digit.is_odd(), value % 2 != 0);
        }
    }

    #[test]
    fn carry_and_borrow_tables_match_arithmetic() {
        for lhs in Digit::ALL {
            for rhs in Digit::ALL {
                for carry in [false, true] {
                    let sum = u16::from(lhs.value()) + u16::from(rhs.value()) + u16::from(carry);
                    assert_eq!(
                        lhs.carrying_add(rhs, carry),
                        (Digit::from_value((sum % 10) as u8).unwrap(), sum >= 10)
                    );

                    let difference =
                        i16::from(lhs.value()) - i16::from(rhs.value()) - i16::from(carry);
                    assert_eq!(
                        lhs.borrowing_sub(rhs, carry),
                        (
                            Digit::from_value(((difference + 10) % 10) as u8).unwrap(),
                            difference < 0
                        )
                    );
                }
            }
        }
    }

    #[test]
    fn checked_and_wrapping_arithmetic_agree() {
        for lhs in Digit::ALL {
            for rhs in Digit::ALL {
                assert_eq!(
                    lhs.checked_add(rhs),
                    if lhs.value() + rhs.value() <= 9 {
                        Some(lhs.wrapping_add(rhs))
                    } else {
                        None
                    }
                );
                assert_eq!(
                    lhs.checked_sub(rhs),
                    if lhs.value() >= rhs.value() {
                        Some(lhs.wrapping_sub(rhs))
                    } else {
                        None
                    }
                );
                assert_eq!(lhs + rhs, lhs.carrying_add(rhs, false));
                assert_eq!(lhs - rhs, lhs.borrowing_sub(rhs, false));
            }
        }
    }
}
