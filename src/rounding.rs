//! Implementation of increment rounding functionality

use crate::options::{TemporalRoundingMode, TemporalUnsignedRoundingMode};

use std::{
    cmp::Ordering,
    ops::{Div, Neg},
};

use num_traits::{ConstZero, Euclid, FromPrimitive, NumCast, Signed, ToPrimitive};

pub(crate) trait Roundable:
    Euclid + Div + PartialOrd + Signed + FromPrimitive + ToPrimitive + NumCast + ConstZero + Copy
{
    fn is_exact(dividend: Self, divisor: Self) -> bool;
    fn compare_remainder(dividend: Self, divisor: Self) -> Option<Ordering>;
    fn is_even_cardinal(dividend: Self, divisor: Self) -> bool;
    fn result_floor(dividend: Self, divisor: Self) -> u64;
    fn result_ceil(dividend: Self, divisor: Self) -> u64;
    fn quotient_abs(dividend: Self, divisor: Self) -> Self {
        // NOTE: Sanity debugs until proper unit tests to vet the below
        debug_assert!(<i128 as NumCast>::from((dividend / divisor).abs()) < Some(u64::MAX as i128));
        (dividend / divisor).abs()
    }
}

pub(crate) trait Round {
    fn round(&self, mode: TemporalRoundingMode) -> i128;
    fn round_as_positive(&self, mode: TemporalRoundingMode) -> u64;
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub(crate) struct IncrementRounder<T: Roundable> {
    sign: bool,
    dividend: T,
    divisor: T,
}

impl<T: Roundable> IncrementRounder<T> {
    // ==== PUBLIC ====
    pub(crate) fn from_potentially_negative_parts(number: T, increment: T) -> Self {
        Self {
            sign: number / increment > T::ZERO,
            dividend: number,
            divisor: increment,
        }
    }

    pub(crate) fn from_positive_parts(number: T, increment: T) -> Self {
        debug_assert!(number / increment > T::ZERO);

        Self {
            sign: true,
            dividend: number,
            divisor: increment,
        }
    }
}

impl<T: Roundable> Round for IncrementRounder<T> {
    fn round(&self, mode: TemporalRoundingMode) -> i128 {
        let unsigned_rounding_mode = mode.get_unsigned_round_mode(self.sign);
        let mut rounded =
            apply_unsigned_rounding_mode(self.dividend, self.divisor, unsigned_rounding_mode)
                as i128;
        if !self.sign {
            rounded = rounded.neg();
        }
        // TODO: Add unit tests for the below
        rounded
            * <i128 as NumCast>::from(self.divisor).expect("increment is representable by a u64")
    }

    fn round_as_positive(&self, mode: TemporalRoundingMode) -> u64 {
        let unsigned_rounding_mode = mode.get_unsigned_round_mode(self.sign);
        let rounded =
            apply_unsigned_rounding_mode(self.dividend, self.divisor, unsigned_rounding_mode);
        // TODO: Add unit tests for the below
        rounded * <u64 as NumCast>::from(self.divisor).expect("increment is representable by a u64")
    }
}

impl Roundable for i128 {
    fn is_exact(dividend: Self, divisor: Self) -> bool {
        dividend.rem_euclid(divisor) == 0
    }

    fn compare_remainder(dividend: Self, divisor: Self) -> Option<Ordering> {
        Some(dividend.rem_euclid(divisor).cmp(&divisor.div_euclid(2)))
    }

    fn is_even_cardinal(dividend: Self, divisor: Self) -> bool {
        Roundable::result_floor(dividend, divisor).rem_euclid(2) == 0
    }

    fn result_floor(dividend: Self, divisor: Self) -> u64 {
        Roundable::quotient_abs(dividend, divisor) as u64
    }

    fn result_ceil(dividend: Self, divisor: Self) -> u64 {
        Roundable::quotient_abs(dividend, divisor) as u64 + 1
    }
}

impl Roundable for f64 {
    fn is_exact(dividend: Self, divisor: Self) -> bool {
        Roundable::quotient_abs(dividend, divisor)
            == Roundable::quotient_abs(dividend, divisor).floor()
    }

    fn compare_remainder(dividend: Self, divisor: Self) -> Option<Ordering> {
        let quotient = Roundable::quotient_abs(dividend, divisor);
        let d1 = quotient - quotient.floor();
        let d2 = quotient.ceil() - quotient;
        d1.partial_cmp(&d2)
    }

    fn is_even_cardinal(dividend: Self, divisor: Self) -> bool {
        let quotient = Roundable::quotient_abs(dividend, divisor);
        (quotient.floor() / (quotient.ceil() - quotient.floor()) % 2.0) == 0.0
    }

    fn result_floor(dividend: Self, divisor: Self) -> u64 {
        Roundable::quotient_abs(dividend, divisor).floor() as u64
    }

    fn result_ceil(dividend: Self, divisor: Self) -> u64 {
        Roundable::quotient_abs(dividend, divisor).ceil() as u64
    }
}

/// Applies the unsigned rounding mode.
fn apply_unsigned_rounding_mode<T: Roundable>(
    dividend: T,
    divisor: T,
    unsigned_rounding_mode: TemporalUnsignedRoundingMode,
) -> u64 {
    // is_floor
    // 1. If x is equal to r1, return r1.
    if Roundable::is_exact(dividend, divisor) {
        return Roundable::result_floor(dividend, divisor);
    }
    // 2. Assert: r1 < x < r2.
    // 3. Assert: unsignedRoundingMode is not undefined.

    // 4. If unsignedRoundingMode is zero, return r1.
    if unsigned_rounding_mode == TemporalUnsignedRoundingMode::Zero {
        return Roundable::result_floor(dividend, divisor);
    };
    // 5. If unsignedRoundingMode is infinity, return r2.
    if unsigned_rounding_mode == TemporalUnsignedRoundingMode::Infinity {
        return Roundable::result_ceil(dividend, divisor);
    };

    // 6. Let d1 be x – r1.
    // 7. Let d2 be r2 – x.
    // 8. If d1 < d2, return r1.
    // 9. If d2 < d1, return r2.
    match Roundable::compare_remainder(dividend, divisor) {
        Some(Ordering::Less) => Roundable::result_floor(dividend, divisor),
        Some(Ordering::Greater) => Roundable::result_ceil(dividend, divisor),
        Some(Ordering::Equal) => {
            // 10. Assert: d1 is equal to d2.
            // 11. If unsignedRoundingMode is half-zero, return r1.
            if unsigned_rounding_mode == TemporalUnsignedRoundingMode::HalfZero {
                return Roundable::result_floor(dividend, divisor);
            };
            // 12. If unsignedRoundingMode is half-infinity, return r2.
            if unsigned_rounding_mode == TemporalUnsignedRoundingMode::HalfInfinity {
                return Roundable::result_ceil(dividend, divisor);
            };
            // 13. Assert: unsignedRoundingMode is half-even.
            assert!(unsigned_rounding_mode == TemporalUnsignedRoundingMode::HalfEven);
            // 14. Let cardinality be (r1 / (r2 – r1)) modulo 2.
            // 15. If cardinality is 0, return r1.
            if Roundable::is_even_cardinal(dividend, divisor) {
                return Roundable::result_floor(dividend, divisor);
            }
            // 16. Return r2.
            Roundable::result_ceil(dividend, divisor)
        }
        None => unreachable!(),
    }
}

#[cfg(test)]
mod tests {
    use super::{IncrementRounder, Round, TemporalRoundingMode};

    #[test]
    fn basic_f64_rounding() {
        let result = IncrementRounder::<f64>::from_positive_parts(2.5, 1.0)
            .round_as_positive(TemporalRoundingMode::Floor);
        assert_eq!(result, 2);

        let result = IncrementRounder::<f64>::from_positive_parts(2.5, 1.0)
            .round_as_positive(TemporalRoundingMode::Ceil);
        assert_eq!(result, 3);

        let result = IncrementRounder::<f64>::from_positive_parts(7.5, 3.0)
            .round_as_positive(TemporalRoundingMode::HalfEven);
        assert_eq!(result, 6);

        let result = IncrementRounder::<f64>::from_positive_parts(10.5, 3.0)
            .round_as_positive(TemporalRoundingMode::HalfEven);
        assert_eq!(result, 12);
    }

    #[test]
    fn basic_i128_rounding() {
        let result = IncrementRounder::<i128>::from_positive_parts(5, 2)
            .round_as_positive(TemporalRoundingMode::Floor);
        assert_eq!(result, 4);

        let result = IncrementRounder::<i128>::from_positive_parts(5, 2)
            .round_as_positive(TemporalRoundingMode::Ceil);
        assert_eq!(result, 6);

        let result = IncrementRounder::<i128>::from_positive_parts(15, 7)
            .round_as_positive(TemporalRoundingMode::HalfEven);
        assert_eq!(result, 14);

        let result = IncrementRounder::<i128>::from_positive_parts(27, 13)
            .round_as_positive(TemporalRoundingMode::HalfEven);
        assert_eq!(result, 26);

        let result = IncrementRounder::<i128>::from_positive_parts(20, 7)
            .round_as_positive(TemporalRoundingMode::HalfEven);
        assert_eq!(result, 21);

        let result = IncrementRounder::<i128>::from_positive_parts(37, 13)
            .round_as_positive(TemporalRoundingMode::HalfEven);
        assert_eq!(result, 39);
    }

    #[test]
    fn neg_i128_rounding() {
        let result = IncrementRounder::<i128>::from_potentially_negative_parts(-9, 2)
            .round(TemporalRoundingMode::Ceil);
        assert_eq!(result, -8);

        let result = IncrementRounder::<i128>::from_potentially_negative_parts(-9, 2)
            .round(TemporalRoundingMode::Floor);
        assert_eq!(result, -10);
    }

    #[test]
    fn neg_f64_rounding() {
        let result = IncrementRounder::<f64>::from_potentially_negative_parts(-8.5, 1.0)
            .round(TemporalRoundingMode::Ceil);
        assert_eq!(result, -8);

        let result = IncrementRounder::<f64>::from_potentially_negative_parts(-8.5, 1.0)
            .round(TemporalRoundingMode::Floor);
        assert_eq!(result, -9);
    }
}
