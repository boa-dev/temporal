//! This module implements the normalized `Duration` records.

use core::{num::NonZeroU128, ops::Add};
use std::println;

use num_traits::{AsPrimitive, Euclid, FromPrimitive};

use crate::{
    components::{
        tz::{TimeZone, TzProvider},
        PlainDate, PlainDateTime,
    },
    iso::{IsoDate, IsoDateTime},
    options::{
        ArithmeticOverflow, Disambiguation, ResolvedRoundingOptions, TemporalRoundingMode,
        TemporalUnit,
    },
    primitive::FiniteF64,
    rounding::{IncrementRounder, Round},
    TemporalError, TemporalResult, TemporalUnwrap, NS_PER_DAY,
};

use super::{DateDuration, Duration, Sign, TimeDuration};

const MAX_TIME_DURATION: i128 = 9_007_199_254_740_991_999_999_999;

// Nanoseconds constants

const NS_PER_DAY_128BIT: i128 = NS_PER_DAY as i128;
const NANOSECONDS_PER_MINUTE: f64 = 60.0 * 1e9;
const NANOSECONDS_PER_HOUR: f64 = 60.0 * 60.0 * 1e9;

// ==== NormalizedTimeDuration ====
//
// A time duration represented in pure nanoseconds.
//
// Invariants:
//
// nanoseconds.abs() <= MAX_TIME_DURATION

/// A Normalized `TimeDuration` that represents the current `TimeDuration` in nanoseconds.
#[derive(Debug, Clone, Copy, Default, PartialEq, PartialOrd)]
pub struct NormalizedTimeDuration(pub(crate) i128);

impl NormalizedTimeDuration {
    /// Equivalent: 7.5.20 NormalizeTimeDuration ( hours, minutes, seconds, milliseconds, microseconds, nanoseconds )
    pub(crate) fn from_time_duration(time: &TimeDuration) -> Self {
        // TODO: Determine if there is a loss in precision from casting. If so, times by 1,000 (calculate in picoseconds) than truncate?
        let mut nanoseconds: i128 = (time.hours.0 * NANOSECONDS_PER_HOUR) as i128;
        nanoseconds += (time.minutes.0 * NANOSECONDS_PER_MINUTE) as i128;
        nanoseconds += (time.seconds.0 * 1_000_000_000.0) as i128;
        nanoseconds += (time.milliseconds.0 * 1_000_000.0) as i128;
        nanoseconds += (time.microseconds.0 * 1_000.0) as i128;
        nanoseconds += time.nanoseconds.0 as i128;
        // NOTE(nekevss): Is it worth returning a `RangeError` below.
        debug_assert!(nanoseconds.abs() <= MAX_TIME_DURATION);
        Self(nanoseconds)
    }

    /// Equivalent to 7.5.27 NormalizedTimeDurationFromEpochNanosecondsDifference ( one, two )
    pub(crate) fn from_nanosecond_difference(one: i128, two: i128) -> TemporalResult<Self> {
        let result = one - two;
        if result.abs() > MAX_TIME_DURATION {
            return Err(TemporalError::range()
                .with_message("normalizedTimeDuration exceeds maxTimeDuration."));
        }
        Ok(Self(result))
    }

    // NOTE: `days: f64` should be an integer -> `i64`.
    /// Equivalent: 7.5.23 Add24HourDaysToNormalizedTimeDuration ( d, days )
    #[allow(unused)]
    pub(crate) fn add_days(&self, days: i64) -> TemporalResult<Self> {
        let result = self.0 + i128::from(days) * i128::from(NS_PER_DAY);
        if result.abs() > MAX_TIME_DURATION {
            return Err(TemporalError::range()
                .with_message("normalizedTimeDuration exceeds maxTimeDuration."));
        }
        Ok(Self(result))
    }

    // TODO: Potentially, update divisor to u64?
    /// `Divide the NormalizedTimeDuraiton` by a divisor.
    pub(crate) fn divide(&self, divisor: i64) -> i128 {
        // TODO: Validate.
        self.0 / i128::from(divisor)
    }

    // NOTE(nekevss): non-euclid is required here for negative rounding.
    /// Returns the div_rem of this NormalizedTimeDuration.
    pub(super) fn div_rem(&self, divisor: u64) -> (i128, i128) {
        (self.0 / i128::from(divisor), self.0 % i128::from(divisor))
    }

    // Returns the fractionalDays value represented by this `NormalizedTimeDuration`
    pub(super) fn as_fractional_days(&self) -> f64 {
        // TODO: Unit test to verify MaxNormalized is within a castable f64 range.
        let (days, remainder) = self.0.div_rem_euclid(&NS_PER_DAY_128BIT);
        days as f64 + (remainder as f64 / NS_PER_DAY as f64)
    }

    // TODO: Potentially abstract sign into `Sign`
    /// Equivalent: 7.5.31 NormalizedTimeDurationSign ( d )
    #[inline]
    #[must_use]
    pub(crate) fn sign(&self) -> Sign {
        Sign::from(self.0.cmp(&0) as i8)
    }

    // NOTE(nekevss): non-euclid is required here for negative rounding.
    /// Return the seconds value of the `NormalizedTimeDuration`.
    pub(crate) fn seconds(&self) -> i64 {
        // SAFETY: See validate_second_cast test.
        (self.0 / 1_000_000_000) as i64
    }

    // NOTE(nekevss): non-euclid is required here for negative rounding.
    /// Returns the subsecond components of the `NormalizedTimeDuration`.
    pub(crate) fn subseconds(&self) -> i32 {
        // SAFETY: Remainder is 10e9 which is in range of i32
        (self.0 % 1_000_000_000) as i32
    }

    fn negate(&self) -> Self {
        Self(-self.0)
    }

    pub(crate) fn checked_sub(&self, other: &Self) -> TemporalResult<Self> {
        let result = self.0 - other.0;
        if result.abs() > MAX_TIME_DURATION {
            return Err(TemporalError::range().with_message(
                "SubtractNormalizedTimeDuration exceeded a valid TimeDuration range.",
            ));
        }
        Ok(Self(result))
    }

    /// The equivalent of `RoundTimeDuration` abstract operation.
    pub(crate) fn round(
        &self,
        days: FiniteF64,
        options: ResolvedRoundingOptions,
    ) -> TemporalResult<(NormalizedDurationRecord, Option<i128>)> {
        // 1. Assert: IsCalendarUnit(unit) is false.
        let (days, norm, total) = match options.smallest_unit {
            // 2. If unit is "day", then
            TemporalUnit::Day => {
                // a. Let fractionalDays be days + DivideNormalizedTimeDuration(norm, nsPerDay).
                let fractional_days = days.checked_add(&FiniteF64(self.as_fractional_days()))?;
                // b. Set days to RoundNumberToIncrement(fractionalDays, increment, roundingMode).
                let days = IncrementRounder::from_potentially_negative_parts(
                    fractional_days.0,
                    options.increment.as_extended_increment(),
                )?
                .round(options.rounding_mode);
                // c. Let total be fractionalDays.
                // d. Set norm to ZeroTimeDuration().
                (
                    FiniteF64::try_from(days)?,
                    NormalizedTimeDuration::default(),
                    i128::from_f64(fractional_days.0),
                )
            }
            // 3. Else,
            TemporalUnit::Hour
            | TemporalUnit::Minute
            | TemporalUnit::Second
            | TemporalUnit::Millisecond
            | TemporalUnit::Microsecond
            | TemporalUnit::Nanosecond => {
                // a. Assert: The value in the "Category" column of the row of Table 22 whose "Singular" column contains unit, is time.
                // b. Let divisor be the value in the "Length in Nanoseconds" column of the row of Table 22 whose "Singular" column contains unit.
                let divisor = options.smallest_unit.as_nanoseconds().temporal_unwrap()?;
                // c. Let total be DivideNormalizedTimeDuration(norm, divisor).
                let total = self.divide(divisor as i64);
                let non_zero_divisor = unsafe { NonZeroU128::new_unchecked(divisor.into()) };
                // d. Set norm to ? RoundNormalizedTimeDurationToIncrement(norm, divisor × increment, roundingMode).
                let norm = self.round_inner(
                    non_zero_divisor
                        .checked_mul(options.increment.as_extended_increment())
                        .temporal_unwrap()?,
                    options.rounding_mode,
                )?;
                (days, norm, Some(total))
            }
            _ => return Err(TemporalError::assert()),
        };

        // 4. Return the Record { [[NormalizedDuration]]: ? CreateNormalizedDurationRecord(0, 0, 0, days, norm), [[Total]]: total  }.
        Ok((
            NormalizedDurationRecord::new(
                DateDuration::new(
                    FiniteF64::default(),
                    FiniteF64::default(),
                    FiniteF64::default(),
                    days,
                )?,
                norm,
            )?,
            total,
        ))
    }

    /// Round the current `NormalizedTimeDuration`.
    pub(super) fn round_inner(
        &self,
        increment: NonZeroU128,
        mode: TemporalRoundingMode,
    ) -> TemporalResult<Self> {
        let rounded = IncrementRounder::<i128>::from_potentially_negative_parts(self.0, increment)?
            .round(mode);
        if rounded.abs() > MAX_TIME_DURATION {
            return Err(TemporalError::range()
                .with_message("normalizedTimeDuration exceeds maxTimeDuration."));
        }
        Ok(Self(rounded))
    }

    pub(super) fn checked_add(&self, other: i128) -> TemporalResult<Self> {
        let result = self.0 + other;
        if result.abs() > MAX_TIME_DURATION {
            return Err(TemporalError::range()
                .with_message("normalizedTimeDuration exceeds maxTimeDuration."));
        }
        Ok(Self(result))
    }
}

// NOTE(nekevss): As this `Add` impl is fallible. Maybe it would be best implemented as a method.
/// Equivalent: 7.5.22 AddNormalizedTimeDuration ( one, two )
impl Add<Self> for NormalizedTimeDuration {
    type Output = TemporalResult<Self>;

    fn add(self, rhs: Self) -> Self::Output {
        let result = self.0 + rhs.0;
        if result.abs() > MAX_TIME_DURATION {
            return Err(TemporalError::range()
                .with_message("normalizedTimeDuration exceeds maxTimeDuration."));
        }
        Ok(Self(result))
    }
}

// ==== NormalizedDurationRecord ====
//
// A record consisting of a DateDuration and NormalizedTimeDuration
//

/// A NormalizedDurationRecord is a duration record that contains
/// a `DateDuration` and `NormalizedTimeDuration`.
#[derive(Debug, Default, Clone, Copy)]
pub struct NormalizedDurationRecord {
    date: DateDuration,
    norm: NormalizedTimeDuration,
}

impl NormalizedDurationRecord {
    /// Creates a new `NormalizedDurationRecord`.
    ///
    /// Equivalent: `CreateNormalizedDurationRecord` & `CombineDateAndNormalizedTimeDuration`.
    pub(crate) fn new(date: DateDuration, norm: NormalizedTimeDuration) -> TemporalResult<Self> {
        if date.sign() != Sign::Zero && norm.sign() != Sign::Zero && date.sign() != norm.sign() {
            return Err(TemporalError::range()
                .with_message("DateDuration and NormalizedTimeDuration must agree."));
        }
        Ok(Self { date, norm })
    }

    pub(crate) fn from_date_duration(date: DateDuration) -> TemporalResult<Self> {
        Self::new(date, NormalizedTimeDuration::default())
    }

    pub(crate) fn date(&self) -> DateDuration {
        self.date
    }

    pub(crate) fn normalized_time_duration(&self) -> NormalizedTimeDuration {
        self.norm
    }

    pub(crate) fn sign(&self) -> TemporalResult<Sign> {
        Ok(self.date.sign())
    }
}

// ==== Nudge Duration Rounding Functions ====

// Below implements the nudge rounding functionality for Duration.
//
// Generally, this rounding is implemented on a NormalizedDurationRecord,
// which is the reason the functionality lives below.

#[allow(unused)]
#[derive(Debug)]
struct NudgeRecord {
    normalized: NormalizedDurationRecord,
    total: Option<i128>, // TODO: adjust
    nudge_epoch_ns: i128,
    expanded: bool,
}

impl NormalizedDurationRecord {
    // TODO: Add assertion into impl.
    // TODO: Add unit tests specifically for nudge_calendar_unit if possible.
    fn nudge_calendar_unit(
        &self,
        sign: Sign,
        dest_epoch_ns: i128,
        dt: &PlainDateTime,
        tz: Option<(&TimeZone, &impl TzProvider)>, // ???
        options: ResolvedRoundingOptions,
    ) -> TemporalResult<NudgeRecord> {
        // NOTE: r2 may never be used...need to test.
        let (r1, r2, start_duration, end_duration) = match options.smallest_unit {
            // 1. If unit is "year", then
            TemporalUnit::Year => {
                // a. Let years be RoundNumberToIncrement(duration.[[Years]], increment, "trunc").
                let years = IncrementRounder::from_potentially_negative_parts(
                    self.date().years.0,
                    options.increment.as_extended_increment(),
                )?
                .round(TemporalRoundingMode::Trunc);
                // b. Let r1 be years.
                let r1 = years;
                // c. Let r2 be years + increment × sign.
                let r2 = years
                    + i128::from(options.increment.get()) * i128::from(sign.as_sign_multiplier());
                // d. Let startDuration be ? CreateNormalizedDurationRecord(r1, 0, 0, 0, ZeroTimeDuration()).
                // e. Let endDuration be ? CreateNormalizedDurationRecord(r2, 0, 0, 0, ZeroTimeDuration()).
                (
                    r1,
                    r2,
                    DateDuration::new(
                        FiniteF64::try_from(r1)?,
                        FiniteF64::default(),
                        FiniteF64::default(),
                        FiniteF64::default(),
                    )?,
                    DateDuration::new(
                        FiniteF64::try_from(r2)?,
                        FiniteF64::default(),
                        FiniteF64::default(),
                        FiniteF64::default(),
                    )?,
                )
            }
            // 2. Else if unit is "month", then
            TemporalUnit::Month => {
                // a. Let months be RoundNumberToIncrement(duration.[[Months]], increment, "trunc").
                let months = IncrementRounder::from_potentially_negative_parts(
                    self.date().months.0,
                    options.increment.as_extended_increment(),
                )?
                .round(TemporalRoundingMode::Trunc);
                // b. Let r1 be months.
                let r1 = months;
                // c. Let r2 be months + increment × sign.
                let r2 = months
                    + i128::from(options.increment.get()) * i128::from(sign.as_sign_multiplier());
                // d. Let startDuration be ? CreateNormalizedDurationRecord(duration.[[Years]], r1, 0, 0, ZeroTimeDuration()).
                // e. Let endDuration be ? CreateNormalizedDurationRecord(duration.[[Years]], r2, 0, 0, ZeroTimeDuration()).
                (
                    r1,
                    r2,
                    DateDuration::new(
                        self.date().years,
                        FiniteF64::try_from(r1)?,
                        FiniteF64::default(),
                        FiniteF64::default(),
                    )?,
                    DateDuration::new(
                        self.date().years,
                        FiniteF64::try_from(r2)?,
                        FiniteF64::default(),
                        FiniteF64::default(),
                    )?,
                )
            }
            // 3. Else if unit is "week", then
            TemporalUnit::Week => {
                // TODO: Reconcile potential overflow on years as i32. `ValidateDuration` requires years, months, weeks to be abs(x) <= 2^32
                // a. Let isoResult1 be BalanceISODate(dateTime.[[Year]] + duration.[[Years]],
                // dateTime.[[Month]] + duration.[[Months]], dateTime.[[Day]]).
                let iso_one = IsoDate::balance(
                    dt.iso_year() + self.date().years.as_date_value()?,
                    i32::from(dt.iso_month()) + self.date().months.as_date_value()?,
                    i32::from(dt.iso_day()),
                );

                // b. Let isoResult2 be BalanceISODate(dateTime.[[Year]] + duration.[[Years]], dateTime.[[Month]] +
                // duration.[[Months]], dateTime.[[Day]] + duration.[[Days]]).
                let iso_two = IsoDate::balance(
                    dt.iso_year() + self.date().years.as_date_value()?,
                    i32::from(dt.iso_month()) + self.date().months.as_date_value()?,
                    i32::from(dt.iso_day()) + self.date().days.as_date_value()?,
                );

                // c. Let weeksStart be ! CreateTemporalDate(isoResult1.[[Year]], isoResult1.[[Month]], isoResult1.[[Day]],
                // calendarRec.[[Receiver]]).
                let weeks_start = PlainDate::try_new(
                    iso_one.year,
                    iso_one.month.into(),
                    iso_one.day.into(),
                    dt.calendar().clone(),
                )?;

                // d. Let weeksEnd be ! CreateTemporalDate(isoResult2.[[Year]], isoResult2.[[Month]], isoResult2.[[Day]],
                // calendarRec.[[Receiver]]).
                let weeks_end = PlainDate::try_new(
                    iso_two.year,
                    iso_two.month.into(),
                    iso_two.day.into(),
                    dt.calendar().clone(),
                )?;

                // e. Let untilOptions be OrdinaryObjectCreate(null).
                // f. Perform ! CreateDataPropertyOrThrow(untilOptions, "largestUnit", "week").
                // g. Let untilResult be ? DifferenceDate(calendarRec, weeksStart, weeksEnd, untilOptions).
                let until_result =
                    weeks_start.internal_diff_date(&weeks_end, TemporalUnit::Week)?;

                // h. Let weeks be RoundNumberToIncrement(duration.[[Weeks]] + untilResult.[[Weeks]], increment, "trunc").
                let weeks = IncrementRounder::from_potentially_negative_parts(
                    self.date().weeks.checked_add(&until_result.weeks())?.0,
                    options.increment.as_extended_increment(),
                )?
                .round(TemporalRoundingMode::Trunc);

                // i. Let r1 be weeks.
                let r1 = weeks;
                // j. Let r2 be weeks + increment × sign.
                let r2 = weeks
                    + i128::from(options.increment.get()) * i128::from(sign.as_sign_multiplier());
                // k. Let startDuration be ? CreateNormalizedDurationRecord(duration.[[Years]], duration.[[Months]], r1, 0, ZeroTimeDuration()).
                // l. Let endDuration be ? CreateNormalizedDurationRecord(duration.[[Years]], duration.[[Months]], r2, 0, ZeroTimeDuration()).
                (
                    r1,
                    r2,
                    DateDuration::new(
                        self.date().years,
                        self.date().months,
                        FiniteF64::try_from(r1)?,
                        FiniteF64::default(),
                    )?,
                    DateDuration::new(
                        self.date().years,
                        self.date().months,
                        FiniteF64::try_from(r2)?,
                        FiniteF64::default(),
                    )?,
                )
            }
            TemporalUnit::Day => {
                // 4. Else,
                // a. Assert: unit is "day".
                // b. Let days be RoundNumberToIncrement(duration.[[Days]], increment, "trunc").
                let days = IncrementRounder::from_potentially_negative_parts(
                    self.date().days.0,
                    options.increment.as_extended_increment(),
                )?
                .round(TemporalRoundingMode::Trunc);
                // c. Let r1 be days.
                let r1 = days;
                // d. Let r2 be days + increment × sign.
                let r2 = days
                    + i128::from(options.increment.get()) * i128::from(sign.as_sign_multiplier());
                // e. Let startDuration be ? CreateNormalizedDurationRecord(duration.[[Years]], duration.[[Months]], duration.[[Weeks]], r1, ZeroTimeDuration()).
                // f. Let endDuration be ? CreateNormalizedDurationRecord(duration.[[Years]], duration.[[Months]], duration.[[Weeks]], r2, ZeroTimeDuration()).
                (
                    r1,
                    r2,
                    DateDuration::new(
                        self.date().years,
                        self.date().months,
                        self.date().weeks,
                        FiniteF64::try_from(r1)?,
                    )?,
                    DateDuration::new(
                        self.date().years,
                        self.date().months,
                        self.date().weeks,
                        FiniteF64::try_from(r2)?,
                    )?,
                )
            }
            _ => unreachable!(), // TODO: potentially reject with range error?
        };

        // 5. Let start be ? AddDateTime(dateTime.[[Year]], dateTime.[[Month]], dateTime.[[Day]], dateTime.[[Hour]], dateTime.[[Minute]],
        // dateTime.[[Second]], dateTime.[[Millisecond]], dateTime.[[Microsecond]], dateTime.[[Nanosecond]], calendarRec,
        // startDuration.[[Years]], startDuration.[[Months]], startDuration.[[Weeks]], startDuration.[[Days]], startDuration.[[NormalizedTime]], undefined).
        let start = dt.iso.add_date_duration(
            dt.calendar().clone(),
            &start_duration,
            NormalizedTimeDuration::default(),
            None,
        )?;

        // 6. Let end be ? AddDateTime(dateTime.[[Year]], dateTime.[[Month]], dateTime.[[Day]], dateTime.[[Hour]],
        // dateTime.[[Minute]], dateTime.[[Second]], dateTime.[[Millisecond]], dateTime.[[Microsecond]],
        // dateTime.[[Nanosecond]], calendarRec, endDuration.[[Years]], endDuration.[[Months]], endDuration.[[Weeks]],
        // endDuration.[[Days]], endDuration.[[NormalizedTime]], undefined).
        let end = dt.iso.add_date_duration(
            dt.calendar().clone(),
            &end_duration,
            NormalizedTimeDuration::default(),
            None,
        )?;

        // NOTE: 7-8 are inversed
        // 8. Else,
        let (start_epoch_ns, end_epoch_ns) = if let Some((tz, provider)) = tz {
            // a. Let startEpochNs be ? GetEpochNanosecondsFor(timeZone, startDateTime, compatible).
            // b. Let endEpochNs be ? GetEpochNanosecondsFor(timeZone, endDateTime, compatible).
            let start_epoch_ns =
                tz.get_epoch_nanoseconds_for(start, Disambiguation::Compatible, provider)?;
            let end_epoch_ns =
                tz.get_epoch_nanoseconds_for(end, Disambiguation::Compatible, provider)?;
            (start_epoch_ns, end_epoch_ns)
        // 7. If timeZoneRec is unset, then
        } else {
            // a. Let startEpochNs be GetUTCEpochNanoseconds(start.[[Year]], start.[[Month]], start.[[Day]], start.[[Hour]], start.[[Minute]], start.[[Second]], start.[[Millisecond]], start.[[Microsecond]], start.[[Nanosecond]]).
            // b. Let endEpochNs be GetUTCEpochNanoseconds(end.[[Year]], end.[[Month]], end.[[Day]], end.[[Hour]], end.[[Minute]], end.[[Second]], end.[[Millisecond]], end.[[Microsecond]], end.[[Nanosecond]]).
            (start.as_nanoseconds()?, end.as_nanoseconds()?)
        };

        // 9. If endEpochNs = startEpochNs, throw a RangeError exception.
        if end_epoch_ns == start_epoch_ns {
            return Err(
                TemporalError::range().with_message("endEpochNs cannot be equal to startEpochNs")
            );
        }

        // TODO: Add early RangeError steps that are currently missing

        // NOTE: Below is removed in place of using `IncrementRounder`
        // 10. If sign < 0, let isNegative be negative; else let isNegative be positive.
        // 11. Let unsignedRoundingMode be GetUnsignedRoundingMode(roundingMode, isNegative).

        // NOTE(nekevss): Step 12..13 could be problematic...need tests
        // and verify, or completely change the approach involved.
        // TODO(nekevss): Validate that the `f64` casts here are valid in all scenarios
        // 12. Let progress be (destEpochNs - startEpochNs) / (endEpochNs - startEpochNs).
        // 13. Let total be r1 + progress × increment × sign.
        let progress =
            (dest_epoch_ns - start_epoch_ns.0) as f64 / (end_epoch_ns.0 - start_epoch_ns.0) as f64;
        let total = r1 as f64
            + progress * options.increment.get() as f64 * f64::from(sign.as_sign_multiplier());

        // TODO: Test and verify that `IncrementRounder` handles the below case.
        // NOTE(nekevss): Below will not return the calculated r1 or r2, so it is imporant to not use
        // the result beyond determining rounding direction.
        // 14. NOTE: The above two steps cannot be implemented directly using floating-point arithmetic.
        // This division can be implemented as if constructing Normalized Time Duration Records for the denominator
        // and numerator of total and performing one division operation with a floating-point result.
        // 15. Let roundedUnit be ApplyUnsignedRoundingMode(total, r1, r2, unsignedRoundingMode).
        let rounded_unit = IncrementRounder::from_potentially_negative_parts(
            total,
            options.increment.as_extended_increment(),
        )?
        .round(options.rounding_mode);

        // 16. If roundedUnit - total < 0, let roundedSign be -1; else let roundedSign be 1.
        // 19. Return Duration Nudge Result Record { [[Duration]]: resultDuration, [[Total]]: total, [[NudgedEpochNs]]: nudgedEpochNs, [[DidExpandCalendarUnit]]: didExpandCalendarUnit }.
        // 17. If roundedSign = sign, then
        if rounded_unit == r2 {
            // a. Let didExpandCalendarUnit be true.
            // b. Let resultDuration be endDuration.
            // c. Let nudgedEpochNs be endEpochNs.
            Ok(NudgeRecord {
                normalized: NormalizedDurationRecord::new(
                    end_duration,
                    NormalizedTimeDuration::default(),
                )?,
                total: Some(total as i128),
                nudge_epoch_ns: end_epoch_ns.0,
                expanded: true,
            })
        // 18. Else,
        } else {
            // a. Let didExpandCalendarUnit be false.
            // b. Let resultDuration be startDuration.
            // c. Let nudgedEpochNs be startEpochNs.
            Ok(NudgeRecord {
                normalized: NormalizedDurationRecord::new(
                    start_duration,
                    NormalizedTimeDuration::default(),
                )?,
                total: Some(total as i128),
                nudge_epoch_ns: start_epoch_ns.0,
                expanded: false,
            })
        }
    }

    // TODO: Clean up
    #[inline]
    fn nudge_to_zoned_time(
        &self,
        sign: Sign,
        iso: &PlainDateTime,
        tz: &TimeZone,
        options: ResolvedRoundingOptions,
        provider: &impl TzProvider,
    ) -> TemporalResult<NudgeRecord> {
        let d = Duration::from(self.date());
        // 1.Let start be ? CalendarDateAdd(calendar, isoDateTime.[[ISODate]], duration.[[Date]], constrain).
        let start = iso.calendar().date_add(
            &PlainDate::new_unchecked(iso.iso.date, iso.calendar().clone()),
            &d,
            ArithmeticOverflow::Constrain,
        )?;
        // 2. Let startDateTime be CombineISODateAndTimeRecord(start, isoDateTime.[[Time]]).
        let start_dt = IsoDateTime::new_unchecked(start.iso, iso.iso.time);

        // 3. Let endDate be BalanceISODate(start.[[Year]], start.[[Month]], start.[[Day]] + sign).
        let end_date = IsoDate::balance(
            start.iso_year(),
            start.iso_month().into(),
            start.iso_day() as i32 + sign as i32,
        );

        // 4. Let endDateTime be CombineISODateAndTimeRecord(endDate, isoDateTime.[[Time]]).
        let end_dt = IsoDateTime::new_unchecked(end_date, iso.iso.time);
        // 5. Let startEpochNs be ? GetEpochNanosecondsFor(timeZone, startDateTime, compatible).
        let start_ns =
            tz.get_epoch_nanoseconds_for(start_dt, Disambiguation::Compatible, provider)?;
        // 6. Let endEpochNs be ? GetEpochNanosecondsFor(timeZone, endDateTime, compatible).
        let end_ns = tz.get_epoch_nanoseconds_for(end_dt, Disambiguation::Compatible, provider)?;
        // 7. Let daySpan be TimeDurationFromEpochNanosecondsDifference(endEpochNs, startEpochNs).
        let day_span = NormalizedTimeDuration::from_nanosecond_difference(end_ns.0, start_ns.0)?;
        // 8. Assert: TimeDurationSign(daySpan) = sign.
        // 9. Let unitLength be the value in the "Length in Nanoseconds" column of the row of Table 21 whose "Value" column contains unit.
        let unit_length = options.smallest_unit.as_nanoseconds().temporal_unwrap()?;
        // 10. Let roundedTimeDuration be ? RoundTimeDurationToIncrement(duration.[[Time]], increment × unitLength, roundingMode).
        let rounded_time = self.norm.round_inner(
            unsafe {
                NonZeroU128::new_unchecked(unit_length.into())
                    .checked_mul(options.increment.as_extended_increment())
                    .temporal_unwrap()?
            },
            options.rounding_mode,
        )?;
        // 11. Let beyondDaySpan be ! AddTimeDuration(roundedTimeDuration, -daySpan).
        let beyond_day_span = rounded_time.checked_add(day_span.negate().0)?;
        // 12. If TimeDurationSign(beyondDaySpan) ≠ -sign, then
        let (expanded, day_delta, rounded_time, nudge_ns) =
            if beyond_day_span.sign() != sign.negate() {
                // a. Let didRoundBeyondDay be true.
                // b. Let dayDelta be sign.
                // c. Set roundedTimeDuration to ? RoundTimeDurationToIncrement(beyondDaySpan, increment × unitLength, roundingMode).
                let rounded_time = self.norm.round_inner(
                    unsafe {
                        NonZeroU128::new_unchecked(unit_length.into())
                            .checked_mul(options.increment.as_extended_increment())
                            .temporal_unwrap()?
                    },
                    options.rounding_mode,
                )?;
                // d. Let nudgedEpochNs be AddTimeDurationToEpochNanoseconds(roundedTimeDuration, endEpochNs).
                let nudged_ns = rounded_time.checked_add(end_ns.0)?;
                (true, sign as i8, rounded_time, nudged_ns)
            // 13. Else,
            } else {
                // a. Let didRoundBeyondDay be false.
                // b. Let dayDelta be 0.
                // c. Let nudgedEpochNs be AddTimeDurationToEpochNanoseconds(roundedTimeDuration, startEpochNs).
                let nudge_ns = rounded_time.checked_add(start_ns.0)?;
                (false, 0, rounded_time, nudge_ns)
            };
        // 14. Let dateDuration be ! AdjustDateDurationRecord(duration.[[Date]], duration.[[Date]].[[Days]] + dayDelta).
        let date = DateDuration::new(
            self.date.years,
            self.date.months,
            self.date.weeks,
            self.date.days.checked_add(&day_delta.into())?,
        )?;
        // 15. Let resultDuration be CombineDateAndTimeDuration(dateDuration, roundedTimeDuration).
        let normalized = NormalizedDurationRecord::new(date, rounded_time)?;
        // 16. Return Duration Nudge Result Record { [[Duration]]: resultDuration, [[NudgedEpochNs]]: nudgedEpochNs, [[DidExpandCalendarUnit]]: didRoundBeyondDay }.
        Ok(NudgeRecord {
            normalized,
            nudge_epoch_ns: nudge_ns.0,
            total: None,
            expanded,
        })
    }

    #[inline]
    fn nudge_to_day_or_time(
        &self,
        dest_epoch_ns: i128,
        options: ResolvedRoundingOptions,
    ) -> TemporalResult<NudgeRecord> {
        // 1. Assert: The value in the "Category" column of the row of Table 22 whose "Singular" column contains smallestUnit, is time.
        // 2. Let norm be ! Add24HourDaysToNormalizedTimeDuration(duration.[[NormalizedTime]], duration.[[Days]]).
        let norm = self
            .normalized_time_duration()
            .add_days(self.date().days.as_())?;

        // 3. Let unitLength be the value in the "Length in Nanoseconds" column of the row of Table 22 whose "Singular" column contains smallestUnit.
        let unit_length = options.smallest_unit.as_nanoseconds().temporal_unwrap()?;
        // 4. Let total be DivideNormalizedTimeDuration(norm, unitLength).
        let total = norm.divide(unit_length as i64);

        // 5. Let roundedNorm be ? RoundNormalizedTimeDurationToIncrement(norm, unitLength × increment, roundingMode).
        let rounded_norm = norm.round_inner(
            unsafe {
                NonZeroU128::new_unchecked(unit_length.into())
                    .checked_mul(options.increment.as_extended_increment())
                    .temporal_unwrap()?
            },
            options.rounding_mode,
        )?;

        // 6. Let diffNorm be ! SubtractNormalizedTimeDuration(roundedNorm, norm).
        let diff_norm = rounded_norm.checked_sub(&norm)?;

        // 7. Let wholeDays be truncate(DivideNormalizedTimeDuration(norm, nsPerDay)).
        let whole_days = norm.divide(NS_PER_DAY as i64);

        // 8. Let roundedFractionalDays be DivideNormalizedTimeDuration(roundedNorm, nsPerDay).
        let (rounded_whole_days, rounded_remainder) = rounded_norm.div_rem(NS_PER_DAY);

        // 9. Let roundedWholeDays be truncate(roundedFractionalDays).
        // 10. Let dayDelta be roundedWholeDays - wholeDays.
        let delta = rounded_whole_days - whole_days;
        // 11. If dayDelta < 0, let dayDeltaSign be -1; else if dayDelta > 0, let dayDeltaSign be 1; else let dayDeltaSign be 0.
        // 12. If dayDeltaSign = NormalizedTimeDurationSign(norm), let didExpandDays be true; else let didExpandDays be false.
        let did_expand_days = delta.signum() as i8 == norm.sign() as i8;

        // 13. Let nudgedEpochNs be AddNormalizedTimeDurationToEpochNanoseconds(diffNorm, destEpochNs).
        let nudged_ns = diff_norm.0 + dest_epoch_ns;

        // 14. Let days be 0.
        let mut days = 0;
        // 15. Let remainder be roundedNorm.
        let mut remainder = rounded_norm;
        // 16. If LargerOfTwoTemporalUnits(largestUnit, "day") is largestUnit, then
        if options.largest_unit.max(TemporalUnit::Day) == options.largest_unit {
            // a. Set days to roundedWholeDays.
            days = rounded_whole_days;
            // b. Set remainder to remainder(roundedFractionalDays, 1) × nsPerDay.
            remainder = NormalizedTimeDuration(rounded_remainder);
        }
        // 17. Let resultDuration be ? CreateNormalizedDurationRecord(duration.[[Years]], duration.[[Months]], duration.[[Weeks]], days, remainder).
        let result_duration = NormalizedDurationRecord::new(
            DateDuration::new(
                self.date().years,
                self.date().months,
                self.date().weeks,
                FiniteF64::try_from(days)?,
            )?,
            remainder,
        )?;
        // 18. Return Duration Nudge Result Record { [[Duration]]: resultDuration, [[Total]]: total,
        // [[NudgedEpochNs]]: nudgedEpochNs, [[DidExpandCalendarUnit]]: didExpandDays }.
        Ok(NudgeRecord {
            normalized: result_duration,
            total: Some(total),
            nudge_epoch_ns: nudged_ns,
            expanded: did_expand_days,
        })
    }

    // 7.5.43 BubbleRelativeDuration ( sign, duration, nudgedEpochNs, dateTime, calendarRec, timeZoneRec, largestUnit, smallestUnit )
    #[inline]
    #[allow(clippy::too_many_arguments)]
    fn bubble_relative_duration(
        &self,
        sign: Sign,
        nudge_epoch_ns: i128,
        date_time: &PlainDateTime,
        tz: Option<(&TimeZone, &impl TzProvider)>,
        largest_unit: TemporalUnit,
        smallest_unit: TemporalUnit,
    ) -> TemporalResult<NormalizedDurationRecord> {
        // Assert: The value in the "Category" column of the row of Table 22 whose "Singular" column contains largestUnit, is date.
        // 2. Assert: The value in the "Category" column of the row of Table 22 whose "Singular" column contains smallestUnit, is date.
        let mut duration = *self;
        // 3. If smallestUnit is "year", return duration.
        if smallest_unit == TemporalUnit::Year {
            return Ok(duration);
        }

        // NOTE: Invert ops as Temporal Proposal table is inverted (i.e. Year = 0 ... Nanosecond = 9)
        // 4. Let largestUnitIndex be the ordinal index of the row of Table 22 whose "Singular" column contains largestUnit.
        // 5. Let smallestUnitIndex be the ordinal index of the row of Table 22 whose "Singular" column contains smallestUnit.
        // 6. Let unitIndex be smallestUnitIndex - 1.
        let mut unit = smallest_unit + 1;
        // 7. Let done be false.
        // 8. Repeat, while unitIndex ≤ largestUnitIndex and done is false,
        while unit != TemporalUnit::Auto && unit <= largest_unit {
            // a. Let unit be the value in the "Singular" column of Table 22 in the row whose ordinal index is unitIndex.
            // b. If unit is not "week", or largestUnit is "week", then
            if unit == TemporalUnit::Week || largest_unit != TemporalUnit::Week {
                unit = unit + 1;
                continue;
            }

            let end_duration = match unit {
                // i. If unit is "year", then
                TemporalUnit::Year => {
                    // 1. Let years be duration.[[Years]] + sign.
                    // 2. Let endDuration be ? CreateNormalizedDurationRecord(years, 0, 0, 0, ZeroTimeDuration()).
                    DateDuration::new(
                        duration
                            .date()
                            .years
                            .checked_add(&FiniteF64::from(sign.as_sign_multiplier()))?,
                        FiniteF64::default(),
                        FiniteF64::default(),
                        FiniteF64::default(),
                    )?
                }
                // ii. Else if unit is "month", then
                TemporalUnit::Month => {
                    // 1. Let months be duration.[[Months]] + sign.
                    // 2. Let endDuration be ? CreateNormalizedDurationRecord(duration.[[Years]], months, 0, 0, ZeroTimeDuration()).
                    DateDuration::new(
                        duration.date().years,
                        duration
                            .date()
                            .months
                            .checked_add(&FiniteF64::from(sign.as_sign_multiplier()))?,
                        FiniteF64::default(),
                        FiniteF64::default(),
                    )?
                }
                // iii. Else if unit is "week", then
                TemporalUnit::Week => {
                    // 1. Let weeks be duration.[[Weeks]] + sign.
                    // 2. Let endDuration be ? CreateNormalizedDurationRecord(duration.[[Years]], duration.[[Months]], weeks, 0, ZeroTimeDuration()).
                    DateDuration::new(
                        duration.date().years,
                        duration.date().months,
                        duration
                            .date()
                            .weeks
                            .checked_add(&FiniteF64::from(sign.as_sign_multiplier()))?,
                        FiniteF64::default(),
                    )?
                }
                // iv. Else,
                TemporalUnit::Day => {
                    // 1. Assert: unit is "day".
                    // 2. Let days be duration.[[Days]] + sign.
                    // 3. Let endDuration be ? CreateNormalizedDurationRecord(duration.[[Years]], duration.[[Months]], duration.[[Weeks]], days, ZeroTimeDuration()).
                    DateDuration::new(
                        duration.date().years,
                        duration.date().months,
                        duration.date().weeks,
                        duration
                            .date()
                            .days
                            .checked_add(&FiniteF64::from(sign.as_sign_multiplier()))?,
                    )?
                }
                _ => unreachable!(),
            };

            // v. Let end be ? AddDateTime(dateTime.[[Year]], dateTime.[[Month]], dateTime.[[Day]], dateTime.[[Hour]], dateTime.[[Minute]],
            // dateTime.[[Second]], dateTime.[[Millisecond]], dateTime.[[Microsecond]], dateTime.[[Nanosecond]], calendarRec,
            // endDuration.[[Years]], endDuration.[[Months]], endDuration.[[Weeks]], endDuration.[[Days]], endDuration.[[NormalizedTime]], undefined).
            let end = date_time.iso.add_date_duration(
                date_time.calendar().clone(),
                &end_duration,
                NormalizedTimeDuration::default(),
                None,
            )?;

            // vi. If timeZoneRec is unset, then
            let end_epoch_ns = if let Some((timezone, provider)) = tz {
                // 1. Let endDateTime be ! CreateTemporalDateTime(end.[[Year]], end.[[Month]], end.[[Day]],
                // end.[[Hour]], end.[[Minute]], end.[[Second]], end.[[Millisecond]], end.[[Microsecond]],
                // end.[[Nanosecond]], calendarRec.[[Receiver]]).
                // 2. Let endInstant be ? GetInstantFor(timeZoneRec, endDateTime, "compatible").
                timezone.get_epoch_nanoseconds_for(end, Disambiguation::Compatible, provider)?
                // 3. Let endEpochNs be endInstant.[[Nanoseconds]].
                // vii. Else,
            } else {
                // 1. Let endEpochNs be GetUTCEpochNanoseconds(end.[[Year]], end.[[Month]], end.[[Day]], end.[[Hour]],
                // end.[[Minute]], end.[[Second]], end.[[Millisecond]], end.[[Microsecond]], end.[[Nanosecond]]).
                end.as_nanoseconds()?
            };
            // viii. Let beyondEnd be nudgedEpochNs - endEpochNs.
            let beyond_end = nudge_epoch_ns - end_epoch_ns.0;
            // ix. If beyondEnd < 0, let beyondEndSign be -1; else if beyondEnd > 0, let beyondEndSign be 1; else let beyondEndSign be 0.
            // x. If beyondEndSign ≠ -sign, then
            if beyond_end.signum() != -i128::from(sign.as_sign_multiplier()) {
                // 1. Set duration to endDuration.
                duration = NormalizedDurationRecord::from_date_duration(end_duration)?;
            // xi. Else,
            } else {
                // 1. Set done to true.
                break;
            }
            // c. Set unitIndex to unitIndex - 1.
            unit = unit + 1;
        }

        Ok(duration)
    }

    // TODO: Potentially revisit and optimize
    // 7.5.44 RoundRelativeDuration ( duration, destEpochNs, dateTime, calendarRec, timeZoneRec, largestUnit, increment, smallestUnit, roundingMode )
    #[inline]
    pub(crate) fn round_relative_duration(
        &self,
        dest_epoch_ns: i128,
        dt: &PlainDateTime,
        timezone_record: Option<(&TimeZone, &impl TzProvider)>,
        options: ResolvedRoundingOptions,
    ) -> TemporalResult<NormalizedDurationRecord> {
        println!("Round relative called");
        // 1. Let irregularLengthUnit be false.
        // 2. If IsCalendarUnit(smallestUnit) is true, set irregularLengthUnit to true.
        // 3. If timeZoneRec is not unset and smallestUnit is "day", set irregularLengthUnit to true.
        let irregular_unit = options.smallest_unit.is_calendar_unit()
            || (timezone_record.is_some() && options.smallest_unit == TemporalUnit::Day);

        // 4. If DurationSign(duration.[[Years]], duration.[[Months]], duration.[[Weeks]], duration.[[Days]], NormalizedTimeDurationSign(duration.[[NormalizedTime]]), 0, 0, 0, 0, 0) < 0, let sign be -1; else let sign be 1.
        let sign = self.sign()?;

        // 5. If irregularLengthUnit is true, then
        let nudge_result = if irregular_unit {
            // a. Let nudgeResult be ? NudgeToCalendarUnit(sign, duration, destEpochNs, dateTime, calendarRec, timeZoneRec, increment, smallestUnit, roundingMode).
            self.nudge_calendar_unit(sign, dest_epoch_ns, dt, timezone_record, options)?
        // 6. Else if timeZoneRec is not unset, then
        } else if let Some((tz, provider)) = timezone_record {
            // a. Let nudgeResult be ? NudgeToZonedTime(sign, duration, dateTime, calendarRec, timeZoneRec, increment, smallestUnit, roundingMode).
            self.nudge_to_zoned_time(sign, dt, tz, options, provider)?
        // 7. Else,
        } else {
            // a. Let nudgeResult be ? NudgeToDayOrTime(duration, destEpochNs, largestUnit, increment, smallestUnit, roundingMode).
            self.nudge_to_day_or_time(dest_epoch_ns, options)?
        };

        println!("nudgeresult: {nudge_result:?}");
        // 8. Set duration to nudgeResult.[[Duration]].
        let mut duration = nudge_result.normalized;

        // 9. If nudgeResult.[[DidExpandCalendarUnit]] is true and smallestUnit is not "week", then
        if nudge_result.expanded && options.smallest_unit != TemporalUnit::Week {
            // a. Let startUnit be LargerOfTwoTemporalUnits(smallestUnit, "day").
            let start_unit = options.smallest_unit.max(TemporalUnit::Day);
            // b. Set duration to ? BubbleRelativeDuration(sign, duration, nudgeResult.[[NudgedEpochNs]], dateTime, calendarRec, timeZoneRec, largestUnit, startUnit).
            duration = duration.bubble_relative_duration(
                sign,
                nudge_result.nudge_epoch_ns,
                dt,
                timezone_record,
                options.largest_unit,
                start_unit,
            )?
        };

        Ok(duration)
    }
}

mod tests {
    #[test]
    fn validate_seconds_cast() {
        let max_seconds = super::MAX_TIME_DURATION.div_euclid(1_000_000_000);
        assert!(max_seconds <= i64::MAX.into())
    }

    // TODO: test f64 cast.
}
