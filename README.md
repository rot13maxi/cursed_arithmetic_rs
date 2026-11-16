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

## Performance Considerations

Benchmarks against `num-bigint` (`BigUint`) on identical deterministic inputs
(Apple Silicon, criterion, 20 samples per case) produced the following. Values
are per-operation medians; every operation is slower for `BigNum`, and the gap
grows with size.

| Operation | Digits | BigNum | num-bigint |
|---|---|---|---|
| add | 10 | 186 ns | 23 ns |
| add | 100 | 1.11 µs | 122 ns |
| add | 1,000 | 7.24 µs | 411 ns |
| add | 10,000 | 70.5 µs | 1.94 µs |
| sub | 10 | 201 ns | 25 ns |
| sub | 100 | 1.11 µs | 121 ns |
| sub | 1,000 | 8.02 µs | 361 ns |
| sub | 10,000 | 68.7 µs | 1.67 µs |
| mul | 10 | 976 ns | 22 ns |
| mul | 100 | 170 µs | 272 ns |
| mul | 1,000 | 20.7 ms | 7.02 µs |
| mul | 10,000 | 1.92 s | 310 µs |
| div | 10 | 7.75 µs | 96 ns |
| div | 100 | 671 µs | 975 ns |
| div | 1,000 | 56.8 ms | 12.7 µs |
| div | 10,000 | *not benchmarked* | *not benchmarked* |

Division at 10,000 digits was skipped: `BigNum` divides by repeated
subtraction per quotient digit, and the case does not finish in acceptable
benchmark time. This is measured behavior of schoolbook long division on enum
digits — orders of magnitude slower than `num-bigint` across the board, and
more so as operand width grows.

Per-digit cost here is dominated by table lookups and `Vec` allocation rather
than ALU throughput: each digit operation is an index into a 10×10 (or
10×10×2) `const` table instead of a hardware instruction. The default-on
`simd` feature accelerates add/sub/mul for operands of 32 or more digits via
a byte-lane path, which is what the wide-dispatch benchmarks above reflect at
the 100+ digit sizes.

### Reproducing

```sh
cargo bench --bench vs_num_bigint
```

Benchmark dependencies (`criterion`, `num-bigint`) are dev-dependencies only;
they are never compiled into the library itself.

## Development

```sh
cargo test
cargo clippy --all-targets -- -D warnings
cargo fmt --check
```

The test suite compares small random values against `u128` and checks large
random numbers for algebraic consistency (`(a * b) / b == a`, distributivity,
commutativity, and parse/display round trips).
