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
are per-operation medians. `BigNum` is shown twice: with the default-on `simd`
feature and with `--no-default-features` (scalar lookup-table path only).
`num-bigint` is unaffected by that feature. Every operation is slower for
`BigNum`, and the gap grows with size.

| Operation | Digits | BigNum (simd) | BigNum (scalar) | num-bigint |
|---|---|---|---|---|
| add | 10 | 47.5 ns | 46.1 ns | 4.77 ns |
| add | 100 | 278 ns | 146.8 ns | 33.3 ns |
| add | 1,000 | 1.95 µs | 1.15 µs | 96.4 ns |
| add | 10,000 | 18.7 µs | 12.0 µs | 673 ns |
| sub | 10 | 58.9 ns | 51.6 ns | 6.19 ns |
| sub | 100 | 273.6 ns | 129.9 ns | 34.9 ns |
| sub | 1,000 | 1.91 µs | 1.04 µs | 78.6 ns |
| sub | 10,000 | 18.7 µs | 10.1 µs | 423 ns |
| mul | 10 | 234.5 ns | 198.6 ns | 4.66 ns |
| mul | 100 | 33.7 µs | 15.5 µs | 68.1 ns |
| mul | 1,000 | 5.79 ms | 7.25 ms | 1.77 µs |
| mul | 10,000 | 577 ms | 726 ms | 76.7 µs |
| div | 10 | 1.92 µs | 1.77 µs | 19.9 ns |
| div | 100 | 140.3 µs | 83.3 µs | 250 ns |
| div | 1,000 | 17.9 ms | 17.5 ms | 4.09 µs |
| div | 10,000 | *not benchmarked* | *not benchmarked* | 181 µs |

Division at 10,000 digits is skipped for `BigNum`: it divides by repeated
subtraction per quotient digit, and the case does not finish in acceptable
benchmark time. `num-bigint` uses Knuth division and is benchmarked at that
size. This is measured behavior of schoolbook long division on enum digits —
orders of magnitude slower than `num-bigint` across the board, and more so as
operand width grows.

Per-digit cost here is dominated by table lookups and `Vec` allocation rather
than ALU throughput: each digit operation is an index into a 10×10 (or
10×10×2) `const` table instead of a hardware instruction.

The `simd` feature's byte-lane path (used for operands of 32 or more digits)
is measured above rather than asserted: multiplication at 1,000 and 10,000
digits is about 20% faster with it, while addition and subtraction are
slower — transcode, lane bookkeeping, and the serial carry ripple cost more
than the lane-parallel table pass saves. Enabling it is therefore a tradeoff,
and the numbers are what they are.

### Reproducing

```sh
cargo bench --bench vs_num_bigint --no-default-features # BigNum (scalar) column
cargo bench --bench vs_num_bigint                       # BigNum (simd) column
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
