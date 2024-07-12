//! This module implements `Time` and any directly related algorithms.

use crate::{
    components::{duration::TimeDuration, Duration},
    iso::IsoTime,
    options::{
        ArithmeticOverflow, DifferenceOperation, DifferenceSettings, ResolvedRoundingOptions,
        RoundingIncrement, TemporalRoundingMode, TemporalUnit,
    },
    parsers::parse_time,
    TemporalError, TemporalResult,
};

use super::{duration::normalized::NormalizedTimeDuration, DateTime};

use std::str::FromStr;

/// The native Rust implementation of `Temporal.PlainTime`.
#[non_exhaustive]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Time {
    pub(crate) iso: IsoTime,
}

// ==== Private API ====

impl Time {
    #[inline]
    #[must_use]
    /// Creates a new unvalidated `Time`.
    pub(crate) fn new_unchecked(iso: IsoTime) -> Self {
        Self { iso }
    }

    /// Returns true if a valid `Time`.
    #[allow(dead_code)]
    pub(crate) fn is_valid(&self) -> bool {
        self.iso.is_valid()
    }

    /// Specification equivalent to `AddTime`
    pub(crate) fn add_normalized_time_duration(&self, norm: NormalizedTimeDuration) -> (i32, Self) {
        // 1. Set second to second + NormalizedTimeDurationSeconds(norm).
        let second = i64::from(self.second()) + norm.seconds();
        // 2. Set nanosecond to nanosecond + NormalizedTimeDurationSubseconds(norm).
        let nanosecond = i32::from(self.nanosecond()) + norm.subseconds();
        // 3. Return BalanceTime(hour, minute, second, millisecond, microsecond, nanosecond).
        let (day, balance_result) = IsoTime::balance(
            f64::from(self.hour()),
            f64::from(self.minute()),
            second as f64,
            f64::from(self.millisecond()),
            f64::from(self.microsecond()),
            f64::from(nanosecond),
        );

        (day, Self::new_unchecked(balance_result))
    }

    /// Adds a `TimeDuration` to the current `Time`.
    ///
    /// Spec Equivalent: `AddDurationToOrSubtractDurationFromPlainTime`.
    pub(crate) fn add_to_time(&self, duration: &TimeDuration) -> Self {
        let (_, result) = IsoTime::balance(
            f64::from(self.hour()) + duration.hours,
            f64::from(self.minute()) + duration.minutes,
            f64::from(self.second()) + duration.seconds,
            f64::from(self.millisecond()) + duration.milliseconds,
            f64::from(self.microsecond()) + duration.microseconds,
            f64::from(self.nanosecond()) + duration.nanoseconds,
        );

        // NOTE (nekevss): IsoTime::balance should never return an invalid `IsoTime`

        Self::new_unchecked(result)
    }

    // TODO: Migrate to
    /// Performs a desired difference op between two `Time`'s, returning the resulting `Duration`.
    pub(crate) fn diff_time(
        &self,
        op: DifferenceOperation,
        other: &Time,
        settings: DifferenceSettings,
    ) -> TemporalResult<Duration> {
        // 1. If operation is SINCE, let sign be -1. Otherwise, let sign be 1.
        // 2. Set other to ? ToTemporalTime(other).
        // 3. Let resolvedOptions be ? SnapshotOwnProperties(? GetOptionsObject(options), null).
        // 4. Let settings be ? GetDifferenceSettings(operation, resolvedOptions, TIME, « », "nanosecond", "hour").
        let (sign, resolved) = ResolvedRoundingOptions::from_diff_settings(
            settings,
            op,
            TemporalUnit::Hour,
            TemporalUnit::Nanosecond,
        )?;

        // 5. Let norm be ! DifferenceTime(temporalTime.[[ISOHour]], temporalTime.[[ISOMinute]],
        // temporalTime.[[ISOSecond]], temporalTime.[[ISOMillisecond]], temporalTime.[[ISOMicrosecond]],
        // temporalTime.[[ISONanosecond]], other.[[ISOHour]], other.[[ISOMinute]], other.[[ISOSecond]],
        // other.[[ISOMillisecond]], other.[[ISOMicrosecond]], other.[[ISONanosecond]]).
        let mut normalized_time = self.iso.diff(&other.iso).to_normalized();

        // 6. If settings.[[SmallestUnit]] is not "nanosecond" or settings.[[RoundingIncrement]] ≠ 1, then
        if resolved.smallest_unit != TemporalUnit::Nanosecond
            || resolved.increment != RoundingIncrement::ONE
        {
            // a. Let roundRecord be ! RoundDuration(0, 0, 0, 0, norm, settings.[[RoundingIncrement]], settings.[[SmallestUnit]], settings.[[RoundingMode]]).
            let (round_record, _) = TimeDuration::round(0.0, &normalized_time, resolved)?;
            // b. Set norm to roundRecord.[[NormalizedDuration]].[[NormalizedTime]].
            normalized_time = round_record.normalized_time_duration()
        };

        // 7. Let result be BalanceTimeDuration(norm, settings.[[LargestUnit]]).
        let result = TimeDuration::from_normalized(normalized_time, resolved.largest_unit)?.1;

        let sign = f64::from(sign as i8);
        // 8. Return ! CreateTemporalDuration(0, 0, 0, 0, sign × result.[[Hours]], sign × result.[[Minutes]], sign × result.[[Seconds]], sign × result.[[Milliseconds]], sign × result.[[Microseconds]], sign × result.[[Nanoseconds]]).
        Duration::new(
            0.0,
            0.0,
            0.0,
            0.0,
            sign * result.hours,
            sign * result.minutes,
            sign * result.seconds,
            sign * result.milliseconds,
            sign * result.microseconds,
            sign * result.nanoseconds,
        )
    }
}

// ==== Public API ====

impl Time {
    /// Creates a new `IsoTime` value.
    pub fn new(
        hour: i32,
        minute: i32,
        second: i32,
        millisecond: i32,
        microsecond: i32,
        nanosecond: i32,
        overflow: ArithmeticOverflow,
    ) -> TemporalResult<Self> {
        let time = IsoTime::new(
            hour,
            minute,
            second,
            millisecond,
            microsecond,
            nanosecond,
            overflow,
        )?;
        Ok(Self::new_unchecked(time))
    }

    /// Returns the internal `hour` field.
    #[inline]
    #[must_use]
    pub const fn hour(&self) -> u8 {
        self.iso.hour
    }

    /// Returns the internal `minute` field.
    #[inline]
    #[must_use]
    pub const fn minute(&self) -> u8 {
        self.iso.minute
    }

    /// Returns the internal `second` field.
    #[inline]
    #[must_use]
    pub const fn second(&self) -> u8 {
        self.iso.second
    }

    /// Returns the internal `millisecond` field.
    #[inline]
    #[must_use]
    pub const fn millisecond(&self) -> u16 {
        self.iso.millisecond
    }

    /// Returns the internal `microsecond` field.
    #[inline]
    #[must_use]
    pub const fn microsecond(&self) -> u16 {
        self.iso.microsecond
    }

    /// Returns the internal `nanosecond` field.
    #[inline]
    #[must_use]
    pub const fn nanosecond(&self) -> u16 {
        self.iso.nanosecond
    }

    /// Add a `Duration` to the current `Time`.
    pub fn add(&self, duration: &Duration) -> TemporalResult<Self> {
        if !duration.is_time_duration() {
            return Err(TemporalError::range()
                .with_message("DateDuration values cannot be added to `Time`."));
        }
        Ok(self.add_time_duration(duration.time()))
    }

    /// Adds a `TimeDuration` to the current `Time`.
    #[inline]
    #[must_use]
    pub fn add_time_duration(&self, duration: &TimeDuration) -> Self {
        self.add_to_time(duration)
    }

    /// Subtract a `Duration` to the current `Time`.
    pub fn subtract(&self, duration: &Duration) -> TemporalResult<Self> {
        if !duration.is_time_duration() {
            return Err(TemporalError::range()
                .with_message("DateDuration values cannot be added to `Time` component."));
        }
        Ok(self.add_time_duration(duration.time()))
    }

    /// Adds a `TimeDuration` to the current `Time`.
    #[inline]
    #[must_use]
    pub fn subtract_time_duration(&self, duration: &TimeDuration) -> Self {
        self.add_to_time(&duration.negated())
    }

    #[inline]
    /// Returns the `Duration` until the provided `Time` from the current `Time`.
    ///
    /// NOTE: `until` assumes the provided other time will occur in the future relative to the current.
    pub fn until(&self, other: &Self, settings: DifferenceSettings) -> TemporalResult<Duration> {
        self.diff_time(DifferenceOperation::Until, other, settings)
    }

    #[inline]
    /// Returns the `Duration` since the provided `Time` from the current `Time`.
    ///
    /// NOTE: `since` assumes the provided other time is in the past relative to the current.
    pub fn since(&self, other: &Self, settings: DifferenceSettings) -> TemporalResult<Duration> {
        self.diff_time(DifferenceOperation::Since, other, settings)
    }

    // TODO (nekevss): optimize and test rounding_increment type (f64 vs. u64).
    /// Rounds the current `Time` according to provided options.
    pub fn round(
        &self,
        smallest_unit: TemporalUnit,
        rounding_increment: Option<f64>,
        rounding_mode: Option<TemporalRoundingMode>,
    ) -> TemporalResult<Self> {
        let increment = RoundingIncrement::try_from(rounding_increment.unwrap_or(1.0))?;
        let mode = rounding_mode.unwrap_or(TemporalRoundingMode::HalfExpand);

        let max = smallest_unit
            .to_maximum_rounding_increment()
            .ok_or_else(|| {
                TemporalError::range().with_message("smallestUnit must be a time value.")
            })?;

        // Safety (nekevss): to_rounding_increment returns a value in the range of a u32.
        increment.validate(u64::from(max), false)?;

        let (_, result) = self.iso.round(increment, smallest_unit, mode, None)?;

        Ok(Self::new_unchecked(result))
    }
}

impl From<DateTime> for Time {
    fn from(value: DateTime) -> Self {
        Time::new_unchecked(value.iso.time)
    }
}

impl FromStr for Time {
    type Err = TemporalError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let result = parse_time(s)?;

        let (millisecond, rem) = (result.nanosecond / 1_000_000, result.nanosecond % 1_000_000);
        let (microsecond, nanosecond) = (rem / 1_000, rem % 1_000);

        Time::new(
            result.hour.into(),
            result.minute.into(),
            result.second.into(),
            millisecond as i32,
            microsecond as i32,
            nanosecond as i32,
            ArithmeticOverflow::Reject,
        )
    }
}

// ==== Test land ====

#[cfg(test)]
mod tests {
    use crate::{
        components::Duration,
        iso::IsoTime,
        options::{ArithmeticOverflow, DifferenceSettings, TemporalUnit},
    };
    use num_traits::FromPrimitive;

    use super::Time;

    fn assert_time(result: Time, values: (u8, u8, u8, u16, u16, u16)) {
        assert_eq!(
            result,
            Time {
                iso: IsoTime {
                    hour: values.0,
                    minute: values.1,
                    second: values.2,
                    millisecond: values.3,
                    microsecond: values.4,
                    nanosecond: values.5,
                }
            }
        );
    }

    #[test]
    fn from_str_cast_sanity_test() {
        let max = u32::MAX;
        let (millisecond, rem) = (max / 1_000_000, max % 1_000_000);
        let (microsecond, nanosecond) = (rem / 1_000, rem % 1_000);

        assert!(i32::from_u32(millisecond).is_some());
        assert!(i32::from_u32(microsecond).is_some());
        assert!(i32::from_u32(nanosecond).is_some());
    }

    #[test]
    fn basic_parse_time() {
        let result = "T12:05:24-05:00[u-ca=iso8601]".parse::<Time>();
        assert_time(result.unwrap(), (12, 05, 24, 0, 0, 0));

        let result = "T12:05:24.123456789-05:00[u-ca=iso8601]".parse::<Time>();
        assert_time(result.unwrap(), (12, 05, 24, 123, 456, 789));

        let result = "2024-05-04 12:05:24.123456789-05:00[u-ca=iso8601]".parse::<Time>();
        assert_time(result.unwrap(), (12, 05, 24, 123, 456, 789));

        let result = "2024-05-04 12:05:24.123456789-05:00[u-ca=iso8601]".parse::<Time>();
        assert_time(result.unwrap(), (12, 05, 24, 123, 456, 789));
    }

    #[test]
    fn time_round_millisecond() {
        let base = Time::new_unchecked(IsoTime::new_unchecked(3, 34, 56, 987, 654, 321));

        let result_1 = base
            .round(TemporalUnit::Millisecond, Some(1.0), None)
            .unwrap();
        assert_time(result_1, (3, 34, 56, 988, 0, 0));

        let result_2 = base
            .round(TemporalUnit::Millisecond, Some(2.0), None)
            .unwrap();
        assert_time(result_2, (3, 34, 56, 988, 0, 0));

        let result_3 = base
            .round(TemporalUnit::Millisecond, Some(4.0), None)
            .unwrap();
        assert_time(result_3, (3, 34, 56, 988, 0, 0));

        let result_4 = base
            .round(TemporalUnit::Millisecond, Some(5.0), None)
            .unwrap();
        assert_time(result_4, (3, 34, 56, 990, 0, 0));
    }

    #[test]
    fn time_round_microsecond() {
        let base = Time::new_unchecked(IsoTime::new_unchecked(3, 34, 56, 987, 654, 321));

        let result_1 = base
            .round(TemporalUnit::Microsecond, Some(1.0), None)
            .unwrap();
        assert_time(result_1, (3, 34, 56, 987, 654, 0));

        let result_2 = base
            .round(TemporalUnit::Microsecond, Some(2.0), None)
            .unwrap();
        assert_time(result_2, (3, 34, 56, 987, 654, 0));

        let result_3 = base
            .round(TemporalUnit::Microsecond, Some(4.0), None)
            .unwrap();
        assert_time(result_3, (3, 34, 56, 987, 656, 0));

        let result_4 = base
            .round(TemporalUnit::Microsecond, Some(5.0), None)
            .unwrap();
        assert_time(result_4, (3, 34, 56, 987, 655, 0));
    }

    #[test]
    fn time_round_nanoseconds() {
        let base = Time::new_unchecked(IsoTime::new_unchecked(3, 34, 56, 987, 654, 321));

        let result_1 = base
            .round(TemporalUnit::Nanosecond, Some(1.0), None)
            .unwrap();
        assert_time(result_1, (3, 34, 56, 987, 654, 321));

        let result_2 = base
            .round(TemporalUnit::Nanosecond, Some(2.0), None)
            .unwrap();
        assert_time(result_2, (3, 34, 56, 987, 654, 322));

        let result_3 = base
            .round(TemporalUnit::Nanosecond, Some(4.0), None)
            .unwrap();
        assert_time(result_3, (3, 34, 56, 987, 654, 320));

        let result_4 = base
            .round(TemporalUnit::Nanosecond, Some(5.0), None)
            .unwrap();
        assert_time(result_4, (3, 34, 56, 987, 654, 320));
    }

    #[test]
    fn add_duration_basic() {
        let base = Time::new_unchecked(IsoTime::new_unchecked(15, 23, 30, 123, 456, 789));
        let result = base.add(&"PT16H".parse::<Duration>().unwrap()).unwrap();

        assert_time(result, (7, 23, 30, 123, 456, 789));
    }

    #[test]
    fn since_basic() {
        let one = Time::new(15, 23, 30, 123, 456, 789, ArithmeticOverflow::Constrain).unwrap();
        let two = Time::new(14, 23, 30, 123, 456, 789, ArithmeticOverflow::Constrain).unwrap();
        let three = Time::new(13, 30, 30, 123, 456, 789, ArithmeticOverflow::Constrain).unwrap();

        let result = one.since(&two, DifferenceSettings::default()).unwrap();
        assert_eq!(result.hours(), 1.0);

        let result = two.since(&one, DifferenceSettings::default()).unwrap();
        assert_eq!(result.hours(), -1.0);

        let result = one.since(&three, DifferenceSettings::default()).unwrap();
        assert_eq!(result.hours(), 1.0);
        assert_eq!(result.minutes(), 53.0);

        let result = three.since(&one, DifferenceSettings::default()).unwrap();
        assert_eq!(result.hours(), -1.0);
        assert_eq!(result.minutes(), -53.0);
    }

    #[test]
    fn until_basic() {
        let one = Time::new(15, 23, 30, 123, 456, 789, ArithmeticOverflow::Constrain).unwrap();
        let two = Time::new(16, 23, 30, 123, 456, 789, ArithmeticOverflow::Constrain).unwrap();
        let three = Time::new(17, 0, 30, 123, 456, 789, ArithmeticOverflow::Constrain).unwrap();

        let result = one.until(&two, DifferenceSettings::default()).unwrap();
        assert_eq!(result.hours(), 1.0);

        let result = two.until(&one, DifferenceSettings::default()).unwrap();
        assert_eq!(result.hours(), -1.0);

        let result = one.until(&three, DifferenceSettings::default()).unwrap();
        assert_eq!(result.hours(), 1.0);
        assert_eq!(result.minutes(), 37.0);

        let result = three.until(&one, DifferenceSettings::default()).unwrap();
        assert_eq!(result.hours(), -1.0);
        assert_eq!(result.minutes(), -37.0);
    }

    #[test]
    // test262/test/built-ins/Temporal/PlainTime/prototype/round/roundingincrement-nanoseconds.js
    fn rounding_increment_nanos() {
        let time = Time::new(3, 34, 56, 987, 654, 321, ArithmeticOverflow::Constrain).unwrap();

        assert_eq!(
            time.round(TemporalUnit::Nanosecond, Some(1.0), None)
                .unwrap(),
            Time::new(3, 34, 56, 987, 654, 321, ArithmeticOverflow::Constrain).unwrap()
        );
        assert_eq!(
            time.round(TemporalUnit::Nanosecond, Some(2.0), None)
                .unwrap(),
            Time::new(3, 34, 56, 987, 654, 322, ArithmeticOverflow::Constrain).unwrap()
        );
        assert_eq!(
            time.round(TemporalUnit::Nanosecond, Some(4.0), None)
                .unwrap(),
            Time::new(3, 34, 56, 987, 654, 320, ArithmeticOverflow::Constrain).unwrap()
        );
        assert_eq!(
            time.round(TemporalUnit::Nanosecond, Some(5.0), None)
                .unwrap(),
            Time::new(3, 34, 56, 987, 654, 320, ArithmeticOverflow::Constrain).unwrap()
        );
        assert_eq!(
            time.round(TemporalUnit::Nanosecond, Some(8.0), None)
                .unwrap(),
            Time::new(3, 34, 56, 987, 654, 320, ArithmeticOverflow::Constrain).unwrap()
        );
        assert_eq!(
            time.round(TemporalUnit::Nanosecond, Some(10.0), None)
                .unwrap(),
            Time::new(3, 34, 56, 987, 654, 320, ArithmeticOverflow::Constrain).unwrap()
        );
        assert_eq!(
            time.round(TemporalUnit::Nanosecond, Some(20.0), None)
                .unwrap(),
            Time::new(3, 34, 56, 987, 654, 320, ArithmeticOverflow::Constrain).unwrap()
        );
        assert_eq!(
            time.round(TemporalUnit::Nanosecond, Some(25.0), None)
                .unwrap(),
            Time::new(3, 34, 56, 987, 654, 325, ArithmeticOverflow::Constrain).unwrap()
        );
        assert_eq!(
            time.round(TemporalUnit::Nanosecond, Some(40.0), None)
                .unwrap(),
            Time::new(3, 34, 56, 987, 654, 320, ArithmeticOverflow::Constrain).unwrap()
        );
        assert_eq!(
            time.round(TemporalUnit::Nanosecond, Some(50.0), None)
                .unwrap(),
            Time::new(3, 34, 56, 987, 654, 300, ArithmeticOverflow::Constrain).unwrap()
        );
        assert_eq!(
            time.round(TemporalUnit::Nanosecond, Some(100.0), None)
                .unwrap(),
            Time::new(3, 34, 56, 987, 654, 300, ArithmeticOverflow::Constrain).unwrap()
        );
        assert_eq!(
            time.round(TemporalUnit::Nanosecond, Some(125.0), None)
                .unwrap(),
            Time::new(3, 34, 56, 987, 654, 375, ArithmeticOverflow::Constrain).unwrap()
        );
        assert_eq!(
            time.round(TemporalUnit::Nanosecond, Some(200.0), None)
                .unwrap(),
            Time::new(3, 34, 56, 987, 654, 400, ArithmeticOverflow::Constrain).unwrap()
        );
        assert_eq!(
            time.round(TemporalUnit::Nanosecond, Some(250.0), None)
                .unwrap(),
            Time::new(3, 34, 56, 987, 654, 250, ArithmeticOverflow::Constrain).unwrap()
        );
        assert_eq!(
            time.round(TemporalUnit::Nanosecond, Some(500.0), None)
                .unwrap(),
            Time::new(3, 34, 56, 987, 654, 500, ArithmeticOverflow::Constrain).unwrap()
        );
    }
}
