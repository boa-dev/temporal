

use crate::options::{TemporalRoundingMode, TemporalUnsignedRoundingMode};

use std::{
    cmp::Ordering,
    ops::{Div, Neg},
};

use num_traits::{ConstZero, Euclid, FromPrimitive, Signed};

trait Roundable: PartialOrd {
    fn is_exact(&self) -> bool;
    fn cmp_rem(&self) -> Option<Ordering>;
    fn is_cardinal(&self) -> bool;
    fn result_floor(&self) -> u64;
    fn result_ceil(&self) -> u64;
}

pub(crate) trait Round {
    fn round(&self, mode: TemporalRoundingMode) -> i128;
    fn round_as_positive(&self, mode: TemporalRoundingMode) -> u64;
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub(crate) struct IncrementRounder<
    T: Euclid + Div + PartialOrd + Signed + FromPrimitive + ConstZero + Copy,
> {
    sign: bool,
    dividend: T,
    divisor: T,
}

impl<T: Euclid + PartialOrd + Signed + FromPrimitive + ConstZero + Copy> IncrementRounder<T> {
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

    // ==== PRIVATE ====

    #[inline]
    fn quotient_abs(&self) -> T {
        (self.dividend / self.divisor).abs()
    }
}

impl Round for IncrementRounder<i128> {
    fn round(&self, mode: TemporalRoundingMode) -> i128 {
        let unsigned_rounding_mode = mode.get_unsigned_round_mode(self.sign);
        let mut rounded = apply_unsigned_rounding_mode(self, unsigned_rounding_mode) as i128;
        if !self.sign {
            rounded = rounded.neg();
        }
        rounded * self.divisor
    }

    fn round_as_positive(&self, mode: TemporalRoundingMode) -> u64 {
        let unsigned_rounding_mode = mode.get_unsigned_round_mode(self.sign);
        let rounded = apply_unsigned_rounding_mode(self, unsigned_rounding_mode);
        rounded * self.divisor as u64
    }
}

impl Roundable for IncrementRounder<i128> {
    fn is_exact(&self) -> bool {
        self.dividend.rem_euclid(self.divisor) == 0
    }

    fn cmp_rem(&self) -> Option<Ordering> {
        Some(
            self.dividend
                .rem_euclid(self.divisor)
                .cmp(&self.divisor.div_euclid(2)),
        )
    }

    fn result_floor(&self) -> u64 {
        // NOTE: Sanity debugs until proper unit tests to vet the below
        debug_assert!(self.quotient_abs() < u64::MAX as i128);
        self.quotient_abs() as u64
    }

    fn result_ceil(&self) -> u64 {
        // NOTE: Sanity debugs until proper unit tests to vet the below
        debug_assert!(self.quotient_abs() < u64::MAX as i128);
        self.quotient_abs() as u64 + 1
    }

    fn is_cardinal(&self) -> bool {
        self.result_floor().rem_euclid(2) == 0
    }
}

impl Round for IncrementRounder<f64> {
    fn round(&self, mode: TemporalRoundingMode) -> i128 {
        let unsigned_rounding_mode = mode.get_unsigned_round_mode(self.sign);
        let mut rounded = apply_unsigned_rounding_mode(self, unsigned_rounding_mode) as i128;
        if !self.sign {
            rounded = rounded.neg();
        }
        rounded * self.divisor as i128
    }

    fn round_as_positive(&self, mode: TemporalRoundingMode) -> u64 {
        let unsigned_rounding_mode = mode.get_unsigned_round_mode(self.sign);
        let rounded = apply_unsigned_rounding_mode(self, unsigned_rounding_mode);
        rounded * self.divisor as u64
    }
}

impl Roundable for IncrementRounder<f64> {
    fn is_exact(&self) -> bool {
        self.quotient_abs() == self.quotient_abs().floor()
    }

    fn cmp_rem(&self) -> Option<Ordering> {
        let d1 = self.quotient_abs() - self.quotient_abs().floor();
        let d2 = self.quotient_abs().ceil() - self.quotient_abs();
        d1.partial_cmp(&d2)
    }

    fn is_cardinal(&self) -> bool {
        (self.quotient_abs().floor() / (self.quotient_abs().ceil() - self.quotient_abs().floor())
            % 2.0)
            == 0.0
    }

    fn result_floor(&self) -> u64 {
        self.quotient_abs().floor() as u64
    }

    fn result_ceil(&self) -> u64 {
        self.quotient_abs().ceil() as u64
    }
}

/// Applies the unsigned rounding mode.
fn apply_unsigned_rounding_mode<T: Roundable>(
    roundable: &T,
    unsigned_rounding_mode: TemporalUnsignedRoundingMode,
) -> u64 {
    // is_floor
    // 1. If x is equal to r1, return r1.
    if roundable.is_exact() {
        return roundable.result_floor();
    }
    // 2. Assert: r1 < x < r2.
    // 3. Assert: unsignedRoundingMode is not undefined.

    // 4. If unsignedRoundingMode is zero, return r1.
    if unsigned_rounding_mode == TemporalUnsignedRoundingMode::Zero {
        return roundable.result_floor();
    };
    // 5. If unsignedRoundingMode is infinity, return r2.
    if unsigned_rounding_mode == TemporalUnsignedRoundingMode::Infinity {
        return roundable.result_ceil();
    };

    // 6. Let d1 be x – r1.
    // 7. Let d2 be r2 – x.
    // 8. If d1 < d2, return r1.
    // 9. If d2 < d1, return r2.
    match roundable.cmp_rem() {
        Some(Ordering::Less) => roundable.result_floor(),
        Some(Ordering::Greater) => roundable.result_ceil(),
        Some(Ordering::Equal) => {
            // 10. Assert: d1 is equal to d2.
            // 11. If unsignedRoundingMode is half-zero, return r1.
            if unsigned_rounding_mode == TemporalUnsignedRoundingMode::HalfZero {
                return roundable.result_floor();
            };
            // 12. If unsignedRoundingMode is half-infinity, return r2.
            if unsigned_rounding_mode == TemporalUnsignedRoundingMode::HalfInfinity {
                return roundable.result_ceil();
            };
            // 13. Assert: unsignedRoundingMode is half-even.
            assert!(unsigned_rounding_mode == TemporalUnsignedRoundingMode::HalfEven);
            // 14. Let cardinality be (r1 / (r2 – r1)) modulo 2.
            // 15. If cardinality is 0, return r1.
            if roundable.is_cardinal() {
                return roundable.result_floor();
            }
            // 16. Return r2.
            roundable.result_ceil()
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
