//! Benchmarks `cursed_arithmetic::BigNum` against `num_bigint::BigUint` on
//! identical inputs at several decimal-digit sizes.
//!
//! The harness may use primitive arithmetic; the library under test may not.
//! Inputs come from a small deterministic LCG so every run benchmarks the
//! same numbers.

use std::hint::black_box;

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main, measurement::WallTime};
use cursed_arithmetic::BigNum;
use num_bigint::BigUint;

/// Deterministic LCG; benches are the oracle layer and may use primitives.
struct Lcg(u64);

impl Lcg {
    fn next_u64(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        self.0
    }

    fn next_digit(&mut self) -> u8 {
        (self.next_u64() % 10) as u8
    }

    /// A decimal string of exactly `digits` digits, leading digit nonzero.
    fn decimal_string(&mut self, digits: usize) -> String {
        let mut s = String::with_capacity(digits);
        s.push((b'1' + (self.next_digit() % 9) as char as u8) as char);
        for _ in 1..digits {
            s.push((b'0' + self.next_digit()) as char);
        }
        s
    }
}

struct Operands {
    cursed: (BigNum, BigNum),
    num: (BigUint, BigUint),
}

fn operands(digits: usize) -> Operands {
    let mut lcg = Lcg(0x5EED_C0DE_1234_5678);
    let a = lcg.decimal_string(digits);
    let b = lcg.decimal_string(digits);
    Operands {
        cursed: (a.parse().unwrap(), b.parse().unwrap()),
        num: (a.parse().unwrap(), b.parse().unwrap()),
    }
}

struct QuotientOperands {
    cursed: (BigNum, BigNum),
    num: (BigUint, BigUint),
}

/// Division operands: a 2N-digit dividend and an N-digit divisor, so the
/// quotient has roughly N digits at every size.
fn quotient_operands(digits: usize) -> QuotientOperands {
    let mut lcg = Lcg(0x5EED_C0DE_1234_5678);
    let a = lcg.decimal_string(digits);
    let b = lcg.decimal_string(digits);
    let dividend = format!("{a}{b}");
    QuotientOperands {
        cursed: (dividend.parse().unwrap(), b.parse().unwrap()),
        num: (dividend.parse().unwrap(), b.parse().unwrap()),
    }
}

fn cursed_add(ops: &Operands) {
    black_box(black_box(&ops.cursed.0).clone() + black_box(&ops.cursed.1).clone());
}

fn cursed_sub(ops: &Operands) {
    // BigNum is non-negative; subtract min from max so the benchmark
    // exercises borrowing, not the underflow panic.
    let (a, b) = &ops.cursed;
    let (lhs, rhs) = if a >= b { (a, b) } else { (b, a) };
    black_box(black_box(lhs).clone() - black_box(rhs).clone());
}

fn cursed_mul(ops: &Operands) {
    black_box(black_box(&ops.cursed.0).clone() * black_box(&ops.cursed.1).clone());
}

fn cursed_div(ops: &QuotientOperands) {
    let (q, _) = black_box(&ops.cursed.0).div_rem(black_box(&ops.cursed.1));
    black_box(q);
}

fn num_add(ops: &Operands) {
    black_box(black_box(&ops.num.0).clone() + black_box(&ops.num.1).clone());
}

fn num_sub(ops: &Operands) {
    let (a, b) = &ops.num;
    let (lhs, rhs) = if a >= b { (a, b) } else { (b, a) };
    black_box(black_box(lhs).clone() - black_box(rhs).clone());
}

fn num_mul(ops: &Operands) {
    black_box(black_box(&ops.num.0).clone() * black_box(&ops.num.1).clone());
}

fn num_div(ops: &QuotientOperands) {
    black_box(black_box(&ops.num.0).clone() / black_box(&ops.num.1).clone());
}

/// `op/size` under `group/lib`, with `args` captured as black-boxed inputs.
fn bench_case<W: criterion::measurement::Measurement, T>(
    group: &mut criterion::BenchmarkGroup<'_, W>,
    lib: &str,
    op: &str,
    digits: usize,
    f: impl Fn() -> T,
) {
    group.bench_with_input(
        BenchmarkId::new(format!("{op}/{lib}"), digits),
        &(),
        |b, ()| b.iter(|| black_box(f())),
    );
}

const SIZES: [usize; 4] = [10, 100, 1_000, 10_000];

/// `BigNum` divides by repeated subtraction per quotient digit, so division
/// at this size does not finish in acceptable bench time; it is intentionally
/// not benchmarked. `num-bigint` uses Knuth division and handles this size,
/// so it is benchmarked with a per-library size cap.
const DIV_MAX_SIZE: [usize; 2] = [1_000, 10_000];
const LIB_BIGNUM: usize = 0;
const LIB_NUM_BIGINT: usize = 1;

fn bench_all(c: &mut Criterion<WallTime>) {
    let mut group = c.benchmark_group("vs_num_bigint");
    group.sample_size(20);
    group.measurement_time(std::time::Duration::from_secs(2));
    group.warm_up_time(std::time::Duration::from_millis(300));

    for &digits in &SIZES {
        let ops = operands(digits);

        bench_case(&mut group, "bignum", "add", digits, || cursed_add(&ops));
        bench_case(&mut group, "num_bigint", "add", digits, || num_add(&ops));
        bench_case(&mut group, "bignum", "sub", digits, || cursed_sub(&ops));
        bench_case(&mut group, "num_bigint", "sub", digits, || num_sub(&ops));
        bench_case(&mut group, "bignum", "mul", digits, || cursed_mul(&ops));
        bench_case(&mut group, "num_bigint", "mul", digits, || num_mul(&ops));

        if digits <= DIV_MAX_SIZE[LIB_BIGNUM] {
            let div_ops = quotient_operands(digits);
            bench_case(&mut group, "bignum", "div", digits, || cursed_div(&div_ops));
        }
        if digits <= DIV_MAX_SIZE[LIB_NUM_BIGINT] {
            let div_ops = quotient_operands(digits);
            bench_case(&mut group, "num_bigint", "div", digits, || {
                num_div(&div_ops)
            });
        }
    }

    group.finish();
}

criterion_group!(benches, bench_all);
criterion_main!(benches);
