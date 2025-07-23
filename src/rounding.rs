//! Implementation of increment rounding functionality

use crate::{
    options::{RoundingMode, UnsignedRoundingMode},
    TemporalResult, TemporalUnwrap,
};

use core::{
    cmp::Ordering,
    num::NonZeroU128,
    ops::{Div, Neg},
};

use num_traits::float::FloatCore;
use num_traits::{ConstZero, Euclid, FromPrimitive, NumCast, Signed, ToPrimitive};

pub(crate) trait Roundable:
    Euclid + Div + PartialOrd + Signed + FromPrimitive + ToPrimitive + NumCast + ConstZero + Copy
{
    fn is_exact(dividend: Self, divisor: Self) -> bool;
    fn compare_remainder(dividend: Self, divisor: Self) -> Option<Ordering>;
    fn is_even_cardinal(dividend: Self, divisor: Self) -> bool;
    fn result_floor(dividend: Self, divisor: Self) -> u128;
    fn result_ceil(dividend: Self, divisor: Self) -> u128;
    fn quotient_abs(dividend: Self, divisor: Self) -> Self {
        // NOTE: Sanity debugs until proper unit tests to vet the below
        // NOTE (nekevss): quotient_abs exceeds the range of u64, so u128 must be used.
        debug_assert!(<u128 as NumCast>::from((dividend / divisor).abs()) < Some(u128::MAX));
        (dividend / divisor).abs()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub(crate) struct IncrementRounder<T: Roundable> {
    sign: bool,
    dividend: T,
    divisor: T,
}

impl<T: Roundable> IncrementRounder<T> {
    #[inline]
    pub(crate) fn from_signed_num(number: T, increment: NonZeroU128) -> TemporalResult<Self> {
        let increment = <T as NumCast>::from(increment.get()).temporal_unwrap()?;
        Ok(Self {
            sign: number >= T::ZERO,
            dividend: number,
            divisor: increment,
        })
    }
}

impl<T: Roundable> IncrementRounder<T> {
    #[inline]
    pub fn round(&self, mode: RoundingMode) -> i128 {
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
}

impl Roundable for i128 {
    fn is_exact(dividend: Self, divisor: Self) -> bool {
        dividend.rem_euclid(divisor) == 0
    }

    fn compare_remainder(dividend: Self, divisor: Self) -> Option<Ordering> {
        Some((dividend.abs() % divisor).cmp(&(divisor / 2)))
    }

    fn is_even_cardinal(dividend: Self, divisor: Self) -> bool {
        Roundable::result_floor(dividend, divisor).rem_euclid(2) == 0
    }

    fn result_floor(dividend: Self, divisor: Self) -> u128 {
        Roundable::quotient_abs(dividend, divisor) as u128
    }

    fn result_ceil(dividend: Self, divisor: Self) -> u128 {
        Roundable::quotient_abs(dividend, divisor) as u128 + 1
    }
}

impl Roundable for f64 {
    fn is_exact(dividend: Self, divisor: Self) -> bool {
        Roundable::quotient_abs(dividend, divisor)
            == Roundable::quotient_abs(dividend, divisor).floor()
    }

    fn compare_remainder(dividend: Self, divisor: Self) -> Option<Ordering> {
        let quotient = Roundable::quotient_abs(dividend, divisor);
        let d1 = quotient - FloatCore::floor(quotient);
        let d2 = FloatCore::ceil(quotient) - quotient;
        d1.partial_cmp(&d2)
    }

    fn is_even_cardinal(dividend: Self, divisor: Self) -> bool {
        let quotient = Roundable::quotient_abs(dividend, divisor);
        (FloatCore::floor(quotient) / (FloatCore::ceil(quotient) - FloatCore::floor(quotient))
            % 2.0)
            == 0.0
    }

    fn result_floor(dividend: Self, divisor: Self) -> u128 {
        Roundable::quotient_abs(dividend, divisor).floor() as u128
    }

    fn result_ceil(dividend: Self, divisor: Self) -> u128 {
        Roundable::quotient_abs(dividend, divisor).ceil() as u128
    }
}

impl Roundable for i64 {
    fn is_exact(dividend: Self, divisor: Self) -> bool {
        dividend.rem_euclid(divisor) == 0
    }

    fn compare_remainder(dividend: Self, divisor: Self) -> Option<Ordering> {
        Some((dividend.abs() % divisor).cmp(&(divisor / 2)))
    }

    fn is_even_cardinal(dividend: Self, divisor: Self) -> bool {
        Roundable::result_floor(dividend, divisor).rem_euclid(2) == 0
    }

    fn result_floor(dividend: Self, divisor: Self) -> u128 {
        Roundable::quotient_abs(dividend, divisor) as u128
    }

    fn result_ceil(dividend: Self, divisor: Self) -> u128 {
        Roundable::quotient_abs(dividend, divisor) as u128 + 1
    }
}

/// Applies the unsigned rounding mode.
fn apply_unsigned_rounding_mode<T: Roundable>(
    dividend: T,
    divisor: T,
    unsigned_rounding_mode: UnsignedRoundingMode,
) -> u128 {
    // is_floor
    // 1. If x is equal to r1, return r1.
    if Roundable::is_exact(dividend, divisor) {
        return Roundable::result_floor(dividend, divisor);
    }
    // 2. Assert: r1 < x < r2.
    // 3. Assert: unsignedRoundingMode is not undefined.

    // 4. If unsignedRoundingMode is zero, return r1.
    if unsigned_rounding_mode == UnsignedRoundingMode::Zero {
        return Roundable::result_floor(dividend, divisor);
    };
    // 5. If unsignedRoundingMode is infinity, return r2.
    if unsigned_rounding_mode == UnsignedRoundingMode::Infinity {
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
            if unsigned_rounding_mode == UnsignedRoundingMode::HalfZero {
                return Roundable::result_floor(dividend, divisor);
            };
            // 12. If unsignedRoundingMode is half-infinity, return r2.
            if unsigned_rounding_mode == UnsignedRoundingMode::HalfInfinity {
                return Roundable::result_ceil(dividend, divisor);
            };
            // 13. Assert: unsignedRoundingMode is half-even.
            debug_assert!(unsigned_rounding_mode == UnsignedRoundingMode::HalfEven);
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
    use core::num::NonZeroU128;

    use super::{IncrementRounder, Roundable, RoundingMode};
    use core::fmt::Debug;

    #[derive(Debug)]
    struct TestCase<T> {
        x: T,
        increment: u128,
        ceil: i128,
        floor: i128,
        expand: i128,
        trunc: i128,
        half_ceil: i128,
        half_floor: i128,
        half_expand: i128,
        half_trunc: i128,
        half_even: i128,
    }

    impl<T: Roundable + Debug> TestCase<T> {
        fn run(&self) {
            let rounder = IncrementRounder::from_signed_num(
                self.x,
                TryFrom::try_from(self.increment).unwrap(),
            )
            .unwrap();
            assert_eq!(
                self.ceil,
                rounder.round(RoundingMode::Ceil),
                "Testing {:?}/{:?} with mode Ceil",
                self.x,
                self.increment
            );
            assert_eq!(
                self.floor,
                rounder.round(RoundingMode::Floor),
                "Testing {:?}/{:?} with mode Floor",
                self.x,
                self.increment
            );
            assert_eq!(
                self.expand,
                rounder.round(RoundingMode::Expand),
                "Testing {:?}/{:?} with mode Expand",
                self.x,
                self.increment
            );
            assert_eq!(
                self.trunc,
                rounder.round(RoundingMode::Trunc),
                "Testing {:?}/{:?} with mode Trunc",
                self.x,
                self.increment
            );
            assert_eq!(
                self.half_ceil,
                rounder.round(RoundingMode::HalfCeil),
                "Testing {:?}/{:?} with mode HalfCeil",
                self.x,
                self.increment
            );
            assert_eq!(
                self.half_floor,
                rounder.round(RoundingMode::HalfFloor),
                "Testing {:?}/{:?} with mode HalfFloor",
                self.x,
                self.increment
            );
            assert_eq!(
                self.half_expand,
                rounder.round(RoundingMode::HalfExpand),
                "Testing {:?}/{:?} with mode HalfExpand",
                self.x,
                self.increment
            );
            assert_eq!(
                self.half_trunc,
                rounder.round(RoundingMode::HalfTrunc),
                "Testing {:?}/{:?} with mode HalfTrunc",
                self.x,
                self.increment
            );
            assert_eq!(
                self.half_even,
                rounder.round(RoundingMode::HalfEven),
                "Testing {:?}/{:?} with mode HalfEven",
                self.x,
                self.increment
            );
        }
    }

    #[test]
    fn test_basic_rounding_cases() {
        const CASES: &[TestCase<i128>] = &[
            TestCase {
                x: 100,
                increment: 10,
                ceil: 100,
                floor: 100,
                expand: 100,
                trunc: 100,
                half_ceil: 100,
                half_floor: 100,
                half_expand: 100,
                half_trunc: 100,
                half_even: 100,
            },
            TestCase {
                x: 101,
                increment: 10,
                ceil: 110,
                floor: 100,
                expand: 110,
                trunc: 100,
                half_ceil: 100,
                half_floor: 100,
                half_expand: 100,
                half_trunc: 100,
                half_even: 100,
            },
            TestCase {
                x: 105,
                increment: 10,
                ceil: 110,
                floor: 100,
                expand: 110,
                trunc: 100,
                half_ceil: 110,
                half_floor: 100,
                half_expand: 110,
                half_trunc: 100,
                half_even: 100,
            },
            TestCase {
                x: 107,
                increment: 10,
                ceil: 110,
                floor: 100,
                expand: 110,
                trunc: 100,
                half_ceil: 110,
                half_floor: 110,
                half_expand: 110,
                half_trunc: 110,
                half_even: 110,
            },
            TestCase {
                x: -100,
                increment: 10,
                ceil: -100,
                floor: -100,
                expand: -100,
                trunc: -100,
                half_ceil: -100,
                half_floor: -100,
                half_expand: -100,
                half_trunc: -100,
                half_even: -100,
            },
            TestCase {
                x: -101,
                increment: 10,
                ceil: -100,
                floor: -110,
                expand: -110,
                trunc: -100,
                half_ceil: -100,
                half_floor: -100,
                half_expand: -100,
                half_trunc: -100,
                half_even: -100,
            },
            TestCase {
                x: -105,
                increment: 10,
                ceil: -100,
                floor: -110,
                expand: -110,
                trunc: -100,
                half_ceil: -100,
                half_floor: -110,
                half_expand: -110,
                half_trunc: -100,
                half_even: -100,
            },
            TestCase {
                x: -107,
                increment: 10,
                ceil: -100,
                floor: -110,
                expand: -110,
                trunc: -100,
                half_ceil: -110,
                half_floor: -110,
                half_expand: -110,
                half_trunc: -110,
                half_even: -110,
            },
        ];

        for case in CASES {
            case.run();
        }
    }

    #[test]
    fn neg_i128_rounding() {
        TestCase {
            x: -9i128,
            increment: 2,
            ceil: -8,
            floor: -10,
            expand: -10,
            trunc: -8,
            half_ceil: -8,
            half_floor: -10,
            half_expand: -10,
            half_trunc: -8,
            half_even: -8,
        }
        .run();

        TestCase {
            x: -14i128,
            increment: 3,
            ceil: -12,
            floor: -15,
            expand: -15,
            trunc: -12,
            half_ceil: -15,
            half_floor: -15,
            half_expand: -15,
            half_trunc: -15,
            half_even: -15,
        }
        .run();
    }

    #[test]
    fn neg_f64_rounding() {
        TestCase {
            x: -8.5f64,
            increment: 1,
            ceil: -8,
            floor: -9,
            expand: -9,
            trunc: -8,
            half_ceil: -8,
            half_floor: -9,
            half_expand: -9,
            half_trunc: -8,
            half_even: -8,
        }
        .run();
    }

    #[test]
    fn dt_since_basic_rounding() {
        let result = IncrementRounder::<i128>::from_signed_num(
            -84082624864197532,
            NonZeroU128::new(1800000000000).unwrap(),
        )
        .unwrap()
        .round(RoundingMode::HalfExpand);

        assert_eq!(result, -84083400000000000);
    }
}
