//! Wide-lane ("SIMD-style") acceleration for long-number arithmetic.
//!
//! The loops below process fixed-size chunks of `u8` lanes with no early
//! exits, in the shape the compiler auto-vectorizes; there are no
//! `core::arch` intrinsics and no `unsafe`, so `#![forbid(unsafe_code)]`
//! still holds and the code stays portable.
//!
//! # Honesty of the lanes
//!
//! The crate's invariant — digit *values* come from lookup tables, never
//! from primitive arithmetic — is preserved. A lane is a structural index:
//! the `u8` produced by the existing `From<Digit> for u8` conversion (a
//! `repr(u8)` cast, which the README already carves out as structural), and
//! every lane result is read back out of a lookup table. The flattened
//! per-lane tables below are `const` transcriptions of digit.rs's
//! `ADD_TABLE`/`SUB_TABLE`/`MUL_TABLE`, so the digit values still originate
//! in those tables. Inside the lane loops only shifts, masks, or-ing of
//! table indices, and table lookups happen; carries and borrows ripple with
//! successor/predecessor lookup tables (`NEXT_LANE`/`PREV_LANE`), mirroring
//! digit.rs's `NEXT_DIGIT`. No primitive `+ - * / %` ever computes a digit
//! value.
//!
//! Carry propagation is inherently sequential, so each operation is split
//! into a lane-parallel pass (per-lane reduced digit + carry/borrow flag out
//! of the flattened table) followed by a serial ripple that uses only table
//! lookups. Results are bit-for-bit identical to the scalar table path for
//! all inputs.

use crate::digit::Digit;

/// Wide-lane arithmetic, compiled only with the `simd` feature (default on).
#[cfg(feature = "simd")]
mod wide {
    use crate::big_num::normalize_vec;
    use crate::digit::{ADD_TABLE, Digit, MUL_TABLE, SUB_TABLE};

    /// Operands shorter than this use the scalar table path; the per-call
    /// transcode/lane overhead is not worth it below this size.
    pub(super) const THRESHOLD: usize = 32;

    /// Lanes processed per chunk. Fixed-size chunks with no early exits keep
    /// the hot loops auto-vectorizable.
    const LANES: usize = 32;

    /// Flattened per-lane addition table: indexed by `(a << 4) | b`, each
    /// entry packs `(reduced digit << 4) | carry flag`. Built from
    /// digit.rs's `ADD_TABLE`, so digit values remain table-sourced.
    static ADD_LUT: [u8; 256] = flatten_add_table();

    /// Flattened per-lane subtraction table: indexed by `(a << 4) | b`, each
    /// entry packs `(reduced digit << 4) | borrow flag`.
    static SUB_LUT: [u8; 256] = flatten_sub_table();

    /// Flattened per-lane multiplication table: indexed by `(a << 4) | b`,
    /// each entry packs `(units digit << 4) | carry digit`.
    static MUL_LUT: [u8; 256] = flatten_mul_table();

    /// Lane indices of the ten digits, for decoding table results.
    const LANE_DIGITS: [Digit; 10] = [
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

    /// Lane-index successor table, mirroring digit.rs's `NEXT_DIGIT`.
    const NEXT_LANE: [u8; 10] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 0];

    /// Lane-index predecessor table, the borrow-side mirror of `NEXT_LANE`.
    const PREV_LANE: [u8; 10] = [9, 0, 1, 2, 3, 4, 5, 6, 7, 8];

    const fn flatten_add_table() -> [u8; 256] {
        let mut lut = [0u8; 256];
        let mut a = 0;
        while a < 10 {
            let mut b = 0;
            while b < 10 {
                let (sum, carry) = ADD_TABLE[a][b];
                lut[(a << 4) | b] = (sum.value() << 4) | (carry as u8);
                b += 1;
            }
            a += 1;
        }
        lut
    }

    const fn flatten_sub_table() -> [u8; 256] {
        let mut lut = [0u8; 256];
        let mut a = 0;
        while a < 10 {
            let mut b = 0;
            while b < 10 {
                let (difference, borrow) = SUB_TABLE[a][b];
                lut[(a << 4) | b] = (difference.value() << 4) | (borrow as u8);
                b += 1;
            }
            a += 1;
        }
        lut
    }

    const fn flatten_mul_table() -> [u8; 256] {
        let mut lut = [0u8; 256];
        let mut a = 0;
        while a < 10 {
            let mut b = 0;
            while b < 10 {
                let (units, carry) = MUL_TABLE[a][b];
                lut[(a << 4) | b] = (units.value() << 4) | carry.value();
                b += 1;
            }
            a += 1;
        }
        lut
    }

    /// Transcodes big-endian `Digit`s into little-endian `u8` lane indices.
    ///
    /// The per-element conversion is the existing `From<Digit> for u8` impl
    /// (a `repr(u8)` cast), which the README already classifies as a
    /// structural bitwise conversion.
    fn transcode_le(digits: &[Digit], lanes: &mut [u8]) {
        for (i, &digit) in digits.iter().rev().enumerate() {
            lanes[i] = u8::from(digit);
        }
    }

    /// Lane-parallel table pass: for each lane computes the reduced digit
    /// (`hi`) and carry/borrow flag (`lo`) for the table entry indexed by the
    /// two lane values. Full 32-lane chunks first, then a scalar tail; no
    /// early exits anywhere.
    fn lane_lut_pass(a: &[u8], b: &[u8], hi: &mut [u8], lo: &mut [u8], lut: &[u8; 256]) {
        let n = a.len();
        let mut i = 0;
        while i + LANES <= n {
            for l in 0..LANES {
                let entry = lut[((a[i + l] << 4) | b[i + l]) as usize];
                hi[i + l] = entry >> 4;
                lo[i + l] = entry & 0x0F;
            }
            i += LANES;
        }
        while i < n {
            let entry = lut[((a[i] << 4) | b[i]) as usize];
            hi[i] = entry >> 4;
            lo[i] = entry & 0x0F;
            i += 1;
        }
    }

    /// Lane-parallel table pass for one fixed operand `a` against every lane
    /// of `b` (used for multiplication rows).
    fn lane_mul_pass(a: u8, b: &[u8], units: &mut [u8], carries: &mut [u8]) {
        let n = b.len();
        let a_index = (a << 4) as usize;
        let mut i = 0;
        while i + LANES <= n {
            for l in 0..LANES {
                let entry = MUL_LUT[a_index | b[i + l] as usize];
                units[i + l] = entry >> 4;
                carries[i + l] = entry & 0x0F;
            }
            i += LANES;
        }
        while i < n {
            let entry = MUL_LUT[a_index | b[i] as usize];
            units[i] = entry >> 4;
            carries[i] = entry & 0x0F;
            i += 1;
        }
    }

    /// Wide-lane addition. Output layout matches the scalar `add_digits`
    /// exactly: `max(len)` digits, plus one more only on a final carry.
    pub(super) fn wide_add(lhs: &[Digit], rhs: &[Digit]) -> Vec<Digit> {
        let n = lhs.len().max(rhs.len());
        let mut a = vec![0u8; n];
        let mut b = vec![0u8; n];
        transcode_le(lhs, &mut a);
        transcode_le(rhs, &mut b);

        let mut sum = vec![0u8; n];
        let mut carry_flag = vec![0u8; n];
        lane_lut_pass(&a, &b, &mut sum, &mut carry_flag, &ADD_LUT);

        // Serial carry ripple: with an incoming carry, the reduced digit is
        // succeeded and the outgoing carry is the incoming one surviving a
        // lane of 9, OR the lane's own carry flag.
        let mut carry = false;
        for i in 0..n {
            let digit = sum[i];
            let flag = carry_flag[i] == 1;
            if carry {
                sum[i] = NEXT_LANE[digit as usize];
                carry = digit == 9 || flag;
            } else {
                carry = flag;
            }
        }

        let mut out = Vec::with_capacity(n + usize::from(carry));
        if carry {
            out.push(Digit::One);
        }
        for &lane in sum.iter().rev() {
            out.push(LANE_DIGITS[lane as usize]);
        }
        out
    }

    /// Wide-lane subtraction for `lhs >= rhs`. Output matches the scalar
    /// `checked_sub_digits`: normalized, `lhs.len()` digits before
    /// normalization.
    pub(super) fn wide_sub(lhs: &[Digit], rhs: &[Digit]) -> Vec<Digit> {
        let n = lhs.len();
        let mut a = vec![0u8; n];
        let mut b = vec![0u8; n];
        transcode_le(lhs, &mut a);
        transcode_le(rhs, &mut b);

        let mut diff = vec![0u8; n];
        let mut borrow_flag = vec![0u8; n];
        lane_lut_pass(&a, &b, &mut diff, &mut borrow_flag, &SUB_LUT);

        // Serial borrow ripple: with an incoming borrow, the reduced digit is
        // preceded and the outgoing borrow is the incoming one surviving a
        // lane of 0, OR the lane's own borrow flag.
        let mut borrow = false;
        for i in 0..n {
            let digit = diff[i];
            let flag = borrow_flag[i] == 1;
            if borrow {
                diff[i] = PREV_LANE[digit as usize];
                borrow = digit == 0 || flag;
            } else {
                borrow = flag;
            }
        }
        debug_assert!(!borrow);

        let out: Vec<Digit> = diff
            .iter()
            .rev()
            .map(|&lane| LANE_DIGITS[lane as usize])
            .collect();
        normalize_vec(out)
    }

    /// Wide-lane schoolbook multiplication: per `lhs` digit, the partial
    /// product row (units and carries over every `rhs` lane) comes out of the
    /// flattened `MUL_LUT` in one parallel pass, then each row is folded into
    /// the result with a parallel add pass plus serial ripple. Same total
    /// value and same normalized output as the scalar path.
    pub(super) fn wide_mul(lhs: &[Digit], rhs: &[Digit]) -> Vec<Digit> {
        let m = rhs.len();
        let mut rhs_le = vec![0u8; m];
        transcode_le(rhs, &mut rhs_le);

        let mut result = vec![0u8; lhs.len() + m + 1];
        let mut units = vec![0u8; m];
        let mut carries = vec![0u8; m];
        let mut partial = vec![0u8; m];
        let mut flags = vec![0u8; m];

        for (i, &digit) in lhs.iter().rev().enumerate() {
            let a = u8::from(digit);
            lane_mul_pass(a, &rhs_le, &mut units, &mut carries);
            add_row_into(&mut result, i, &units, &mut partial, &mut flags);
            add_row_into(&mut result, i + 1, &carries, &mut partial, &mut flags);
        }

        let out: Vec<Digit> = result
            .iter()
            .rev()
            .map(|&lane| LANE_DIGITS[lane as usize])
            .collect();
        normalize_vec(out)
    }

    /// Adds one row of lane digits into `result` starting at `offset`,
    /// mirroring the scalar `add_digit_at` loop: a parallel pass computes each
    /// position's reduced sum and carry flag, then carries ripple serially
    /// (and, if still set, onward through the higher result lanes).
    fn add_row_into(
        result: &mut [u8],
        offset: usize,
        row: &[u8],
        partial: &mut [u8],
        flags: &mut [u8],
    ) {
        let m = row.len();
        lane_lut_pass(&result[offset..offset + m], row, partial, flags, &ADD_LUT);

        let mut carry = false;
        for j in 0..m {
            let digit = partial[j];
            let flag = flags[j] == 1;
            if carry {
                partial[j] = NEXT_LANE[digit as usize];
                carry = digit == 9 || flag;
            } else {
                carry = flag;
            }
            result[offset + j] = partial[j];
        }

        let mut index = offset + m;
        while carry {
            assert!(
                index < result.len(),
                "multiplication overflowed its result buffer"
            );
            let entry = ADD_LUT[((result[index] << 4) | 1) as usize];
            result[index] = entry >> 4;
            carry = entry & 1 == 1;
            index += 1;
        }
    }
}

/// Wide-lane addition for long operands; `None` means "use the scalar table
/// path" (feature disabled or operands below the lane threshold).
pub(crate) fn simd_add(lhs: &[Digit], rhs: &[Digit]) -> Option<Vec<Digit>> {
    #[cfg(feature = "simd")]
    {
        if lhs.len() >= wide::THRESHOLD && rhs.len() >= wide::THRESHOLD {
            return Some(wide::wide_add(lhs, rhs));
        }
    }
    #[cfg(not(feature = "simd"))]
    {
        let _ = (lhs, rhs);
    }
    None
}

/// Wide-lane subtraction for `lhs >= rhs` (the caller has already compared);
/// `None` means "use the scalar table path".
pub(crate) fn simd_sub(lhs: &[Digit], rhs: &[Digit]) -> Option<Vec<Digit>> {
    #[cfg(feature = "simd")]
    {
        if lhs.len() >= wide::THRESHOLD && rhs.len() >= wide::THRESHOLD {
            return Some(wide::wide_sub(lhs, rhs));
        }
    }
    #[cfg(not(feature = "simd"))]
    {
        let _ = (lhs, rhs);
    }
    None
}

/// Wide-lane multiplication for long operands; `None` means "use the scalar
/// table path".
pub(crate) fn simd_mul(lhs: &[Digit], rhs: &[Digit]) -> Option<Vec<Digit>> {
    #[cfg(feature = "simd")]
    {
        if lhs.len() >= wide::THRESHOLD && rhs.len() >= wide::THRESHOLD {
            return Some(wide::wide_mul(lhs, rhs));
        }
    }
    #[cfg(not(feature = "simd"))]
    {
        let _ = (lhs, rhs);
    }
    None
}
