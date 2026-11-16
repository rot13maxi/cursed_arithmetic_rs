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

/// Expands a 10×10 table of digit results into an exhaustive `match`.
macro_rules! digit_table {
    ($name:ident, $ret:ty, $($row:ident: [$(($col:ident, $a:ident, $b:ident)),* $(,)?]),* $(,)?) => {
        pub(crate) const fn $name(lhs: Digit, rhs: Digit) -> $ret {
            match lhs {
                $( Digit::$row => match rhs {
                    $( Digit::$col => ($a, $b), )*
                }, )*
            }
        }
    };
}

digit_table!(add_table, (Digit, bool),
    Zero: [(Zero, Zero, false), (One, One, false), (Two, Two, false), (Three, Three, false), (Four, Four, false), (Five, Five, false), (Six, Six, false), (Seven, Seven, false), (Eight, Eight, false), (Nine, Nine, false)],
    One: [(Zero, One, false), (One, Two, false), (Two, Three, false), (Three, Four, false), (Four, Five, false), (Five, Six, false), (Six, Seven, false), (Seven, Eight, false), (Eight, Nine, false), (Nine, Zero, true)],
    Two: [(Zero, Two, false), (One, Three, false), (Two, Four, false), (Three, Five, false), (Four, Six, false), (Five, Seven, false), (Six, Eight, false), (Seven, Nine, false), (Eight, Zero, true), (Nine, One, true)],
    Three: [(Zero, Three, false), (One, Four, false), (Two, Five, false), (Three, Six, false), (Four, Seven, false), (Five, Eight, false), (Six, Nine, false), (Seven, Zero, true), (Eight, One, true), (Nine, Two, true)],
    Four: [(Zero, Four, false), (One, Five, false), (Two, Six, false), (Three, Seven, false), (Four, Eight, false), (Five, Nine, false), (Six, Zero, true), (Seven, One, true), (Eight, Two, true), (Nine, Three, true)],
    Five: [(Zero, Five, false), (One, Six, false), (Two, Seven, false), (Three, Eight, false), (Four, Nine, false), (Five, Zero, true), (Six, One, true), (Seven, Two, true), (Eight, Three, true), (Nine, Four, true)],
    Six: [(Zero, Six, false), (One, Seven, false), (Two, Eight, false), (Three, Nine, false), (Four, Zero, true), (Five, One, true), (Six, Two, true), (Seven, Three, true), (Eight, Four, true), (Nine, Five, true)],
    Seven: [(Zero, Seven, false), (One, Eight, false), (Two, Nine, false), (Three, Zero, true), (Four, One, true), (Five, Two, true), (Six, Three, true), (Seven, Four, true), (Eight, Five, true), (Nine, Six, true)],
    Eight: [(Zero, Eight, false), (One, Nine, false), (Two, Zero, true), (Three, One, true), (Four, Two, true), (Five, Three, true), (Six, Four, true), (Seven, Five, true), (Eight, Six, true), (Nine, Seven, true)],
    Nine: [(Zero, Nine, false), (One, Zero, true), (Two, One, true), (Three, Two, true), (Four, Three, true), (Five, Four, true), (Six, Five, true), (Seven, Six, true), (Eight, Seven, true), (Nine, Eight, true)],
);

digit_table!(sub_table, (Digit, bool),
    Zero: [(Zero, Zero, false), (One, Nine, true), (Two, Eight, true), (Three, Seven, true), (Four, Six, true), (Five, Five, true), (Six, Four, true), (Seven, Three, true), (Eight, Two, true), (Nine, One, true)],
    One: [(Zero, One, false), (One, Zero, false), (Two, Nine, true), (Three, Eight, true), (Four, Seven, true), (Five, Six, true), (Six, Five, true), (Seven, Four, true), (Eight, Three, true), (Nine, Two, true)],
    Two: [(Zero, Two, false), (One, One, false), (Two, Zero, false), (Three, Nine, true), (Four, Eight, true), (Five, Seven, true), (Six, Six, true), (Seven, Five, true), (Eight, Four, true), (Nine, Three, true)],
    Three: [(Zero, Three, false), (One, Two, false), (Two, One, false), (Three, Zero, false), (Four, Nine, true), (Five, Eight, true), (Six, Seven, true), (Seven, Six, true), (Eight, Five, true), (Nine, Four, true)],
    Four: [(Zero, Four, false), (One, Three, false), (Two, Two, false), (Three, One, false), (Four, Zero, false), (Five, Nine, true), (Six, Eight, true), (Seven, Seven, true), (Eight, Six, true), (Nine, Five, true)],
    Five: [(Zero, Five, false), (One, Four, false), (Two, Three, false), (Three, Two, false), (Four, One, false), (Five, Zero, false), (Six, Nine, true), (Seven, Eight, true), (Eight, Seven, true), (Nine, Six, true)],
    Six: [(Zero, Six, false), (One, Five, false), (Two, Four, false), (Three, Three, false), (Four, Two, false), (Five, One, false), (Six, Zero, false), (Seven, Nine, true), (Eight, Eight, true), (Nine, Seven, true)],
    Seven: [(Zero, Seven, false), (One, Six, false), (Two, Five, false), (Three, Four, false), (Four, Three, false), (Five, Two, false), (Six, One, false), (Seven, Zero, false), (Eight, Nine, true), (Nine, Eight, true)],
    Eight: [(Zero, Eight, false), (One, Seven, false), (Two, Six, false), (Three, Five, false), (Four, Four, false), (Five, Three, false), (Six, Two, false), (Seven, One, false), (Eight, Zero, false), (Nine, Nine, true)],
    Nine: [(Zero, Nine, false), (One, Eight, false), (Two, Seven, false), (Three, Six, false), (Four, Five, false), (Five, Four, false), (Six, Three, false), (Seven, Two, false), (Eight, One, false), (Nine, Zero, false)],
);

digit_table!(mul_table, (Digit, Digit),
    Zero: [(Zero, Zero, Zero), (One, Zero, Zero), (Two, Zero, Zero), (Three, Zero, Zero), (Four, Zero, Zero), (Five, Zero, Zero), (Six, Zero, Zero), (Seven, Zero, Zero), (Eight, Zero, Zero), (Nine, Zero, Zero)],
    One: [(Zero, Zero, Zero), (One, One, Zero), (Two, Two, Zero), (Three, Three, Zero), (Four, Four, Zero), (Five, Five, Zero), (Six, Six, Zero), (Seven, Seven, Zero), (Eight, Eight, Zero), (Nine, Nine, Zero)],
    Two: [(Zero, Zero, Zero), (One, Two, Zero), (Two, Four, Zero), (Three, Six, Zero), (Four, Eight, Zero), (Five, Zero, One), (Six, Two, One), (Seven, Four, One), (Eight, Six, One), (Nine, Eight, One)],
    Three: [(Zero, Zero, Zero), (One, Three, Zero), (Two, Six, Zero), (Three, Nine, Zero), (Four, Two, One), (Five, Five, One), (Six, Eight, One), (Seven, One, Two), (Eight, Four, Two), (Nine, Seven, Two)],
    Four: [(Zero, Zero, Zero), (One, Four, Zero), (Two, Eight, Zero), (Three, Two, One), (Four, Six, One), (Five, Zero, Two), (Six, Four, Two), (Seven, Eight, Two), (Eight, Two, Three), (Nine, Six, Three)],
    Five: [(Zero, Zero, Zero), (One, Five, Zero), (Two, Zero, One), (Three, Five, One), (Four, Zero, Two), (Five, Five, Two), (Six, Zero, Three), (Seven, Five, Three), (Eight, Zero, Four), (Nine, Five, Four)],
    Six: [(Zero, Zero, Zero), (One, Six, Zero), (Two, Two, One), (Three, Eight, One), (Four, Four, Two), (Five, Zero, Three), (Six, Six, Three), (Seven, Two, Four), (Eight, Eight, Four), (Nine, Four, Five)],
    Seven: [(Zero, Zero, Zero), (One, Seven, Zero), (Two, Four, One), (Three, One, Two), (Four, Eight, Two), (Five, Five, Three), (Six, Two, Four), (Seven, Nine, Four), (Eight, Six, Five), (Nine, Three, Six)],
    Eight: [(Zero, Zero, Zero), (One, Eight, Zero), (Two, Six, One), (Three, Four, Two), (Four, Two, Three), (Five, Zero, Four), (Six, Eight, Four), (Seven, Six, Five), (Eight, Four, Six), (Nine, Two, Seven)],
    Nine: [(Zero, Zero, Zero), (One, Nine, Zero), (Two, Eight, One), (Three, Seven, Two), (Four, Six, Three), (Five, Five, Four), (Six, Four, Five), (Seven, Three, Six), (Eight, Two, Seven), (Nine, One, Eight)],
);

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
        match self {
            Zero => 0,
            One => 1,
            Two => 2,
            Three => 3,
            Four => 4,
            Five => 5,
            Six => 6,
            Seven => 7,
            Eight => 8,
            Nine => 9,
        }
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
        match self {
            Zero => '0',
            One => '1',
            Two => '2',
            Three => '3',
            Four => '4',
            Five => '5',
            Six => '6',
            Seven => '7',
            Eight => '8',
            Nine => '9',
        }
    }

    /// Adds two digits plus an incoming carry, returning the result digit and
    /// the outgoing carry.
    pub const fn carrying_add(self, rhs: Self, carry: bool) -> (Self, bool) {
        let (sum, carry_out) = add_table(self, rhs);
        if carry {
            let (sum, carry_from_one) = add_table(sum, One);
            (sum, carry_out || carry_from_one)
        } else {
            (sum, carry_out)
        }
    }

    /// Subtracts two digits and an incoming borrow, returning the result digit
    /// and the outgoing borrow.
    pub const fn borrowing_sub(self, rhs: Self, borrow: bool) -> (Self, bool) {
        let (difference, borrow_out) = sub_table(self, rhs);
        if borrow {
            let (difference, borrow_from_one) = sub_table(difference, One);
            (difference, borrow_out || borrow_from_one)
        } else {
            (difference, borrow_out)
        }
    }

    /// Multiplies two digits, returning `(units, carry)`.
    pub const fn carrying_mul(self, rhs: Self) -> (Self, Self) {
        mul_table(self, rhs)
    }

    /// Checked digit addition.
    pub const fn checked_add(self, rhs: Self) -> Option<Self> {
        let (sum, carry) = add_table(self, rhs);
        if carry { None } else { Some(sum) }
    }

    /// Checked digit subtraction.
    pub const fn checked_sub(self, rhs: Self) -> Option<Self> {
        let (difference, borrow) = sub_table(self, rhs);
        if borrow { None } else { Some(difference) }
    }

    /// Checked digit multiplication. Returns `None` when the product exceeds
    /// nine.
    pub const fn checked_mul(self, rhs: Self) -> Option<Self> {
        let (units, carry) = mul_table(self, rhs);
        match carry {
            Zero => Some(units),
            _ => None,
        }
    }

    /// Digit addition modulo ten.
    pub const fn wrapping_add(self, rhs: Self) -> Self {
        add_table(self, rhs).0
    }

    /// Digit subtraction modulo ten.
    pub const fn wrapping_sub(self, rhs: Self) -> Self {
        sub_table(self, rhs).0
    }

    /// Digit multiplication modulo ten.
    pub const fn wrapping_mul(self, rhs: Self) -> Self {
        mul_table(self, rhs).0
    }

    /// The next digit, wrapping from nine back to zero.
    pub const fn successor(self) -> Self {
        match self {
            Zero => One,
            One => Two,
            Two => Three,
            Three => Four,
            Four => Five,
            Five => Six,
            Six => Seven,
            Seven => Eight,
            Eight => Nine,
            Nine => Zero,
        }
    }

    /// Whether this digit is even.
    pub const fn is_even(self) -> bool {
        match self {
            Zero | Two | Four | Six | Eight => true,
            One | Three | Five | Seven | Nine => false,
        }
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
            value: u32::from(value),
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
