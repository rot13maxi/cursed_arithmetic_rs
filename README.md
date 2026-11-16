# cursed_arithmetic

Integer and bignum arithmetic built from decimal lookup tables. No primitive
arithmetic is used to compute digit values.

```rust
use cursed_arithmetic::{BigNum, Digit};

let a: BigNum = "12345678901234567890".parse().unwrap();
let b: BigNum = "98765432109876543210".parse().unwrap();
assert_eq!(
    (a * b).to_string(),
    "1219326311370217952237463801111263526900"
);

let n = BigNum::from_digits([Digit::Four, Digit::Two]);
assert_eq!(n.to_string(), "42");
```

## The cursed part

`Digit` is a real enum with ten variants. `BigNum` wraps a big-endian
`Vec<Digit>` and implements schoolbook arithmetic: carries, borrows, long
multiplication, and long division. No `unsafe`, no external dependencies, and
no primitive `+`, `-`, `*`, `/`, or `%` in the digit algorithms.

Digit addition and subtraction are operators with a twist:

```rust
use std::ops::Add;
use cursed_arithmetic::Digit;

assert_eq!(Digit::Nine + Digit::One, (Digit::Zero, true));
assert_eq!(Digit::Three + Digit::Four, (Digit::Seven, false));
```

## No primitive arithmetic

Every digit-level result comes from a lookup table:

- `carrying_add` and `borrowing_sub` index a 10×10 table, with a second lookup
  for an incoming carry or borrow.
- `carrying_mul` indexes a 10×10 table of `(units, carry)`.
- `successor`, parity, and digit-to-character mapping use small tables.

`BigNum` is built from those tables: multiplication adds partial products digit
by digit, division repeatedly subtracts the divisor and advances the quotient
with `Digit::successor`, and `u128` conversion walks bits and uses a one-bit
full-adder table.

The remaining primitive integer operations are structural: indices, lengths,
capacities, bitwise conversion, and the explicit `u128`/`u64` boundary. Tests
use primitive arithmetic as an independent oracle.

## What works

- `Add`, `Sub`, `Mul`, `Div`, `Rem` and their assign variants, including
  reference combinations and primitive integer RHS values.
- Numeric `Ord`, parsing with `FromStr`, and `Display`/`Debug`.
- `u128`/`u64` conversion, `Vec<Digit>` conversion, iteration, `Sum`, and
  `Product`.
- Decimal shifts: `n << k` is `n * 10^k`, `n >> k` is `n / 10^k`.
- `pow`, `factorial`, `gcd`, `digit_sum`, `digital_root`, `is_even`, and
  `is_odd`.
- `checked_sub`, `checked_div`, `checked_rem`, and `div_rem`.

## Semantics

- Numbers are non-negative. There is no sign.
- `BigNum` is normalized: no leading zeroes, zero is `[Digit::Zero]`.
- Subtraction panics on underflow, matching primitive integer behavior.
- Division and remainder panic on a zero divisor. Use the checked methods to
  get `Option` instead.
- `Shl`/`Shr` operate on decimal digits, not binary bits.

## Development

```sh
cargo test
cargo clippy --all-targets -- -D warnings
cargo fmt --check
```

The test suite compares small random values against `u128` and checks large
random numbers for algebraic consistency (`(a * b) / b == a`, distributivity,
commutativity, and parse/display round trips).
