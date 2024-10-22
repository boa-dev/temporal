//! An implementation of the Temporal Instant.

use std::{num::NonZeroU128, str::FromStr};

use crate::{
    components::{duration::TimeDuration, Duration},
    iso::{IsoDate, IsoDateTime, IsoTime},
    options::{
        ArithmeticOverflow, DifferenceOperation, DifferenceSettings, ResolvedRoundingOptions,
        RoundingOptions, TemporalUnit,
    },
    parsers::parse_instant,
    primitive::FiniteF64,
    rounding::{IncrementRounder, Round},
    Sign, TemporalError, TemporalResult, TemporalUnwrap,
};

use num_traits::{Euclid, FromPrimitive, ToPrimitive};

use super::duration::normalized::NormalizedTimeDuration;

const NANOSECONDS_PER_SECOND: f64 = 1e9;
const NANOSECONDS_PER_MINUTE: f64 = 60f64 * NANOSECONDS_PER_SECOND;
const NANOSECONDS_PER_HOUR: f64 = 60f64 * NANOSECONDS_PER_MINUTE;

/// The native Rust implementation of `Temporal.Instant`
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Instant {
    pub(crate) epoch_nanos: i128,
}

// ==== Private API ====

impl Instant {
    // TODO: Update to `i128`?
    /// Adds a `TimeDuration` to the current `Instant`.
    ///
    /// Temporal-Proposal equivalent: `AddDurationToOrSubtractDurationFrom`.
    pub(crate) fn add_to_instant(&self, duration: &TimeDuration) -> TemporalResult<Self> {
        let result = self.epoch_nanoseconds()
            + duration.nanoseconds.0
            + (duration.microseconds.0 * 1000f64)
            + (duration.milliseconds.0 * 1_000_000f64)
            + (duration.seconds.0 * NANOSECONDS_PER_SECOND)
            + (duration.minutes.0 * NANOSECONDS_PER_MINUTE)
            + (duration.hours.0 * NANOSECONDS_PER_HOUR);
        let nanos = i128::from_f64(result).ok_or_else(|| {
            TemporalError::range().with_message("Duration added to instant exceeded valid range.")
        })?;
        Self::new(nanos)
    }

    // TODO: Add test for `diff_instant`.
    // NOTE(nekevss): As the below is internal, op will be left as a boolean
    // with a `since` op being true and `until` being false.
    /// Internal operation to handle `since` and `until` difference ops.
    #[allow(unused)]
    pub(crate) fn diff_instant(
        &self,
        op: DifferenceOperation,
        other: &Self,
        options: DifferenceSettings,
    ) -> TemporalResult<TimeDuration> {
        // 1. If operation is since, let sign be -1. Otherwise, let sign be 1.
        // 2. Set other to ? ToTemporalInstant(other).
        // 3. Let resolvedOptions be ? SnapshotOwnProperties(? GetOptionsObject(options), null).
        // 4. Let settings be ? GetDifferenceSettings(operation, resolvedOptions, time, « », "nanosecond", "second").
        let (sign, resolved_options) = ResolvedRoundingOptions::from_diff_settings(
            options,
            op,
            TemporalUnit::Second,
            TemporalUnit::Nanosecond,
        )?;

        // Below are the steps from Difference Instant.
        // 5. Let diffRecord be DifferenceInstant(instant.[[Nanoseconds]], other.[[Nanoseconds]],
        // settings.[[RoundingIncrement]], settings.[[SmallestUnit]], settings.[[RoundingMode]]).
        let diff = NormalizedTimeDuration::from_nanosecond_difference(
            other.epoch_nanos,
            self.epoch_nanos,
        )?;
        let (round_record, _) = diff.round(FiniteF64::default(), resolved_options)?;

        // 6. Let norm be diffRecord.[[NormalizedTimeDuration]].
        // 7. Let result be ! BalanceTimeDuration(norm, settings.[[LargestUnit]]).
        let (_, result) = TimeDuration::from_normalized(
            round_record.normalized_time_duration(),
            resolved_options.largest_unit,
        )?;

        // 8. Return ! CreateTemporalDuration(0, 0, 0, 0, sign × result.[[Hours]], sign × result.[[Minutes]], sign × result.[[Seconds]], sign × result.[[Milliseconds]], sign × result.[[Microseconds]], sign × result.[[Nanoseconds]]).
        match sign {
            Sign::Positive | Sign::Zero => Ok(result),
            Sign::Negative => Ok(result.negated()),
        }
    }

    /// Rounds a current `Instant` given the resolved options, returning a `BigInt` result.
    pub(crate) fn round_instant(
        &self,
        resolved_options: ResolvedRoundingOptions,
    ) -> TemporalResult<i128> {
        let increment = resolved_options.increment.as_extended_increment();
        let increment = match resolved_options.smallest_unit {
            TemporalUnit::Hour => increment
                .checked_mul(NonZeroU128::new(NANOSECONDS_PER_HOUR as u128).temporal_unwrap()?),
            TemporalUnit::Minute => increment
                .checked_mul(NonZeroU128::new(NANOSECONDS_PER_MINUTE as u128).temporal_unwrap()?),
            TemporalUnit::Second => increment
                .checked_mul(NonZeroU128::new(NANOSECONDS_PER_SECOND as u128).temporal_unwrap()?),
            TemporalUnit::Millisecond => {
                increment.checked_mul(NonZeroU128::new(1_000_000).temporal_unwrap()?)
            }
            TemporalUnit::Microsecond => {
                increment.checked_mul(NonZeroU128::new(1_000).temporal_unwrap()?)
            }
            TemporalUnit::Nanosecond => Some(increment),
            _ => {
                return Err(TemporalError::range()
                    .with_message("Invalid unit provided for Instant::round."))
            }
        };

        // NOTE: Potentially remove the below and just `temporal_unwrap`
        let Some(increment) = increment else {
            return Err(TemporalError::range().with_message("Increment exceeded a valid range."));
        };

        let rounded = IncrementRounder::<i128>::from_positive_parts(self.epoch_nanos, increment)?
            .round_as_positive(resolved_options.rounding_mode);

        Ok(rounded.into())
    }

    /// Utility for converting `Instant` to f64.
    ///
    /// # Panics
    ///
    /// This function will panic if called on an invalid `Instant`.
    pub(crate) fn to_f64(&self) -> f64 {
        self.epoch_nanos
            .to_f64()
            .expect("A valid instant is representable by f64.")
    }
}

// ==== Public API ====

impl Instant {
    /// Create a new validated `Instant`.
    #[inline]
    pub fn new(epoch_nanoseconds: i128) -> TemporalResult<Self> {
        if !is_valid_epoch_nanos(&epoch_nanoseconds) {
            return Err(TemporalError::range()
                .with_message("Instant nanoseconds are not within a valid epoch range."));
        }
        Ok(Self {
            epoch_nanos: epoch_nanoseconds,
        })
    }

    pub fn from_epoch_milliseconds(epoch_milliseconds: i128) -> TemporalResult<Self> {
        let epoch_nanos = epoch_milliseconds
            .checked_mul(1_000_000)
            .unwrap_or(i128::MAX);
        if !is_valid_epoch_nanos(&epoch_nanos) {
            return Err(TemporalError::range()
                .with_message("Instant nanoseconds are not within a valid epoch range."));
        }
        Ok(Self { epoch_nanos })
    }

    /// Adds a `Duration` to the current `Instant`, returning an error if the `Duration`
    /// contains a `DateDuration`.
    #[inline]
    pub fn add(&self, duration: Duration) -> TemporalResult<Self> {
        if !duration.is_time_duration() {
            return Err(TemporalError::range()
                .with_message("DateDuration values cannot be added to instant."));
        }
        self.add_time_duration(duration.time())
    }

    /// Adds a `TimeDuration` to `Instant`.
    #[inline]
    pub fn add_time_duration(&self, duration: &TimeDuration) -> TemporalResult<Self> {
        self.add_to_instant(duration)
    }

    /// Subtract a `Duration` to the current `Instant`, returning an error if the `Duration`
    /// contains a `DateDuration`.
    #[inline]
    pub fn subtract(&self, duration: Duration) -> TemporalResult<Self> {
        if !duration.is_time_duration() {
            return Err(TemporalError::range()
                .with_message("DateDuration values cannot be added to instant."));
        }
        self.subtract_time_duration(duration.time())
    }

    /// Subtracts a `TimeDuration` to `Instant`.
    #[inline]
    pub fn subtract_time_duration(&self, duration: &TimeDuration) -> TemporalResult<Self> {
        self.add_to_instant(&duration.negated())
    }

    /// Returns a `TimeDuration` representing the duration since provided `Instant`
    #[inline]
    pub fn since(
        &self,
        other: &Self,
        settings: DifferenceSettings,
    ) -> TemporalResult<TimeDuration> {
        self.diff_instant(DifferenceOperation::Since, other, settings)
    }

    /// Returns a `TimeDuration` representing the duration until provided `Instant`
    #[inline]
    pub fn until(
        &self,
        other: &Self,
        settings: DifferenceSettings,
    ) -> TemporalResult<TimeDuration> {
        self.diff_instant(DifferenceOperation::Until, other, settings)
    }

    /// Returns an `Instant` by rounding the current `Instant` according to the provided settings.
    pub fn round(&self, options: RoundingOptions) -> TemporalResult<Self> {
        let resolved_options = ResolvedRoundingOptions::from_instant_options(options)?;

        let round_result = self.round_instant(resolved_options)?;
        Self::new(round_result)
    }

    /// Returns the `epochSeconds` value for this `Instant`.
    #[must_use]
    pub fn epoch_seconds(&self) -> f64 {
        (&self.epoch_nanos / 1_000_000_000)
            .to_f64()
            .expect("A validated Instant should be within a valid f64")
            .floor()
    }

    /// Returns the `epochMilliseconds` value for this `Instant`.
    #[must_use]
    pub fn epoch_milliseconds(&self) -> f64 {
        (&self.epoch_nanos / 1_000_000)
            .to_f64()
            .expect("A validated Instant should be within a valid f64")
            .floor()
    }

    /// Returns the `epochMicroseconds` value for this `Instant`.
    #[must_use]
    pub fn epoch_microseconds(&self) -> f64 {
        (&self.epoch_nanos / 1_000)
            .to_f64()
            .expect("A validated Instant should be within a valid f64")
            .floor()
    }

    /// Returns the `epochNanoseconds` value for this `Instant`.
    #[must_use]
    pub fn epoch_nanoseconds(&self) -> f64 {
        self.to_f64()
    }
}

// ==== Utility Functions ====

/// Utility for determining if the nanos are within a valid range.
#[inline]
#[must_use]
pub(crate) fn is_valid_epoch_nanos(nanos: &i128) -> bool {
    (crate::NS_MIN_INSTANT..=crate::NS_MAX_INSTANT).contains(nanos)
}

impl FromStr for Instant {
    type Err = TemporalError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let ixdtf_record = parse_instant(s)?;

        // Find the IsoDate
        let iso_date = IsoDate::new_with_overflow(
            ixdtf_record.date.year,
            ixdtf_record.date.month.into(),
            ixdtf_record.date.day.into(),
            ArithmeticOverflow::Reject,
        )?;

        // Find the IsoTime
        let (millisecond, remainder) = ixdtf_record.time.nanosecond.div_rem_euclid(&1_000_000);
        let (microsecond, nanosecond) = remainder.div_rem_euclid(&1_000);
        let iso_time = IsoTime::new(
            ixdtf_record.time.hour.into(),
            ixdtf_record.time.minute.into(),
            ixdtf_record.time.second.into(),
            millisecond as i32,
            microsecond as i32,
            nanosecond as i32,
            ArithmeticOverflow::Reject,
        )?;

        // Find the offset
        let offset = f64::from(ixdtf_record.offset.hour) * NANOSECONDS_PER_HOUR
            + f64::from(ixdtf_record.offset.minute) * NANOSECONDS_PER_MINUTE
            + f64::from(ixdtf_record.offset.second) * NANOSECONDS_PER_SECOND
            + f64::from(ixdtf_record.offset.nanosecond);

        let nanoseconds = IsoDateTime::new_unchecked(iso_date, iso_time)
            .as_nanoseconds()
            .map(|v| v + offset as i128);

        Self::from_epoch_milliseconds(nanoseconds.unwrap_or(i128::MAX))
    }
}

// ==== Instant Tests ====

#[cfg(test)]
mod tests {
    use crate::{
        components::{duration::TimeDuration, Instant},
        options::{DifferenceSettings, TemporalRoundingMode, TemporalUnit},
        primitive::FiniteF64,
        NS_MAX_INSTANT, NS_MIN_INSTANT,
    };
    use num_traits::ToPrimitive;

    #[test]
    #[allow(clippy::float_cmp)]
    fn max_and_minimum_instant_bounds() {
        // This test is primarily to assert that the `expect` in the epoch methods is
        // valid, i.e., a valid instant is within the range of an f64.
        let max = NS_MAX_INSTANT;
        let min = NS_MIN_INSTANT;
        let max_instant = Instant::new(max).unwrap();
        let min_instant = Instant::new(min).unwrap();

        assert_eq!(max_instant.epoch_nanoseconds(), max.to_f64().unwrap());
        assert_eq!(min_instant.epoch_nanoseconds(), min.to_f64().unwrap());

        let max_plus_one = NS_MAX_INSTANT + 1;
        let min_minus_one = NS_MIN_INSTANT - 1;

        assert!(Instant::new(max_plus_one).is_err());
        assert!(Instant::new(min_minus_one).is_err());
    }

    #[test]
    fn basic_instant_until() {
        let init_diff_setting = |unit: TemporalUnit| -> DifferenceSettings {
            DifferenceSettings {
                largest_unit: Some(TemporalUnit::Hour),
                rounding_mode: Some(TemporalRoundingMode::Ceil),
                increment: None,
                smallest_unit: Some(unit),
            }
        };

        let assert_time_duration = |td: TimeDuration, expected: (f64, f64, f64, f64, f64, f64)| {
            assert_eq!(
                td,
                TimeDuration {
                    hours: FiniteF64(expected.0),
                    minutes: FiniteF64(expected.1),
                    seconds: FiniteF64(expected.2),
                    milliseconds: FiniteF64(expected.3),
                    microseconds: FiniteF64(expected.4),
                    nanoseconds: FiniteF64(expected.5),
                }
            )
        };

        let earlier = Instant::new(
            217_178_610_123_456_789, /* 1976-11-18T15:23:30.123456789Z */
        )
        .unwrap();
        let later = Instant::new(
            1_572_345_998_271_986_289, /* 2019-10-29T10:46:38.271986289Z */
        )
        .unwrap();

        let positive_result = earlier
            .until(&later, init_diff_setting(TemporalUnit::Hour))
            .unwrap();
        assert_time_duration(positive_result, (376436.0, 0.0, 0.0, 0.0, 0.0, 0.0));
        let negative_result = later
            .until(&earlier, init_diff_setting(TemporalUnit::Hour))
            .unwrap();
        assert_time_duration(negative_result, (-376435.0, 0.0, 0.0, 0.0, 0.0, 0.0));

        let positive_result = earlier
            .until(&later, init_diff_setting(TemporalUnit::Minute))
            .unwrap();
        assert_time_duration(positive_result, (376435.0, 24.0, 0.0, 0.0, 0.0, 0.0));
        let negative_result = later
            .until(&earlier, init_diff_setting(TemporalUnit::Minute))
            .unwrap();
        assert_time_duration(negative_result, (-376435.0, -23.0, 0.0, 0.0, 0.0, 0.0));

        // ... Skip to lower units ...

        let positive_result = earlier
            .until(&later, init_diff_setting(TemporalUnit::Microsecond))
            .unwrap();
        assert_time_duration(positive_result, (376435.0, 23.0, 8.0, 148.0, 530.0, 0.0));
        let negative_result = later
            .until(&earlier, init_diff_setting(TemporalUnit::Microsecond))
            .unwrap();
        assert_time_duration(
            negative_result,
            (-376435.0, -23.0, -8.0, -148.0, -529.0, 0.0),
        );

        let positive_result = earlier
            .until(&later, init_diff_setting(TemporalUnit::Nanosecond))
            .unwrap();
        assert_time_duration(positive_result, (376435.0, 23.0, 8.0, 148.0, 529.0, 500.0));
        let negative_result = later
            .until(&earlier, init_diff_setting(TemporalUnit::Nanosecond))
            .unwrap();
        assert_time_duration(
            negative_result,
            (-376435.0, -23.0, -8.0, -148.0, -529.0, -500.0),
        );
    }

    #[test]
    fn basic_instant_since() {
        let init_diff_setting = |unit: TemporalUnit| -> DifferenceSettings {
            DifferenceSettings {
                largest_unit: Some(TemporalUnit::Hour),
                rounding_mode: Some(TemporalRoundingMode::Ceil),
                increment: None,
                smallest_unit: Some(unit),
            }
        };

        let assert_time_duration = |td: TimeDuration, expected: (f64, f64, f64, f64, f64, f64)| {
            assert_eq!(
                td,
                TimeDuration {
                    hours: FiniteF64(expected.0),
                    minutes: FiniteF64(expected.1),
                    seconds: FiniteF64(expected.2),
                    milliseconds: FiniteF64(expected.3),
                    microseconds: FiniteF64(expected.4),
                    nanoseconds: FiniteF64(expected.5),
                }
            )
        };

        let earlier = Instant::new(
            217_178_610_123_456_789, /* 1976-11-18T15:23:30.123456789Z */
        )
        .unwrap();
        let later = Instant::new(
            1_572_345_998_271_986_289, /* 2019-10-29T10:46:38.271986289Z */
        )
        .unwrap();

        let positive_result = later
            .since(&earlier, init_diff_setting(TemporalUnit::Hour))
            .unwrap();
        assert_time_duration(positive_result, (376436.0, 0.0, 0.0, 0.0, 0.0, 0.0));
        let negative_result = earlier
            .since(&later, init_diff_setting(TemporalUnit::Hour))
            .unwrap();
        assert_time_duration(negative_result, (-376435.0, 0.0, 0.0, 0.0, 0.0, 0.0));

        let positive_result = later
            .since(&earlier, init_diff_setting(TemporalUnit::Minute))
            .unwrap();
        assert_time_duration(positive_result, (376435.0, 24.0, 0.0, 0.0, 0.0, 0.0));
        let negative_result = earlier
            .since(&later, init_diff_setting(TemporalUnit::Minute))
            .unwrap();
        assert_time_duration(negative_result, (-376435.0, -23.0, 0.0, 0.0, 0.0, 0.0));

        // ... Skip to lower units ...

        let positive_result = later
            .since(&earlier, init_diff_setting(TemporalUnit::Microsecond))
            .unwrap();
        assert_time_duration(positive_result, (376435.0, 23.0, 8.0, 148.0, 530.0, 0.0));
        let negative_result = earlier
            .since(&later, init_diff_setting(TemporalUnit::Microsecond))
            .unwrap();
        assert_time_duration(
            negative_result,
            (-376435.0, -23.0, -8.0, -148.0, -529.0, 0.0),
        );

        let positive_result = later
            .since(&earlier, init_diff_setting(TemporalUnit::Nanosecond))
            .unwrap();
        assert_time_duration(positive_result, (376435.0, 23.0, 8.0, 148.0, 529.0, 500.0));
        let negative_result = earlier
            .since(&later, init_diff_setting(TemporalUnit::Nanosecond))
            .unwrap();
        assert_time_duration(
            negative_result,
            (-376435.0, -23.0, -8.0, -148.0, -529.0, -500.0),
        );
    }
}
