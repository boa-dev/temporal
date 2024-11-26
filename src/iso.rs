//! This module implements the internal ISO field slots.
//!
//! The three main types of slots are:
//!   - `IsoDateTime`
//!   - `IsoDate`
//!   - `IsoTime`
//!
//! An `IsoDate` represents the `[[ISOYear]]`, `[[ISOMonth]]`, and `[[ISODay]]` internal slots.
//!
//! An `IsoTime` represents the `[[ISOHour]]`, `[[ISOMinute]]`, `[[ISOsecond]]`, `[[ISOmillisecond]]`,
//! `[[ISOmicrosecond]]`, and `[[ISOnanosecond]]` internal slots.
//!
//! An `IsoDateTime` has the internal slots of both an `IsoDate` and `IsoTime`.

use alloc::string::ToString;
use core::num::NonZeroU128;

use crate::{
    components::{
        calendar::Calendar,
        duration::{
            normalized::{NormalizedDurationRecord, NormalizedTimeDuration},
            DateDuration, TimeDuration,
        },
        Duration, PartialTime, PlainDate,
    }, error::TemporalError, options::{ArithmeticOverflow, ResolvedRoundingOptions, TemporalUnit}, primitive::FiniteF64, rounding::{IncrementRounder, Round}, temporal_assert, time::EpochNanoseconds, utils, TemporalResult, TemporalUnwrap, NS_PER_DAY
};
use icu_calendar::{Date as IcuDate, Iso};
use num_traits::{cast::FromPrimitive, AsPrimitive, ToPrimitive};

/// `IsoDateTime` is the record of the `IsoDate` and `IsoTime` internal slots.
#[non_exhaustive]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct IsoDateTime {
    pub date: IsoDate,
    pub time: IsoTime,
}

impl IsoDateTime {
    /// Creates a new `IsoDateTime` without any validaiton.
    pub(crate) fn new_unchecked(date: IsoDate, time: IsoTime) -> Self {
        Self { date, time }
    }

    /// Creates a new validated `IsoDateTime` that is within valid limits.
    pub fn new(date: IsoDate, time: IsoTime) -> TemporalResult<Self> {
        if !iso_dt_within_valid_limits(date, &time) {
            return Err(
                TemporalError::range().with_message("IsoDateTime not within a valid range.")
            );
        }
        Ok(Self::new_unchecked(date, time))
    }

    // NOTE: The below assumes that nanos is from an `Instant` and thus in a valid range. -> Needs validation.
    //
    // TODO: Move away from offset use of f64
    /// Creates an `IsoDateTime` from a `BigInt` of epochNanoseconds.
    #[allow(clippy::neg_cmp_op_on_partial_ord)]
    pub(crate) fn from_epoch_nanos(nanos: &i128, offset: i64) -> TemporalResult<Self> {
        // Skip the assert as nanos should be validated by Instant.
        // TODO: Determine whether value needs to be validated as integral.
        // Get the component ISO parts
        let mathematical_nanos = nanos.to_i64().ok_or_else(|| {
            TemporalError::range().with_message("nanos was not within a valid range.")
        })?;

        // 2. Let remainderNs be epochNanoseconds modulo 10^6.
        let remainder_nanos = mathematical_nanos % 1_000_000;

        // 3. Let epochMilliseconds be 𝔽((epochNanoseconds - remainderNs) / 10^6).
        let epoch_millis = (mathematical_nanos - remainder_nanos) / 1_000_000;

        let year = utils::epoch_time_to_epoch_year(epoch_millis as f64);
        let month = utils::epoch_time_to_month_in_year(epoch_millis as f64) + 1;
        let day = utils::epoch_time_to_date(epoch_millis as f64);

        // 7. Let hour be ℝ(! HourFromTime(epochMilliseconds)).
        let hour = (epoch_millis / 3_600_000) % 24;
        // 8. Let minute be ℝ(! MinFromTime(epochMilliseconds)).
        let minute = (epoch_millis / 60_000) % 60;
        // 9. Let second be ℝ(! SecFromTime(epochMilliseconds)).
        let second = (epoch_millis / 1000) % 60;
        // 10. Let millisecond be ℝ(! msFromTime(epochMilliseconds)).
        let millis = (epoch_millis % 1000) % 1000;

        // 11. Let microsecond be floor(remainderNs / 1000).
        let micros = remainder_nanos / 1000;
        // 12. Assert: microsecond < 1000.
        temporal_assert!(micros < 1000);
        // 13. Let nanosecond be remainderNs modulo 1000.
        let nanos = remainder_nanos % 1000;

        Ok(Self::balance(
            year,
            i32::from(month),
            i32::from(day),
            hour,
            minute,
            second,
            millis,
            micros,
            nanos + offset,
        ))
    }

    #[allow(clippy::too_many_arguments)]
    pub fn balance(
        year: i32,
        month: i32,
        day: i32,
        hour: i64,
        minute: i64,
        second: i64,
        millisecond: i64,
        microsecond: i64,
        nanosecond: i64,
    ) -> Self {
        let (overflow_day, time) =
            IsoTime::balance(hour, minute, second, millisecond, microsecond, nanosecond);
        let date = IsoDate::balance(year, month, day + overflow_day);
        Self::new_unchecked(date, time)
    }

    /// Returns whether the `IsoDateTime` is within valid limits.
    pub(crate) fn is_within_limits(&self) -> bool {
        iso_dt_within_valid_limits(self.date, &self.time)
    }

    /// Returns this `IsoDateTime` in nanoseconds
    pub fn as_nanoseconds(&self) -> TemporalResult<EpochNanoseconds> {
        utc_epoch_nanos(self.date, &self.time)
    }

    /// Specification equivalent to 5.5.9 `AddDateTime`.
    pub(crate) fn add_date_duration(
        &self,
        calendar: Calendar,
        date_duration: &DateDuration,
        norm: NormalizedTimeDuration,
        overflow: Option<ArithmeticOverflow>,
    ) -> TemporalResult<Self> {
        // 1. Assert: IsValidISODate(year, month, day) is true.
        // 2. Assert: ISODateTimeWithinLimits(year, month, day, hour, minute, second, millisecond, microsecond, nanosecond) is true.
        // 3. Let timeResult be AddTime(hour, minute, second, millisecond, microsecond, nanosecond, norm).
        let t_result = self.time.add(norm);

        // 4. Let datePart be ! CreateTemporalDate(year, month, day, calendarRec.[[Receiver]]).
        let date = PlainDate::new_unchecked(self.date, calendar);

        // 5. Let dateDuration be ? CreateTemporalDuration(years, months, weeks, days + timeResult.[[Days]], 0, 0, 0, 0, 0, 0).
        let date_duration = DateDuration::new(
            date_duration.years,
            date_duration.months,
            date_duration.weeks,
            date_duration
                .days
                .checked_add(&FiniteF64::from(t_result.0))?,
        )?;
        let duration = Duration::from(date_duration);

        // 6. Let addedDate be ? AddDate(calendarRec, datePart, dateDuration, options).
        let added_date = date.add_date(&duration, overflow)?;

        // 7. Return ISO Date-Time Record { [[Year]]: addedDate.[[ISOYear]], [[Month]]: addedDate.[[ISOMonth]],
        // [[Day]]: addedDate.[[ISODay]], [[Hour]]: timeResult.[[Hour]], [[Minute]]: timeResult.[[Minute]],
        // [[Second]]: timeResult.[[Second]], [[Millisecond]]: timeResult.[[Millisecond]],
        // [[Microsecond]]: timeResult.[[Microsecond]], [[Nanosecond]]: timeResult.[[Nanosecond]]  }.
        Ok(Self::new_unchecked(added_date.iso, t_result.1))
    }

    pub(crate) fn round(&self, resolved_options: ResolvedRoundingOptions) -> TemporalResult<Self> {
        let (rounded_days, rounded_time) = self.time.round(resolved_options)?;
        let balance_result = IsoDate::balance(
            self.date.year,
            self.date.month.into(),
            i32::from(self.date.day) + rounded_days,
        );
        Self::new(balance_result, rounded_time)
    }

    // TODO: Determine whether to provide an options object...seems duplicative.
    /// 5.5.11 DifferenceISODateTime ( y1, mon1, d1, h1, min1, s1, ms1, mus1, ns1, y2, mon2, d2, h2, min2, s2, ms2, mus2, ns2, calendarRec, largestUnit, options )
    pub(crate) fn diff(
        &self,
        other: &Self,
        calendar: &Calendar,
        largest_unit: TemporalUnit,
    ) -> TemporalResult<NormalizedDurationRecord> {
        // 1. Assert: ISODateTimeWithinLimits(y1, mon1, d1, h1, min1, s1, ms1, mus1, ns1) is true.
        // 2. Assert: ISODateTimeWithinLimits(y2, mon2, d2, h2, min2, s2, ms2, mus2, ns2) is true.
        // 3. Assert: If y1 ≠ y2, and mon1 ≠ mon2, and d1 ≠ d2, and LargerOfTwoTemporalUnits(largestUnit, "day")
        // is not "day", CalendarMethodsRecordHasLookedUp(calendarRec, date-until) is true.

        // 4. Let timeDuration be DifferenceTime(h1, min1, s1, ms1, mus1, ns1, h2, min2, s2, ms2, mus2, ns2).
        let mut time_duration =
            NormalizedTimeDuration::from_time_duration(&self.time.diff(&other.time));

        // 5. Let timeSign be NormalizedTimeDurationSign(timeDuration).
        let time_sign = time_duration.sign() as i8;

        // 6. Let dateSign be CompareISODate(y2, mon2, d2, y1, mon1, d1).
        let date_sign = other.date.cmp(&self.date) as i32;
        // 7. Let adjustedDate be CreateISODateRecord(y2, mon2, d2).
        let mut adjusted_date = other.date;

        // 8. If timeSign = -dateSign, then
        if i32::from(time_sign) == -date_sign {
            // a. Set adjustedDate to BalanceISODate(adjustedDate.[[Year]], adjustedDate.[[Month]], adjustedDate.[[Day]] + timeSign).
            adjusted_date = IsoDate::balance(
                adjusted_date.year,
                i32::from(adjusted_date.month),
                i32::from(adjusted_date.day) + i32::from(time_sign),
            );
            // b. Set timeDuration to ? Add24HourDaysToNormalizedTimeDuration(timeDuration, -timeSign).
            time_duration = time_duration.add_days(-i64::from(time_sign))?;
        }

        // 9. Let date1 be ! CreateTemporalDate(y1, mon1, d1, calendarRec.[[Receiver]]).
        let date_one = PlainDate::new_unchecked(self.date, calendar.clone());
        // 10. Let date2 be ! CreateTemporalDate(adjustedDate.[[Year]], adjustedDate.[[Month]],
        // adjustedDate.[[Day]], calendarRec.[[Receiver]]).
        let date_two = PlainDate::try_new(
            adjusted_date.year,
            adjusted_date.month.into(),
            adjusted_date.day.into(),
            calendar.clone(),
        )?;

        // 11. Let dateLargestUnit be LargerOfTwoTemporalUnits("day", largestUnit).
        // 12. Let untilOptions be ! SnapshotOwnProperties(options, null).
        let date_largest_unit = largest_unit.max(TemporalUnit::Day);

        // 13. Perform ! CreateDataPropertyOrThrow(untilOptions, "largestUnit", dateLargestUnit).
        // 14. Let dateDifference be ? DifferenceDate(calendarRec, date1, date2, untilOptions).
        let date_diff = date_one.internal_diff_date(&date_two, date_largest_unit)?;

        // 16. If largestUnit is not dateLargestUnit, then
        let days = if largest_unit == date_largest_unit {
            // 15. Let days be dateDifference.[[Days]].
            date_diff.days()
        } else {
            // a. Set timeDuration to ? Add24HourDaysToNormalizedTimeDuration(timeDuration, dateDifference.[[Days]]).
            time_duration = time_duration.add_days(date_diff.days().as_())?;
            // b. Set days to 0.
            FiniteF64::default()
        };

        // 17. Return ? CreateNormalizedDurationRecord(dateDifference.[[Years]], dateDifference.[[Months]], dateDifference.[[Weeks]], days, timeDuration).
        NormalizedDurationRecord::new(
            DateDuration::new_unchecked(
                date_diff.years(),
                date_diff.months(),
                date_diff.weeks(),
                days,
            ),
            time_duration,
        )
    }
}

// ==== `IsoDate` section ====

/// A trait for accessing the `IsoDate` across the various Temporal objects
pub trait IsoDateSlots {
    /// Returns the target's internal `IsoDate`.
    fn iso_date(&self) -> IsoDate;
}

/// `IsoDate` serves as a record for the `[[ISOYear]]`, `[[ISOMonth]]`,
/// and `[[ISODay]]` internal fields.
///
/// These fields are used for the `Temporal.PlainDate` object, the
/// `Temporal.YearMonth` object, and the `Temporal.MonthDay` object.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct IsoDate {
    pub year: i32,
    pub month: u8,
    pub day: u8,
}

impl IsoDate {
    /// Creates a new `IsoDate` without determining the validity.
    pub(crate) const fn new_unchecked(year: i32, month: u8, day: u8) -> Self {
        Self { year, month, day }
    }

    pub(crate) fn new_with_overflow(
        year: i32,
        month: i32,
        day: i32,
        overflow: ArithmeticOverflow,
    ) -> TemporalResult<Self> {
        let id = match overflow {
            ArithmeticOverflow::Constrain => {
                let month = month.clamp(1, 12);
                let day = constrain_iso_day(year, month, day);
                // NOTE: Values are clamped in a u8 range.
                Self::new_unchecked(year, month as u8, day)
            }
            ArithmeticOverflow::Reject => {
                if !is_valid_date(year, month, day) {
                    return Err(TemporalError::range().with_message("not a valid ISO date."));
                }
                // NOTE: Values have been verified to be in a u8 range.
                Self::new_unchecked(year, month as u8, day as u8)
            }
        };

        if !iso_dt_within_valid_limits(id, &IsoTime::noon()) {
            return Err(
                TemporalError::range().with_message("Date is not within ISO date time limits.")
            );
        }

        Ok(id)
    }

    /// Create a balanced `IsoDate`
    ///
    /// Equivalent to `BalanceISODate`.
    pub(crate) fn balance(year: i32, month: i32, day: i32) -> Self {
        let epoch_days = iso_date_to_epoch_days(year, month - 1, day);
        let ms = utils::epoch_days_to_epoch_ms(epoch_days, 0f64);
        Self::new_unchecked(
            utils::epoch_time_to_epoch_year(ms),
            utils::epoch_time_to_month_in_year(ms) + 1,
            utils::epoch_time_to_date(ms),
        )
    }

    pub(crate) fn is_valid_day_range(&self) -> TemporalResult<()> {
        if self.to_epoch_days().abs() > 100_000_000 {
            return Err(TemporalError::range().with_message("Not in a valid ISO day range."));
        }
        Ok(())
    }

    /// Returns this `IsoDate` in nanoseconds.
    #[inline]
    pub(crate) fn as_nanoseconds(&self) -> TemporalResult<EpochNanoseconds> {
        utc_epoch_nanos(*self, &IsoTime::default())
    }

    /// Functionally the same as Date's abstract operation `MakeDay`
    ///
    /// Equivalent to `IsoDateToEpochDays`
    #[inline]
    pub(crate) fn to_epoch_days(self) -> i32 {
        iso_date_to_epoch_days(self.year, (self.month - 1).into(), self.day.into())
    }

    /// Returns if the current `IsoDate` is valid.
    pub(crate) fn is_valid(self) -> bool {
        is_valid_date(self.year, self.month.into(), self.day.into())
    }

    /// Returns the resulting `IsoDate` from adding a provided `Duration` to this `IsoDate`
    pub(crate) fn add_date_duration(
        self,
        duration: &DateDuration,
        overflow: ArithmeticOverflow,
    ) -> TemporalResult<Self> {
        // 1. Assert: year, month, day, years, months, weeks, and days are integers.
        // 2. Assert: overflow is either "constrain" or "reject".
        // 3. Let intermediate be ! BalanceISOYearMonth(year + years, month + months).
        let intermediate = balance_iso_year_month(
            self.year + duration.years.as_date_value()?,
            i32::from(self.month) + duration.months.as_date_value()?,
        );

        // 4. Let intermediate be ? RegulateISODate(intermediate.[[Year]], intermediate.[[Month]], day, overflow).
        let intermediate = Self::new_with_overflow(
            intermediate.0,
            intermediate.1,
            i32::from(self.day),
            overflow,
        )?;

        // 5. Set days to days + 7 × weeks.
        let additional_days =
            duration.days.as_date_value()? + (duration.weeks.as_date_value()? * 7);
        // 6. Let d be intermediate.[[Day]] + days.
        let d = i32::from(intermediate.day) + additional_days;

        // 7. Return BalanceISODate(intermediate.[[Year]], intermediate.[[Month]], d).
        Ok(Self::balance(
            intermediate.year,
            intermediate.month.into(),
            d,
        ))
    }

    pub(crate) fn diff_iso_date(
        &self,
        other: &Self,
        largest_unit: TemporalUnit,
    ) -> TemporalResult<DateDuration> {
        // 1. Assert: IsValidISODate(y1, m1, d1) is true.
        // 2. Assert: IsValidISODate(y2, m2, d2) is true.
        // 3. Let sign be -CompareISODate(y1, m1, d1, y2, m2, d2).
        let sign = -(self.cmp(other) as i8);
        // 4. If sign = 0, return ! CreateDateDurationRecord(0, 0, 0, 0).
        if sign == 0 {
            return Ok(DateDuration::default());
        };

        // 5. Let years be 0.
        let mut years = 0;
        let mut months = 0;
        // 6. If largestUnit is "year", then
        if largest_unit == TemporalUnit::Year || largest_unit == TemporalUnit::Month {
            // others.year - self.year is adopted from temporal-proposal/polyfill as it saves iterations.
            // a. Let candidateYears be sign.
            let mut candidate_years: i32 = other.year - self.year;
            if candidate_years != 0 {
                candidate_years -= i32::from(sign);
            }
            // b. Repeat, while ISODateSurpasses(sign, y1 + candidateYears, m1, d1, y2, m2, d2) is false,
            while !iso_date_surpasses(
                &IsoDate::new_unchecked(self.year + candidate_years, self.month, self.day),
                other,
                sign,
            ) {
                // i. Set years to candidateYears.
                years = candidate_years;
                // ii. Set candidateYears to candidateYears + sign.
                candidate_years += i32::from(sign);
            }

            // 7. Let months be 0.
            // 8. If largestUnit is "year" or largestUnit is "month", then
            // a. Let candidateMonths be sign.
            let mut candidate_months: i32 = sign.into();
            // b. Let intermediate be BalanceISOYearMonth(y1 + years, m1 + candidateMonths).
            let mut intermediate =
                balance_iso_year_month(self.year + years, i32::from(self.month) + candidate_months);
            // c. Repeat, while ISODateSurpasses(sign, intermediate.[[Year]], intermediate.[[Month]], d1, y2, m2, d2) is false,
            // Safety: balance_iso_year_month should always return a month value from 1..=12
            while !iso_date_surpasses(
                &IsoDate::new_unchecked(intermediate.0, intermediate.1 as u8, self.day),
                other,
                sign,
            ) {
                // i. Set months to candidateMonths.
                months = candidate_months;
                // ii. Set candidateMonths to candidateMonths + sign.
                candidate_months += i32::from(sign);
                // iii. Set intermediate to BalanceISOYearMonth(intermediate.[[Year]], intermediate.[[Month]] + sign).
                intermediate =
                    balance_iso_year_month(intermediate.0, intermediate.1 + i32::from(sign));
            }
        }

        // 9. Set intermediate to BalanceISOYearMonth(y1 + years, m1 + months).
        let intermediate =
            balance_iso_year_month(self.year + years, i32::from(self.month) + months);
        // 10. Let constrained be ! RegulateISODate(intermediate.[[Year]], intermediate.[[Month]], d1, "constrain").
        let constrained = Self::new_with_overflow(
            intermediate.0,
            intermediate.1,
            self.day.into(),
            ArithmeticOverflow::Constrain,
        )?;

        // NOTE: Below is adapted from the polyfill. Preferring this as it avoids looping.
        // 11. Let weeks be 0.
        let days = iso_date_to_epoch_days(other.year, i32::from(other.month) - 1, other.day.into())
            - iso_date_to_epoch_days(
                constrained.year,
                i32::from(constrained.month) - 1,
                constrained.day.into(),
            );

        let (weeks, days) = if largest_unit == TemporalUnit::Week {
            (days / 7, days % 7)
        } else {
            (0, days)
        };

        // 17. Return ! CreateDateDurationRecord(years, months, weeks, days).
        DateDuration::new(
            FiniteF64::from(years),
            FiniteF64::from(months),
            FiniteF64::from(weeks),
            FiniteF64::from(days),
        )
    }
}

impl IsoDate {
    /// Creates `[[ISOYear]]`, `[[isoMonth]]`, `[[isoDay]]` fields from `ICU4X`'s `Date<Iso>` struct.
    pub(crate) fn as_icu4x(self) -> TemporalResult<IcuDate<Iso>> {
        IcuDate::try_new_iso_date(self.year, self.month, self.day)
            .map_err(|e| TemporalError::range().with_message(e.to_string()))
    }
}

// ==== `IsoTime` section ====

/// An `IsoTime` record that contains `Temporal`'s
/// time slots.
#[non_exhaustive]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct IsoTime {
    pub hour: u8,         // 0..=23
    pub minute: u8,       // 0..=59
    pub second: u8,       // 0..=59
    pub millisecond: u16, // 0..=999
    pub microsecond: u16, // 0..=999
    pub nanosecond: u16,  // 0..=999
}

impl IsoTime {
    /// Creates a new `IsoTime` without any validation.
    pub(crate) fn new_unchecked(
        hour: u8,
        minute: u8,
        second: u8,
        millisecond: u16,
        microsecond: u16,
        nanosecond: u16,
    ) -> Self {
        Self {
            hour,
            minute,
            second,
            millisecond,
            microsecond,
            nanosecond,
        }
    }

    /// Creates a new regulated `IsoTime`.
    pub fn new(
        hour: i32,
        minute: i32,
        second: i32,
        millisecond: i32,
        microsecond: i32,
        nanosecond: i32,
        overflow: ArithmeticOverflow,
    ) -> TemporalResult<IsoTime> {
        match overflow {
            ArithmeticOverflow::Constrain => {
                let h = hour.clamp(0, 23) as u8;
                let min = minute.clamp(0, 59) as u8;
                let sec = second.clamp(0, 59) as u8;
                let milli = millisecond.clamp(0, 999) as u16;
                let micro = microsecond.clamp(0, 999) as u16;
                let nano = nanosecond.clamp(0, 999) as u16;
                Ok(Self::new_unchecked(h, min, sec, milli, micro, nano))
            }
            ArithmeticOverflow::Reject => {
                if !is_valid_time(hour, minute, second, millisecond, microsecond, nanosecond) {
                    return Err(TemporalError::range().with_message("IsoTime is not valid"));
                };
                Ok(Self::new_unchecked(
                    hour as u8,
                    minute as u8,
                    second as u8,
                    millisecond as u16,
                    microsecond as u16,
                    nanosecond as u16,
                ))
            }
        }
    }

    /// Creates a new `Time` with the fields provided from a `PartialTime`.
    #[inline]
    pub(crate) fn with(
        &self,
        partial: PartialTime,
        overflow: ArithmeticOverflow,
    ) -> TemporalResult<Self> {
        let hour = partial.hour.unwrap_or(self.hour.into());
        let minute = partial.minute.unwrap_or(self.minute.into());
        let second = partial.second.unwrap_or(self.second.into());
        let millisecond = partial.millisecond.unwrap_or(self.millisecond.into());
        let microsecond = partial.microsecond.unwrap_or(self.microsecond.into());
        let nanosecond = partial.nanosecond.unwrap_or(self.nanosecond.into());
        Self::new(
            hour,
            minute,
            second,
            millisecond,
            microsecond,
            nanosecond,
            overflow,
        )
    }

    /// Returns an `IsoTime` set to 12:00:00
    pub(crate) const fn noon() -> Self {
        Self {
            hour: 12,
            minute: 0,
            second: 0,
            millisecond: 0,
            microsecond: 0,
            nanosecond: 0,
        }
    }

    /// Returns an `IsoTime` based off parse components.
    pub(crate) fn from_components(
        hour: i32,
        minute: i32,
        second: i32,
        fraction: f64,
    ) -> TemporalResult<Self> {
        let millisecond = fraction * 1000f64;
        let micros = millisecond.rem_euclid(1f64) * 1000f64;
        let nanos = micros.rem_euclid(1f64).mul_add(1000f64, 0.5).floor();

        Self::new(
            hour,
            minute,
            second,
            millisecond as i32,
            micros as i32,
            nanos as i32,
            ArithmeticOverflow::Reject,
        )
    }

    // NOTE(nekevss): f64 is needed here as values could exceed i32 when input.
    /// Balances and creates a new `IsoTime` with `day` overflow from the provided values.
    pub(crate) fn balance(
        hour: i64,
        minute: i64,
        second: i64,
        millisecond: i64,
        microsecond: i64,
        nanosecond: i64,
    ) -> (i32, Self) {
        // 1. Set microsecond to microsecond + floor(nanosecond / 1000).
        // 2. Set nanosecond to nanosecond modulo 1000.
        let (quotient, nanosecond) = div_mod(nanosecond, 1000);
        let microsecond = microsecond + quotient;

        // 3. Set millisecond to millisecond + floor(microsecond / 1000).
        // 4. Set microsecond to microsecond modulo 1000.
        let (quotient, microsecond) = div_mod(microsecond, 1000);
        let millisecond = millisecond + quotient;

        // 5. Set second to second + floor(millisecond / 1000).
        // 6. Set millisecond to millisecond modulo 1000.
        let (quotient, millisecond) = div_mod(millisecond, 1000);
        let second = second + quotient;

        // 7. Set minute to minute + floor(second / 60).
        // 8. Set second to second modulo 60.
        let (quotient, second) = div_mod(second, 60);
        let minute = minute + quotient;

        // 9. Set hour to hour + floor(minute / 60).
        // 10. Set minute to minute modulo 60.
        let (quotient, minute) = div_mod(minute, 60);
        let hour = hour + quotient;

        // 11. Let days be floor(hour / 24).
        // 12. Set hour to hour modulo 24.
        let (days, hour) = div_mod(hour, 24);

        let time = Self::new_unchecked(
            hour as u8,
            minute as u8,
            second as u8,
            millisecond as u16,
            microsecond as u16,
            nanosecond as u16,
        );

        (days as i32, time)
    }

    /// Difference this `IsoTime` against another and returning a `TimeDuration`.
    pub(crate) fn diff(&self, other: &Self) -> TimeDuration {
        let h = i32::from(other.hour) - i32::from(self.hour);
        let m = i32::from(other.minute) - i32::from(self.minute);
        let s = i32::from(other.second) - i32::from(self.second);
        let ms = i32::from(other.millisecond) - i32::from(self.millisecond);
        let mis = i32::from(other.microsecond) - i32::from(self.microsecond);
        let ns = i32::from(other.nanosecond) - i32::from(self.nanosecond);

        TimeDuration::new_unchecked(
            FiniteF64::from(h),
            FiniteF64::from(m),
            FiniteF64::from(s),
            FiniteF64::from(ms),
            FiniteF64::from(mis),
            FiniteF64::from(ns),
        )
    }

    // NOTE (nekevss): Specification seemed to be off / not entirely working, so the below was adapted from the
    // temporal-polyfill
    // TODO: DayLengthNS can probably be a u64, but keep as is for now and optimize.
    /// Rounds the current `IsoTime` according to the provided settings.
    pub(crate) fn round(
        &self,
        resolved_options: ResolvedRoundingOptions,
    ) -> TemporalResult<(i32, Self)> {
        // 1. If unit is "day" or "hour", then
        let quantity = match resolved_options.smallest_unit {
            TemporalUnit::Day | TemporalUnit::Hour => {
                // a. Let quantity be ((((hour × 60 + minute) × 60 + second) × 1000 + millisecond)
                // × 1000 + microsecond) × 1000 + nanosecond.
                let minutes = i128::from(self.hour) * 60 + i128::from(self.minute);
                let seconds = minutes * 60 + i128::from(self.second);
                let millis = seconds * 1000 + i128::from(self.millisecond);
                let micros = millis * 1000 + i128::from(self.microsecond);
                micros * 1000 + i128::from(self.nanosecond)
            }
            // 2. Else if unit is "minute", then
            TemporalUnit::Minute => {
                // a. Let quantity be (((minute × 60 + second) × 1000 + millisecond) × 1000 + microsecond) × 1000 + nanosecond.
                let seconds = i128::from(self.minute) * 60 + i128::from(self.second);
                let millis = seconds * 1000 + i128::from(self.millisecond);
                let micros = millis * 1000 + i128::from(self.microsecond);
                micros * 1000 + i128::from(self.nanosecond)
            }
            // 3. Else if unit is "second", then
            TemporalUnit::Second => {
                // a. Let quantity be ((second × 1000 + millisecond) × 1000 + microsecond) × 1000 + nanosecond.
                let millis = i128::from(self.second) * 1000 + i128::from(self.millisecond);
                let micros = millis * 1000 + i128::from(self.microsecond);
                micros * 1000 + i128::from(self.nanosecond)
            }
            // 4. Else if unit is "millisecond", then
            TemporalUnit::Millisecond => {
                // a. Let quantity be (millisecond × 1000 + microsecond) × 1000 + nanosecond.
                let micros = i128::from(self.millisecond) * 1000 + i128::from(self.microsecond);
                micros * 1000 + i128::from(self.nanosecond)
            }
            // 5. Else if unit is "microsecond", then
            TemporalUnit::Microsecond => {
                // a. Let quantity be microsecond × 1000 + nanosecond.
                i128::from(self.microsecond) * 1000 + i128::from(self.nanosecond)
            }
            // 6. Else,
            TemporalUnit::Nanosecond => {
                // a. Assert: unit is "nanosecond".
                // b. Let quantity be nanosecond.
                i128::from(self.nanosecond)
            }
            _ => {
                return Err(TemporalError::range()
                    .with_message("Invalid smallestUnit value for time rounding."))
            }
        };
        // 7. Let unitLength be the value in the "Length in Nanoseconds" column of the row of Table 22 whose "Singular" column contains unit.
        let length = NonZeroU128::new(
            resolved_options
                .smallest_unit
                .as_nanoseconds()
                .temporal_unwrap()?
                .into(),
        )
        .temporal_unwrap()?;

        let increment = resolved_options
            .increment
            .as_extended_increment()
            .checked_mul(length)
            .ok_or(TemporalError::range().with_message("increment exceeded valid range."))?;

        // 8. Let result be RoundNumberToIncrement(quantity, increment × unitLength, roundingMode) / unitLength.
        let result =
            IncrementRounder::<i128>::from_potentially_negative_parts(quantity, increment)?
                .round(resolved_options.rounding_mode)
                / length.get() as i128;

        let result_i64 = i64::from_i128(result)
            .ok_or(TemporalError::range().with_message("round result valid range."))?;

        match resolved_options.smallest_unit {
            // 9. If unit is "day", then
            // a. Return Time Record { [[Days]]: result, [[Hour]]: 0, [[Minute]]: 0, [[Second]]: 0, [[Millisecond]]: 0, [[Microsecond]]: 0, [[Nanosecond]]: 0  }.
            TemporalUnit::Day => Ok((result_i64 as i32, Self::default())),
            // 10. If unit is "hour", then
            // a. Return BalanceTime(result, 0, 0, 0, 0, 0).
            TemporalUnit::Hour => Ok(Self::balance(result_i64, 0, 0, 0, 0, 0)),
            // 11. If unit is "minute", then
            // a. Return BalanceTime(hour, result, 0.0, 0.0, 0.0, 0).
            TemporalUnit::Minute => Ok(Self::balance(self.hour.into(), result_i64, 0, 0, 0, 0)),
            // 12. If unit is "second", then
            // a. Return BalanceTime(hour, minute, result, 0.0, 0.0, 0).
            TemporalUnit::Second => Ok(Self::balance(
                self.hour.into(),
                self.minute.into(),
                result_i64,
                0,
                0,
                0,
            )),
            // 13. If unit is "millisecond", then
            // a. Return BalanceTime(hour, minute, second, result, 0.0, 0).
            TemporalUnit::Millisecond => Ok(Self::balance(
                self.hour.into(),
                self.minute.into(),
                self.second.into(),
                result_i64,
                0,
                0,
            )),
            // 14. If unit is "microsecond", then
            // a. Return BalanceTime(hour, minute, second, millisecond, result, 0).
            TemporalUnit::Microsecond => Ok(Self::balance(
                self.hour.into(),
                self.minute.into(),
                self.second.into(),
                self.millisecond.into(),
                result_i64,
                0,
            )),
            // 15. Assert: unit is "nanosecond".
            // 16. Return BalanceTime(hour, minute, second, millisecond, microsecond, result).
            TemporalUnit::Nanosecond => Ok(Self::balance(
                self.hour.into(),
                self.minute.into(),
                self.second.into(),
                self.millisecond.into(),
                self.microsecond.into(),
                result_i64,
            )),
            _ => Err(TemporalError::assert()),
        }
    }

    /// Checks if the time is a valid `IsoTime`
    pub(crate) fn is_valid(&self) -> bool {
        if !(0..=23).contains(&self.hour) {
            return false;
        }

        let min_sec = 0..=59;
        if !min_sec.contains(&self.minute) || !min_sec.contains(&self.second) {
            return false;
        }

        let sub_second = 0..=999;
        sub_second.contains(&self.millisecond)
            && sub_second.contains(&self.microsecond)
            && sub_second.contains(&self.nanosecond)
    }

    pub(crate) fn add(&self, norm: NormalizedTimeDuration) -> (i32, Self) {
        // 1. Set second to second + NormalizedTimeDurationSeconds(norm).
        let seconds = i64::from(self.second) + norm.seconds();
        // 2. Set nanosecond to nanosecond + NormalizedTimeDurationSubseconds(norm).
        let nanos = i32::from(self.nanosecond) + norm.subseconds();
        // 3. Return BalanceTime(hour, minute, second, millisecond, microsecond, nanosecond).
        Self::balance(
            self.hour.into(),
            self.minute.into(),
            seconds,
            self.millisecond.into(),
            self.microsecond.into(),
            nanos.into(),
        )
    }

    /// `IsoTimeToEpochMs`
    ///
    /// Note: This method is library specific and not in spec
    ///
    /// Functionally the same as Date's `MakeTime`
    pub(crate) fn to_epoch_ms(self) -> f64 {
        ((f64::from(self.hour) * utils::MS_PER_HOUR
            + f64::from(self.minute) * utils::MS_PER_MINUTE)
            + f64::from(self.second) * 1000f64)
            + f64::from(self.millisecond)
    }
}

// ==== `IsoDateTime` specific utility functions ====

#[inline]
/// Utility function to determine if a `DateTime`'s components create a `DateTime` within valid limits
fn iso_dt_within_valid_limits(date: IsoDate, time: &IsoTime) -> bool {
    if iso_date_to_epoch_days(date.year, (date.month - 1).into(), date.day.into()).abs()
        > 100_000_001
    {
        return false;
    }
    let Ok(ns) = utc_epoch_nanos(date, time) else {
        return false;
    };

    let max = crate::NS_MAX_INSTANT + i128::from(NS_PER_DAY);
    let min = crate::NS_MIN_INSTANT - i128::from(NS_PER_DAY);

    min <= ns.0 && max >= ns.0
}

#[inline]
/// Utility function to convert a `IsoDate` and `IsoTime` values into epoch nanoseconds
fn utc_epoch_nanos(date: IsoDate, time: &IsoTime) -> TemporalResult<EpochNanoseconds> {
    let ms = time.to_epoch_ms();
    let epoch_ms = utils::epoch_days_to_epoch_ms(date.to_epoch_days(), ms);

    let epoch_nanos = epoch_ms.mul_add(
        1_000_000f64,
        f64::from(time.microsecond).mul_add(1_000f64, f64::from(time.nanosecond)),
    );

    EpochNanoseconds::try_from(epoch_nanos)
}

// ==== `IsoDate` specific utiltiy functions ====

/// Returns the Epoch days based off the given year, month, and day.
///
/// NOTE: Month should be in a range of 0-11
#[inline]
fn iso_date_to_epoch_days(year: i32, month: i32, day: i32) -> i32 {
    // 1. Let resolvedYear be year + floor(month / 12).
    let resolved_year = year + month.div_euclid(12);
    // 2. Let resolvedMonth be month modulo 12.
    let resolved_month = month.rem_euclid(12);

    // 3. Find a time t such that EpochTimeToEpochYear(t) is resolvedYear,
    // EpochTimeToMonthInYear(t) is resolvedMonth, and EpochTimeToDate(t) is 1.
    let year_t = utils::epoch_time_for_year(resolved_year);
    let month_t = utils::epoch_time_for_month_given_year(resolved_month, resolved_year);

    // 4. Return EpochTimeToDayNumber(t) + date - 1.
    utils::epoch_time_to_day_number((year_t + month_t).copysign(year_t)) + day - 1
}

#[inline]
// Determines if the month and day are valid for the given year.
fn is_valid_date(year: i32, month: i32, day: i32) -> bool {
    if !(1..=12).contains(&month) {
        return false;
    }
    is_valid_iso_day(year, month, day)
}

#[inline]
/// Returns with the `this` surpasses `other`.
fn iso_date_surpasses(this: &IsoDate, other: &IsoDate, sign: i8) -> bool {
    this.cmp(other) as i8 * sign == 1
}

#[inline]
fn balance_iso_year_month(year: i32, month: i32) -> (i32, i32) {
    // 1. Assert: year and month are integers.
    // 2. Set year to year + floor((month - 1) / 12).
    let y = year + (month - 1).div_euclid(12);
    // 3. Set month to ((month - 1) modulo 12) + 1.
    let m = (month - 1).rem_euclid(12) + 1;
    // 4. Return the Record { [[Year]]: year, [[Month]]: month  }.
    (y, m)
}

#[inline]
pub(crate) fn constrain_iso_day(year: i32, month: i32, day: i32) -> u8 {
    let days_in_month = utils::iso_days_in_month(year, month);
    day.clamp(1, days_in_month) as u8
}

#[inline]
pub(crate) fn is_valid_iso_day(year: i32, month: i32, day: i32) -> bool {
    let days_in_month = utils::iso_days_in_month(year, month);
    (1..=days_in_month).contains(&day)
}

// ==== `IsoTime` specific utilities ====

#[inline]
fn is_valid_time(hour: i32, minute: i32, second: i32, ms: i32, mis: i32, ns: i32) -> bool {
    if !(0..=23).contains(&hour) {
        return false;
    }

    let min_sec = 0..=59;
    if !min_sec.contains(&minute) || !min_sec.contains(&second) {
        return false;
    }

    let sub_second = 0..=999;
    sub_second.contains(&ms) && sub_second.contains(&mis) && sub_second.contains(&ns)
}

// NOTE(nekevss): Considering the below: Balance can probably be altered from f64.
#[inline]
fn div_mod(dividend: i64, divisor: i64) -> (i64, i64) {
    (dividend.div_euclid(divisor), dividend.rem_euclid(divisor))
}
