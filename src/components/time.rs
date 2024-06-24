//! This module implements `Time` and any directly related algorithms.

use crate::{
    components::{duration::TimeDuration, Duration},
    iso::IsoTime,
    options::{ArithmeticOverflow, RoundingIncrement, TemporalRoundingMode, TemporalUnit},
    TemporalError, TemporalResult,
};

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

    /// Adds a `TimeDuration` to the current `Time`.
    ///
    /// Spec Equivalent: `AddDurationToOrSubtractDurationFromPlainTime` AND `AddTime`.
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

    /// Performs a desired difference op between two `Time`'s, returning the resulting `Duration`.
    pub(crate) fn diff_time(
        &self,
        op: bool,
        other: &Time,
        rounding_mode: Option<TemporalRoundingMode>,
        rounding_increment: Option<RoundingIncrement>,
        largest_unit: Option<TemporalUnit>,
        smallest_unit: Option<TemporalUnit>,
    ) -> TemporalResult<Duration> {
        // 1. If operation is SINCE, let sign be -1. Otherwise, let sign be 1.
        // 2. Set other to ? ToTemporalTime(other).
        // 3. Let resolvedOptions be ? SnapshotOwnProperties(? GetOptionsObject(options), null).
        // 4. Let settings be ? GetDifferenceSettings(operation, resolvedOptions, TIME, « », "nanosecond", "hour").
        let rounding_increment = rounding_increment.unwrap_or_default();
        let (sign, rounding_mode) = if op {
            (
                -1.0,
                rounding_mode
                    .unwrap_or(TemporalRoundingMode::Trunc)
                    .negate(),
            )
        } else {
            (1.0, rounding_mode.unwrap_or(TemporalRoundingMode::Trunc))
        };

        let smallest_unit = smallest_unit.unwrap_or(TemporalUnit::Nanosecond);
        // Use the defaultlargestunit which is max smallestlargestdefault and smallestunit
        let largest_unit = largest_unit.unwrap_or(smallest_unit.max(TemporalUnit::Hour));

        // 5. Let norm be ! DifferenceTime(temporalTime.[[ISOHour]], temporalTime.[[ISOMinute]],
        // temporalTime.[[ISOSecond]], temporalTime.[[ISOMillisecond]], temporalTime.[[ISOMicrosecond]],
        // temporalTime.[[ISONanosecond]], other.[[ISOHour]], other.[[ISOMinute]], other.[[ISOSecond]],
        // other.[[ISOMillisecond]], other.[[ISOMicrosecond]], other.[[ISONanosecond]]).
        let time = self.iso.diff(&other.iso);

        // 6. If settings.[[SmallestUnit]] is not "nanosecond" or settings.[[RoundingIncrement]] ≠ 1, then
        let norm = if smallest_unit != TemporalUnit::Nanosecond
            || rounding_increment != RoundingIncrement::ONE
        {
            // a. Let roundRecord be ! RoundDuration(0, 0, 0, 0, norm, settings.[[RoundingIncrement]], settings.[[SmallestUnit]], settings.[[RoundingMode]]).
            let round_record = time.round(rounding_increment, smallest_unit, rounding_mode)?;
            // b. Set norm to roundRecord.[[NormalizedDuration]].[[NormalizedTime]].
            round_record.0
        } else {
            time.to_normalized()
        };

        // 7. Let result be BalanceTimeDuration(norm, settings.[[LargestUnit]]).
        let result = TimeDuration::from_normalized(norm, largest_unit)?.1;
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
    pub fn until(
        &self,
        other: &Self,
        rounding_mode: Option<TemporalRoundingMode>,
        rounding_increment: Option<RoundingIncrement>,
        largest_unit: Option<TemporalUnit>,
        smallest_unit: Option<TemporalUnit>,
    ) -> TemporalResult<Duration> {
        self.diff_time(
            false,
            other,
            rounding_mode,
            rounding_increment,
            largest_unit,
            smallest_unit,
        )
    }

    #[inline]
    /// Returns the `Duration` since the provided `Time` from the current `Time`.
    ///
    /// NOTE: `since` assumes the provided other time is in the past relative to the current.
    pub fn since(
        &self,
        other: &Self,
        rounding_mode: Option<TemporalRoundingMode>,
        rounding_increment: Option<RoundingIncrement>,
        largest_unit: Option<TemporalUnit>,
        smallest_unit: Option<TemporalUnit>,
    ) -> TemporalResult<Duration> {
        self.diff_time(
            true,
            other,
            rounding_mode,
            rounding_increment,
            largest_unit,
            smallest_unit,
        )
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

// ==== Test land ====

#[cfg(test)]
mod tests {
    use crate::{
        components::Duration,
        iso::IsoTime,
        options::{ArithmeticOverflow, TemporalUnit},
    };

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

        let result = one.since(&two, None, None, None, None).unwrap();
        assert_eq!(result.hours(), 1.0);

        let result = two.since(&one, None, None, None, None).unwrap();
        assert_eq!(result.hours(), -1.0);

        let result = one.since(&three, None, None, None, None).unwrap();
        assert_eq!(result.hours(), 1.0);
        assert_eq!(result.minutes(), 53.0);

        let result = three.since(&one, None, None, None, None).unwrap();
        assert_eq!(result.hours(), -1.0);
        assert_eq!(result.minutes(), -53.0);
    }

    #[test]
    fn until_basic() {
        let one = Time::new(15, 23, 30, 123, 456, 789, ArithmeticOverflow::Constrain).unwrap();
        let two = Time::new(16, 23, 30, 123, 456, 789, ArithmeticOverflow::Constrain).unwrap();
        let three = Time::new(17, 0, 30, 123, 456, 789, ArithmeticOverflow::Constrain).unwrap();

        let result = one.until(&two, None, None, None, None).unwrap();
        assert_eq!(result.hours(), 1.0);

        let result = two.until(&one, None, None, None, None).unwrap();
        assert_eq!(result.hours(), -1.0);

        let result = one.until(&three, None, None, None, None).unwrap();
        assert_eq!(result.hours(), 1.0);
        assert_eq!(result.minutes(), 37.0);

        let result = three.until(&one, None, None, None, None).unwrap();
        assert_eq!(result.hours(), -1.0);
        assert_eq!(result.minutes(), -37.0);
    }
}
