//! An implementation of the Temporal Instant.

use alloc::string::String;
use core::{num::NonZeroU128, str::FromStr};

use crate::{
    builtins::core::{
        duration::TimeDuration, zoneddatetime::nanoseconds_to_formattable_offset_minutes, Duration,
    },
    iso::{IsoDate, IsoDateTime, IsoTime},
    options::{
        ArithmeticOverflow, DifferenceOperation, DifferenceSettings, DisplayOffset,
        ResolvedRoundingOptions, RoundingOptions, TemporalUnit, ToStringRoundingOptions,
    },
    parsers::{parse_instant, IxdtfStringBuilder},
    primitive::FiniteF64,
    rounding::{IncrementRounder, Round},
    time::EpochNanoseconds,
    Sign, TemporalError, TemporalResult, TemporalUnwrap, TimeZone,
};

use ixdtf::parsers::records::UtcOffsetRecordOrZ;

use super::{
    duration::normalized::{NormalizedDurationRecord, NormalizedTimeDuration},
    timezone::TimeZoneProvider,
};

const NANOSECONDS_PER_SECOND: f64 = 1e9;
const NANOSECONDS_PER_MINUTE: f64 = 60f64 * NANOSECONDS_PER_SECOND;
const NANOSECONDS_PER_HOUR: f64 = 60f64 * NANOSECONDS_PER_MINUTE;

/// The native Rust implementation of `Temporal.Instant`
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Instant(EpochNanoseconds);

impl From<EpochNanoseconds> for Instant {
    fn from(value: EpochNanoseconds) -> Self {
        Self(value)
    }
}

// ==== Private API ====

impl Instant {
    // TODO: Update to `i128`?
    /// Adds a `TimeDuration` to the current `Instant`.
    ///
    /// Temporal-Proposal equivalent: `AddDurationToOrSubtractDurationFrom`.
    pub(crate) fn add_to_instant(&self, duration: &TimeDuration) -> TemporalResult<Self> {
        let current_nanos = self.epoch_nanoseconds() as f64;
        let result = current_nanos
            + duration.nanoseconds.0
            + (duration.microseconds.0 * 1000f64)
            + (duration.milliseconds.0 * 1_000_000f64)
            + (duration.seconds.0 * NANOSECONDS_PER_SECOND)
            + (duration.minutes.0 * NANOSECONDS_PER_MINUTE)
            + (duration.hours.0 * NANOSECONDS_PER_HOUR);
        Ok(Self::from(EpochNanoseconds::try_from(result)?))
    }

    pub(crate) fn diff_instant_internal(
        &self,
        other: &Self,
        resolved_options: ResolvedRoundingOptions,
    ) -> TemporalResult<NormalizedDurationRecord> {
        let diff =
            NormalizedTimeDuration::from_nanosecond_difference(other.as_i128(), self.as_i128())?;
        let (round_record, _) = diff.round(FiniteF64::default(), resolved_options)?;
        Ok(round_record)
    }

    // TODO: Add test for `diff_instant`.
    // NOTE(nekevss): As the below is internal, op will be left as a boolean
    // with a `since` op being true and `until` being false.
    /// Internal operation to handle `since` and `until` difference ops.
    pub(crate) fn diff_instant(
        &self,
        op: DifferenceOperation,
        other: &Self,
        options: DifferenceSettings,
    ) -> TemporalResult<Duration> {
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
        let internal_record = self.diff_instant_internal(other, resolved_options)?;

        let result = Duration::from_normalized(internal_record, resolved_options.largest_unit)?;

        // 6. Let norm be diffRecord.[[NormalizedTimeDuration]].
        // 7. Let result be ! BalanceTimeDuration(norm, settings.[[LargestUnit]]).
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

        let rounded = IncrementRounder::<i128>::from_signed_num(self.as_i128(), increment)?
            .round(resolved_options.rounding_mode);

        Ok(rounded)
    }

    // Utility for converting `Instant` to `i128`.
    pub fn as_i128(&self) -> i128 {
        self.0 .0
    }
}

// ==== Public API ====

impl Instant {
    /// Create a new validated `Instant`.
    #[inline]
    pub fn try_new(nanoseconds: i128) -> TemporalResult<Self> {
        Ok(Self::from(EpochNanoseconds::try_from(nanoseconds)?))
    }

    pub fn from_epoch_milliseconds(epoch_milliseconds: i128) -> TemporalResult<Self> {
        let epoch_nanos = epoch_milliseconds
            .checked_mul(1_000_000)
            .unwrap_or(i128::MAX);
        Self::try_new(epoch_nanos)
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
    pub fn since(&self, other: &Self, settings: DifferenceSettings) -> TemporalResult<Duration> {
        self.diff_instant(DifferenceOperation::Since, other, settings)
    }

    /// Returns a `TimeDuration` representing the duration until provided `Instant`
    #[inline]
    pub fn until(&self, other: &Self, settings: DifferenceSettings) -> TemporalResult<Duration> {
        self.diff_instant(DifferenceOperation::Until, other, settings)
    }

    /// Returns an `Instant` by rounding the current `Instant` according to the provided settings.
    pub fn round(&self, options: RoundingOptions) -> TemporalResult<Self> {
        let resolved_options = ResolvedRoundingOptions::from_instant_options(options)?;

        let round_result = self.round_instant(resolved_options)?;
        Self::try_new(round_result)
    }

    /// Returns the `epochMilliseconds` value for this `Instant`.
    #[must_use]
    pub fn epoch_milliseconds(&self) -> i64 {
        (self.as_i128() / 1_000_000) as i64
    }

    /// Returns the `epochNanoseconds` value for this `Instant`.
    #[must_use]
    pub fn epoch_nanoseconds(&self) -> i128 {
        self.as_i128()
    }
}

// ==== Instant Provider API ====

impl Instant {
    pub fn to_ixdtf_string_with_provider(
        &self,
        timezone: Option<&TimeZone>,
        options: ToStringRoundingOptions,
        provider: &impl TimeZoneProvider,
    ) -> TemporalResult<String> {
        let resolved_options = options.resolve()?;
        let round = self.round_instant(ResolvedRoundingOptions::from_to_string_options(
            &resolved_options,
        ))?;
        let rounded_instant = Instant::try_new(round)?;

        let mut ixdtf = IxdtfStringBuilder::default();
        let datetime = if let Some(timezone) = timezone {
            let datetime = timezone.get_iso_datetime_for(&rounded_instant, provider)?;
            let nanoseconds = timezone.get_offset_nanos_for(rounded_instant.as_i128(), provider)?;
            let (sign, hour, minute) = nanoseconds_to_formattable_offset_minutes(nanoseconds)?;
            ixdtf = ixdtf.with_minute_offset(sign, hour, minute, DisplayOffset::Auto);
            datetime
        } else {
            ixdtf = ixdtf.with_z(DisplayOffset::Auto);
            TimeZone::default().get_iso_datetime_for(&rounded_instant, provider)?
        };
        let ixdtf_string = ixdtf
            .with_date(datetime.date)
            .with_time(datetime.time, resolved_options.precision)
            .build();

        Ok(ixdtf_string)
    }
}

// ==== Utility Functions ====

impl FromStr for Instant {
    type Err = TemporalError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let ixdtf_record = parse_instant(s)?;

        // Find the IsoDate
        let iso_date = IsoDate::new_with_overflow(
            ixdtf_record.date.year,
            ixdtf_record.date.month,
            ixdtf_record.date.day,
            ArithmeticOverflow::Reject,
        )?;

        // Find the IsoTime
        let iso_time = IsoTime::from_components(
            ixdtf_record.time.hour,
            ixdtf_record.time.minute,
            ixdtf_record.time.second,
            ixdtf_record.time.nanosecond,
        )?;

        // Find the offset
        let offset = match ixdtf_record.offset {
            UtcOffsetRecordOrZ::Offset(offset) => {
                f64::from(offset.hour) * NANOSECONDS_PER_HOUR
                    + f64::from(offset.minute) * NANOSECONDS_PER_MINUTE
                    + f64::from(offset.second) * NANOSECONDS_PER_SECOND
                    + f64::from(offset.nanosecond)
            }
            UtcOffsetRecordOrZ::Z => 0.0,
        };
        let nanoseconds = IsoDateTime::new_unchecked(iso_date, iso_time)
            .as_nanoseconds()
            .map(|v| v.0 + offset as i128);

        Self::try_new(nanoseconds.unwrap_or(i128::MAX))
    }
}

// ==== Instant Tests ====

#[cfg(test)]
mod tests {

    use crate::{
        builtins::core::{duration::TimeDuration, Instant},
        options::{DifferenceSettings, TemporalRoundingMode, TemporalUnit},
        primitive::FiniteF64,
        NS_MAX_INSTANT, NS_MIN_INSTANT,
    };

    #[test]
    #[allow(clippy::float_cmp)]
    fn max_and_minimum_instant_bounds() {
        // This test is primarily to assert that the `expect` in the epoch methods is
        // valid, i.e., a valid instant is within the range of an f64.
        let max = NS_MAX_INSTANT;
        let min = NS_MIN_INSTANT;
        let max_instant = Instant::try_new(max).unwrap();
        let min_instant = Instant::try_new(min).unwrap();

        assert_eq!(max_instant.epoch_nanoseconds(), max);
        assert_eq!(min_instant.epoch_nanoseconds(), min);

        let max_plus_one = NS_MAX_INSTANT + 1;
        let min_minus_one = NS_MIN_INSTANT - 1;

        assert!(Instant::try_new(max_plus_one).is_err());
        assert!(Instant::try_new(min_minus_one).is_err());
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

        let assert_time_duration = |td: &TimeDuration, expected: (f64, f64, f64, f64, f64, f64)| {
            assert_eq!(
                td,
                &TimeDuration {
                    hours: FiniteF64(expected.0),
                    minutes: FiniteF64(expected.1),
                    seconds: FiniteF64(expected.2),
                    milliseconds: FiniteF64(expected.3),
                    microseconds: FiniteF64(expected.4),
                    nanoseconds: FiniteF64(expected.5),
                }
            )
        };

        let earlier = Instant::try_new(
            217_178_610_123_456_789, /* 1976-11-18T15:23:30.123456789Z */
        )
        .unwrap();
        let later = Instant::try_new(
            1_572_345_998_271_986_289, /* 2019-10-29T10:46:38.271986289Z */
        )
        .unwrap();

        let positive_result = earlier
            .until(&later, init_diff_setting(TemporalUnit::Hour))
            .unwrap();
        assert_time_duration(positive_result.time(), (376436.0, 0.0, 0.0, 0.0, 0.0, 0.0));
        let negative_result = later
            .until(&earlier, init_diff_setting(TemporalUnit::Hour))
            .unwrap();
        assert_time_duration(negative_result.time(), (-376435.0, 0.0, 0.0, 0.0, 0.0, 0.0));

        let positive_result = earlier
            .until(&later, init_diff_setting(TemporalUnit::Minute))
            .unwrap();
        assert_time_duration(positive_result.time(), (376435.0, 24.0, 0.0, 0.0, 0.0, 0.0));
        let negative_result = later
            .until(&earlier, init_diff_setting(TemporalUnit::Minute))
            .unwrap();
        assert_time_duration(
            negative_result.time(),
            (-376435.0, -23.0, 0.0, 0.0, 0.0, 0.0),
        );

        // ... Skip to lower units ...

        let positive_result = earlier
            .until(&later, init_diff_setting(TemporalUnit::Microsecond))
            .unwrap();
        assert_time_duration(
            positive_result.time(),
            (376435.0, 23.0, 8.0, 148.0, 530.0, 0.0),
        );
        let negative_result = later
            .until(&earlier, init_diff_setting(TemporalUnit::Microsecond))
            .unwrap();
        assert_time_duration(
            negative_result.time(),
            (-376435.0, -23.0, -8.0, -148.0, -529.0, 0.0),
        );

        let positive_result = earlier
            .until(&later, init_diff_setting(TemporalUnit::Nanosecond))
            .unwrap();
        assert_time_duration(
            positive_result.time(),
            (376435.0, 23.0, 8.0, 148.0, 529.0, 500.0),
        );
        let negative_result = later
            .until(&earlier, init_diff_setting(TemporalUnit::Nanosecond))
            .unwrap();
        assert_time_duration(
            negative_result.time(),
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

        let assert_time_duration = |td: &TimeDuration, expected: (f64, f64, f64, f64, f64, f64)| {
            assert_eq!(
                td,
                &TimeDuration {
                    hours: FiniteF64(expected.0),
                    minutes: FiniteF64(expected.1),
                    seconds: FiniteF64(expected.2),
                    milliseconds: FiniteF64(expected.3),
                    microseconds: FiniteF64(expected.4),
                    nanoseconds: FiniteF64(expected.5),
                }
            )
        };

        let earlier = Instant::try_new(
            217_178_610_123_456_789, /* 1976-11-18T15:23:30.123456789Z */
        )
        .unwrap();
        let later = Instant::try_new(
            1_572_345_998_271_986_289, /* 2019-10-29T10:46:38.271986289Z */
        )
        .unwrap();

        let positive_result = later
            .since(&earlier, init_diff_setting(TemporalUnit::Hour))
            .unwrap();
        assert_time_duration(positive_result.time(), (376436.0, 0.0, 0.0, 0.0, 0.0, 0.0));
        let negative_result = earlier
            .since(&later, init_diff_setting(TemporalUnit::Hour))
            .unwrap();
        assert_time_duration(negative_result.time(), (-376435.0, 0.0, 0.0, 0.0, 0.0, 0.0));

        let positive_result = later
            .since(&earlier, init_diff_setting(TemporalUnit::Minute))
            .unwrap();
        assert_time_duration(positive_result.time(), (376435.0, 24.0, 0.0, 0.0, 0.0, 0.0));
        let negative_result = earlier
            .since(&later, init_diff_setting(TemporalUnit::Minute))
            .unwrap();
        assert_time_duration(
            negative_result.time(),
            (-376435.0, -23.0, 0.0, 0.0, 0.0, 0.0),
        );

        // ... Skip to lower units ...

        let positive_result = later
            .since(&earlier, init_diff_setting(TemporalUnit::Microsecond))
            .unwrap();
        assert_time_duration(
            positive_result.time(),
            (376435.0, 23.0, 8.0, 148.0, 530.0, 0.0),
        );
        let negative_result = earlier
            .since(&later, init_diff_setting(TemporalUnit::Microsecond))
            .unwrap();
        assert_time_duration(
            negative_result.time(),
            (-376435.0, -23.0, -8.0, -148.0, -529.0, 0.0),
        );

        let positive_result = later
            .since(&earlier, init_diff_setting(TemporalUnit::Nanosecond))
            .unwrap();
        assert_time_duration(
            positive_result.time(),
            (376435.0, 23.0, 8.0, 148.0, 529.0, 500.0),
        );
        let negative_result = earlier
            .since(&later, init_diff_setting(TemporalUnit::Nanosecond))
            .unwrap();
        assert_time_duration(
            negative_result.time(),
            (-376435.0, -23.0, -8.0, -148.0, -529.0, -500.0),
        );
    }

    // /test/built-ins/Temporal/Instant/prototype/add/cross-epoch.js
    #[cfg(feature = "tzdb")]
    #[test]
    fn instant_add_across_epoch() {
        use crate::{
            options::ToStringRoundingOptions, partial::PartialDuration, tzdb::FsTzdbProvider,
        };
        use crate::builtins::core::Duration;
        use core::str::FromStr;

        let instant = Instant::from_str("1969-12-25T12:23:45.678901234Z").unwrap();
        let one = instant
            .subtract(
                Duration::from_partial_duration(PartialDuration {
                    hours: Some(240.into()),
                    nanoseconds: Some(800.into()),
                    ..Default::default()
                })
                .unwrap(),
            )
            .unwrap();
        let two = instant
            .add(
                Duration::from_partial_duration(PartialDuration {
                    hours: Some(240.into()),
                    nanoseconds: Some(800.into()),
                    ..Default::default()
                })
                .unwrap(),
            )
            .unwrap();
        let three = two
            .subtract(
                Duration::from_partial_duration(PartialDuration {
                    hours: Some(480.into()),
                    nanoseconds: Some(1600.into()),
                    ..Default::default()
                })
                .unwrap(),
            )
            .unwrap();
        let four = one
            .add(
                Duration::from_partial_duration(PartialDuration {
                    hours: Some(480.into()),
                    nanoseconds: Some(1600.into()),
                    ..Default::default()
                })
                .unwrap(),
            )
            .unwrap();

        let one_comp = Instant::from_str("1969-12-15T12:23:45.678900434Z").unwrap();
        let two_comp = Instant::from_str("1970-01-04T12:23:45.678902034Z").unwrap();

        // Assert the comparisons all hold.
        assert_eq!(one, one_comp);
        assert_eq!(two, two_comp);
        assert_eq!(three, one);
        assert_eq!(four, two);

        // Assert the to_string is valid.
        let provider = &FsTzdbProvider::default();
        let inst_string = instant
            .to_ixdtf_string_with_provider(None, ToStringRoundingOptions::default(), provider)
            .unwrap();
        let one_string = one
            .to_ixdtf_string_with_provider(None, ToStringRoundingOptions::default(), provider)
            .unwrap();
        let two_string = two
            .to_ixdtf_string_with_provider(None, ToStringRoundingOptions::default(), provider)
            .unwrap();

        assert_eq!(&inst_string, "1969-12-25T12:23:45.678901234Z");
        assert_eq!(&one_string, "1969-12-15T12:23:45.678900434Z");
        assert_eq!(&two_string, "1970-01-04T12:23:45.678902034Z");
    }
}