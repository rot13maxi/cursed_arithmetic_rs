# cursed_arithmetic

Integer and bignum arithmetic built from decimal lookup tables and a linked
list of enum digits. No primitive arithmetic is used to compute digit values.

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

`Digit` is a real enum with ten variants. `BigNum` stores digits in a singly
linked list, most significant first. There are no slices, no indices, and no
`Vec<Digit>` inside `BigNum`. Schoolbook arithmetic walks the list: carries,
borrows, long multiplication, and long division.

Digit operations are exhaustive `match` tables, not arrays indexed by casts:

```rust
use std::ops::Add;
use cursed_arithmetic::Digit;

assert_eq!(Digit::Nine + Digit::One, (Digit::Zero, true));
assert_eq!(Digit::Three + Digit::Four, (Digit::Seven, false));
```

Structural quantities — lengths, shifts, exponents, padding, and loop
counters — are also `BigNum`s, wrapped in an internal `Nat`. Even the number of
zeroes to append is computed by table-driven successor and predecessor walks.

## No primitive arithmetic

Every digit-level result comes from a 10×10 `match` table:

- `carrying_add` and `borrowing_sub` look up a result digit and carry/borrow,
  with a second lookup for an incoming carry or borrow.
- `carrying_mul` looks up `(units, carry)`.
- `successor`, parity, and digit-to-character mapping are exhaustive matches.

`BigNum` is built from those tables. Multiplication adds table-computed
partial products; division repeatedly subtracts the divisor and advances the
quotient with `Digit::successor`. Primitive integers appear only at the API
boundary, where `usize`/`u128`/`u32` values are converted through decimal
strings, and inside `Box`/`Vec`/`std` formatting internals. Tests use primitive
arithmetic as an independent oracle.

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

`BigNum::digits()` now returns a borrowing iterator instead of a slice.
`BigNum::into_digits()` still returns a `Vec<Digit>` at the boundary.

## Semantics

- Numbers are non-negative. There is no sign.
- `BigNum` is normalized: no leading zeroes, zero is a single `Digit::Zero`
  node.
- Subtraction panics on underflow, matching primitive integer behavior.
- Division and remainder panic on a zero divisor. Use the checked methods to
  get `Option` instead.
- `Shl`/`Shr` operate on decimal digits, not binary bits.

## Performance Considerations

This is intentionally absurd. The linked-list representation allocates one
`Box` per digit, and multiplication allocates partial products rather than
using slices. Division still uses repeated subtraction per quotient digit.
Expect it to be dramatically slower than `num-bigint`, especially at large
sizes.

The benchmark harness compares `BigNum` against `num_bigint::BigUint` on
identical deterministic inputs:

```sh
cargo bench --bench vs_num_bigint
```

The old SIMD feature and its wide-lane benchmark column are gone. There is no
`simd` feature anymore; every path uses the cursed scalar tables.

## Development

```sh
cargo test
cargo clippy --all-targets -- -D warnings
cargo fmt --check
```

The test suite compares small random values against `u128` and checks large
random numbers for algebraic consistency (`(a * b) / b == a`, distributivity,
commutativity, and parse/display round trips).
