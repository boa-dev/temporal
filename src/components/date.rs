//! This module implements `Date` and any directly related algorithms.

use tinystr::TinyAsciiStr;

use crate::{
    components::{
        calendar::{CalendarProtocol, CalendarSlot},
        duration::DateDuration,
        DateTime, Duration,
    },
    iso::{IsoDate, IsoDateSlots},
    options::{ArithmeticOverflow, RelativeTo, TemporalRoundingMode, TemporalUnit},
    parser::parse_date_time,
    utils, TemporalError, TemporalResult,
};
use std::str::FromStr;

use super::{
    calendar::{CalendarDateLike, GetCalendarSlot},
    duration::TimeDuration,
};

/// The native Rust implementation of `Temporal.PlainDate`.
#[non_exhaustive]
#[derive(Debug, Default, Clone)]
pub struct Date<C: CalendarProtocol> {
    pub(crate) iso: IsoDate,
    calendar: CalendarSlot<C>,
}

// ==== Private API ====

impl<C: CalendarProtocol> Date<C> {
    /// Create a new `Date` with the date values and calendar slot.
    #[inline]
    #[must_use]
    pub(crate) fn new_unchecked(iso: IsoDate, calendar: CalendarSlot<C>) -> Self {
        Self { iso, calendar }
    }

    #[inline]
    /// Returns a new moved date and the days associated with that adjustment
    pub(crate) fn move_relative_date(
        &self,
        duration: &Duration,
        context: &mut C::Context,
    ) -> TemporalResult<(Self, f64)> {
        let new_date = self.add_date(duration, ArithmeticOverflow::Constrain, context)?;
        let days = f64::from(self.days_until(&new_date));
        Ok((new_date, days))
    }
    /// Returns the date after adding the given duration to date with a provided context.
    ///
    /// Temporal Equivalent: 3.5.13 `AddDate ( calendar, plainDate, duration [ , options [ , dateAdd ] ] )`
    #[inline]
    pub(crate) fn add_date(
        &self,
        duration: &Duration,
        overflow: ArithmeticOverflow,
        context: &mut C::Context,
    ) -> TemporalResult<Self> {
        // 2. If options is not present, set options to undefined.
        // 3. If duration.[[Years]] ≠ 0, or duration.[[Months]] ≠ 0, or duration.[[Weeks]] ≠ 0, then
        if duration.date().years != 0.0
            || duration.date().months != 0.0
            || duration.date().weeks != 0.0
        {
            // a. If dateAdd is not present, then
            // i. Set dateAdd to unused.
            // ii. If calendar is an Object, set dateAdd to ? GetMethod(calendar, "dateAdd").
            // b. Return ? CalendarDateAdd(calendar, plainDate, duration, options, dateAdd).
            return self.calendar().date_add(self, duration, overflow, context);
        }

        // 4. Let overflow be ? ToTemporalOverflow(options).
        // 5. Let norm be NormalizeTimeDuration(duration.[[Hours]], duration.[[Minutes]], duration.[[Seconds]], duration.[[Milliseconds]], duration.[[Microseconds]], duration.[[Nanoseconds]]).
        // 6. Let days be duration.[[Days]] + BalanceTimeDuration(norm, "day").[[Days]].
        let days = duration.days()
            + TimeDuration::from_normalized(duration.time().to_normalized(), TemporalUnit::Day)?.0;

        // 7. Let result be ? AddISODate(plainDate.[[ISOYear]], plainDate.[[ISOMonth]], plainDate.[[ISODay]], 0, 0, 0, days, overflow).
        let result = self
            .iso
            .add_date_duration(&DateDuration::new(0f64, 0f64, 0f64, days)?, overflow)?;

        Ok(Self::new_unchecked(result, self.calendar().clone()))
    }

    /// Returns a duration representing the difference between the dates one and two with a provided context.
    ///
    /// Temporal Equivalent: 3.5.6 `DifferenceDate ( calendar, one, two, options )`
    #[inline]
    pub(crate) fn internal_diff_date(
        &self,
        other: &Self,
        largest_unit: TemporalUnit,
        context: &mut C::Context,
    ) -> TemporalResult<Duration> {
        if self.iso.year == other.iso.year
            && self.iso.month == other.iso.month
            && self.iso.day == other.iso.day
        {
            return Ok(Duration::default());
        }

        if largest_unit == TemporalUnit::Day {
            let days = self.days_until(other);
            return Ok(Duration::from_date_duration(&DateDuration::new(
                0f64,
                0f64,
                0f64,
                f64::from(days),
            )?));
        }

        self.calendar()
            .date_until(self, other, largest_unit, context)
    }

    /// Equivalent: DifferenceTemporalPlainDate
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn diff_date(
        &self,
        op: bool,
        other: &Self,
        rounding_mode: Option<TemporalRoundingMode>,
        rounding_increment: Option<f64>,
        largest_unit: Option<TemporalUnit>,
        smallest_unit: Option<TemporalUnit>,
        context: &mut C::Context,
    ) -> TemporalResult<Duration> {
        // 1. If operation is SINCE, let sign be -1. Otherwise, let sign be 1.
        // 2. Set other to ? ToTemporalDate(other).

        // TODO(improvement): Implement `PartialEq` for `CalendarSlot<C>`
        // 3. If ? CalendarEquals(temporalDate.[[Calendar]], other.[[Calendar]]) is false, throw a RangeError exception.
        if self.calendar().identifier(context)? != other.calendar().identifier(context)? {
            return Err(TemporalError::range()
                .with_message("Calendars are for difference operation are not the same."));
        }

        // 4. Let resolvedOptions be ? SnapshotOwnProperties(? GetOptionsObject(options), null).
        // 5. Let settings be ? GetDifferenceSettings(operation, resolvedOptions, DATE, « », "day", "day").
        let increment = utils::to_rounding_increment(rounding_increment)?;
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
        let smallest_unit = smallest_unit.unwrap_or(TemporalUnit::Day);
        // Use the defaultlargestunit which is max smallestlargestdefault and smallestunit
        let largest_unit = largest_unit.unwrap_or(smallest_unit.max(TemporalUnit::Day));

        // 6. If temporalDate.[[ISOYear]] = other.[[ISOYear]], and temporalDate.[[ISOMonth]] = other.[[ISOMonth]],
        // and temporalDate.[[ISODay]] = other.[[ISODay]], then
        if self.iso == other.iso {
            // a. Return ! CreateTemporalDuration(0, 0, 0, 0, 0, 0, 0, 0, 0, 0).
            return Ok(Duration::default());
        }

        // 7. Let calendarRec be ? CreateCalendarMethodsRecord(temporalDate.[[Calendar]], « DATE-ADD, DATE-UNTIL »).
        // 8. Perform ! CreateDataPropertyOrThrow(resolvedOptions, "largestUnit", settings.[[LargestUnit]]).
        // 9. Let result be ? DifferenceDate(calendarRec, temporalDate, other, resolvedOptions).
        let result = self.internal_diff_date(other, largest_unit, context)?;

        // 10. If settings.[[SmallestUnit]] is "day" and settings.[[RoundingIncrement]] = 1,
        // let roundingGranularityIsNoop be true; else let roundingGranularityIsNoop be false.
        let is_noop = smallest_unit == TemporalUnit::Day && rounding_increment == Some(1.0);

        // 12. Return ! CreateTemporalDuration(sign × result.[[Years]], sign × result.[[Months]], sign × result.[[Weeks]], sign × result.[[Days]], 0, 0, 0, 0, 0, 0).
        if is_noop {
            return Duration::new(
                result.years() * sign,
                result.months() * sign,
                result.weeks() * sign,
                result.days() * sign,
                0.0,
                0.0,
                0.0,
                0.0,
                0.0,
                0.0,
            );
        }

        // 11. If roundingGranularityIsNoop is false, then
        // a. Let roundRecord be ? RoundDuration(result.[[Years]], result.[[Months]], result.[[Weeks]],
        // result.[[Days]], ZeroTimeDuration(), settings.[[RoundingIncrement]], settings.[[SmallestUnit]],
        // settings.[[RoundingMode]], temporalDate, calendarRec).
        // TODO: Look into simplifying round_internal's parameters.
        let round_record = result.round_internal(
            increment,
            smallest_unit,
            rounding_mode,
            &RelativeTo::<C, ()> {
                zdt: None,
                date: Some(self),
            },
            None,
            context,
        )?;
        // b. Let roundResult be roundRecord.[[NormalizedDuration]].
        let round_result = round_record.0 .0 .0;
        // c. Set result to ? BalanceDateDurationRelative(roundResult.[[Years]], roundResult.[[Months]], roundResult.[[Weeks]],
        // roundResult.[[Days]], settings.[[LargestUnit]], settings.[[SmallestUnit]], temporalDate, calendarRec).
        let result =
            round_result.balance_relative(largest_unit, smallest_unit, Some(self), context)?;

        Duration::new(
            result.years * sign,
            result.months * sign,
            result.weeks * sign,
            result.days * sign,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
        )
    }
}

// ==== Public API ====

impl<C: CalendarProtocol> Date<C> {
    /// Creates a new `Date` while checking for validity.
    pub fn new(
        year: i32,
        month: i32,
        day: i32,
        calendar: CalendarSlot<C>,
        overflow: ArithmeticOverflow,
    ) -> TemporalResult<Self> {
        let iso = IsoDate::new(year, month, day, overflow)?;
        Ok(Self::new_unchecked(iso, calendar))
    }

    #[must_use]
    /// Creates a `Date` from a `DateTime`.
    pub fn from_datetime(dt: &DateTime<C>) -> Self {
        Self {
            iso: dt.iso_date(),
            calendar: dt.calendar().clone(),
        }
    }

    #[inline]
    #[must_use]
    /// Returns this `Date`'s ISO year value.
    pub const fn iso_year(&self) -> i32 {
        self.iso.year
    }

    #[inline]
    #[must_use]
    /// Returns this `Date`'s ISO month value.
    pub const fn iso_month(&self) -> u8 {
        self.iso.month
    }

    #[inline]
    #[must_use]
    /// Returns this `Date`'s ISO day value.
    pub const fn iso_day(&self) -> u8 {
        self.iso.day
    }

    #[inline]
    #[must_use]
    /// Returns a reference to this `Date`'s calendar slot.
    pub fn calendar(&self) -> &CalendarSlot<C> {
        &self.calendar
    }

    /// 3.5.7 `IsValidISODate`
    ///
    /// Checks if the current date is a valid `ISODate`.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.iso.is_valid()
    }

    /// `DaysUntil`
    ///
    /// Calculates the epoch days between two `Date`s
    #[inline]
    #[must_use]
    pub fn days_until(&self, other: &Self) -> i32 {
        other.iso.to_epoch_days() - self.iso.to_epoch_days()
    }

    pub fn contextual_add(
        &self,
        duration: &Duration,
        overflow: Option<ArithmeticOverflow>,
        context: &mut C::Context,
    ) -> TemporalResult<Self> {
        self.add_date(
            duration,
            overflow.unwrap_or(ArithmeticOverflow::Constrain),
            context,
        )
    }

    pub fn contextual_subtract(
        &self,
        duration: &Duration,
        overflow: Option<ArithmeticOverflow>,
        context: &mut C::Context,
    ) -> TemporalResult<Self> {
        self.add_date(
            &duration.negated(),
            overflow.unwrap_or(ArithmeticOverflow::Constrain),
            context,
        )
    }

    pub fn contextual_until(
        &self,
        other: &Self,
        rounding_mode: Option<TemporalRoundingMode>,
        rounding_increment: Option<f64>,
        smallest_unit: Option<TemporalUnit>,
        largest_unit: Option<TemporalUnit>,
        context: &mut C::Context,
    ) -> TemporalResult<Duration> {
        self.diff_date(
            false,
            other,
            rounding_mode,
            rounding_increment,
            smallest_unit,
            largest_unit,
            context,
        )
    }

    pub fn contextual_since(
        &self,
        other: &Self,
        rounding_mode: Option<TemporalRoundingMode>,
        rounding_increment: Option<f64>,
        smallest_unit: Option<TemporalUnit>,
        largest_unit: Option<TemporalUnit>,
        context: &mut C::Context,
    ) -> TemporalResult<Duration> {
        self.diff_date(
            true,
            other,
            rounding_mode,
            rounding_increment,
            smallest_unit,
            largest_unit,
            context,
        )
    }
}

// ==== Calendar-derived Public API ====

impl Date<()> {
    /// Returns the calendar year value.
    pub fn year(&self) -> TemporalResult<i32> {
        self.calendar
            .year(&CalendarDateLike::Date(self.clone()), &mut ())
    }

    /// Returns the calendar month value.
    pub fn month(&self) -> TemporalResult<u8> {
        self.calendar
            .month(&CalendarDateLike::Date(self.clone()), &mut ())
    }

    /// Returns the calendar month code value.
    pub fn month_code(&self) -> TemporalResult<TinyAsciiStr<4>> {
        self.calendar
            .month_code(&CalendarDateLike::Date(self.clone()), &mut ())
    }

    /// Returns the calendar day value.
    pub fn day(&self) -> TemporalResult<u8> {
        self.calendar
            .day(&CalendarDateLike::Date(self.clone()), &mut ())
    }

    /// Returns the calendar day of week value.
    pub fn day_of_week(&self) -> TemporalResult<u16> {
        self.calendar
            .day_of_week(&CalendarDateLike::Date(self.clone()), &mut ())
    }

    /// Returns the calendar day of year value.
    pub fn day_of_year(&self) -> TemporalResult<u16> {
        self.calendar
            .day_of_year(&CalendarDateLike::Date(self.clone()), &mut ())
    }

    /// Returns the calendar week of year value.
    pub fn week_of_year(&self) -> TemporalResult<u16> {
        self.calendar
            .week_of_year(&CalendarDateLike::Date(self.clone()), &mut ())
    }

    /// Returns the calendar year of week value.
    pub fn year_of_week(&self) -> TemporalResult<i32> {
        self.calendar
            .year_of_week(&CalendarDateLike::Date(self.clone()), &mut ())
    }

    /// Returns the calendar days in week value.
    pub fn days_in_week(&self) -> TemporalResult<u16> {
        self.calendar
            .days_in_week(&CalendarDateLike::Date(self.clone()), &mut ())
    }

    /// Returns the calendar days in month value.
    pub fn days_in_month(&self) -> TemporalResult<u16> {
        self.calendar
            .days_in_month(&CalendarDateLike::Date(self.clone()), &mut ())
    }

    /// Returns the calendar days in year value.
    pub fn days_in_year(&self) -> TemporalResult<u16> {
        self.calendar
            .days_in_year(&CalendarDateLike::Date(self.clone()), &mut ())
    }

    /// Returns the calendar months in year value.
    pub fn months_in_year(&self) -> TemporalResult<u16> {
        self.calendar
            .months_in_year(&CalendarDateLike::Date(self.clone()), &mut ())
    }

    /// Returns returns whether the date in a leap year for the given calendar.
    pub fn in_leap_year(&self) -> TemporalResult<bool> {
        self.calendar
            .in_leap_year(&CalendarDateLike::Date(self.clone()), &mut ())
    }

    /// Returns the result of adding a `Duration` to the current `Date`.
    pub fn add(
        &self,
        duration: &Duration,
        overflow: Option<ArithmeticOverflow>,
    ) -> TemporalResult<Self> {
        self.contextual_add(duration, overflow, &mut ())
    }

    /// Returns the result of subtracting a `Duration` from the current `Date`.
    pub fn subtract(
        &self,
        duration: &Duration,
        overflow: Option<ArithmeticOverflow>,
    ) -> TemporalResult<Self> {
        self.contextual_subtract(duration, overflow, &mut ())
    }

    /// Returns a `Duration` representing the time until a provided `Date`.
    pub fn until(
        &self,
        other: &Self,
        rounding_mode: Option<TemporalRoundingMode>,
        rounding_increment: Option<f64>,
        smallest_unit: Option<TemporalUnit>,
        largest_unit: Option<TemporalUnit>,
    ) -> TemporalResult<Duration> {
        self.contextual_until(
            other,
            rounding_mode,
            rounding_increment,
            smallest_unit,
            largest_unit,
            &mut (),
        )
    }

    /// Returns a `Duration` representing the time since a provided `Date`.
    pub fn since(
        &self,
        other: &Self,
        rounding_mode: Option<TemporalRoundingMode>,
        rounding_increment: Option<f64>,
        smallest_unit: Option<TemporalUnit>,
        largest_unit: Option<TemporalUnit>,
    ) -> TemporalResult<Duration> {
        self.contextual_since(
            other,
            rounding_mode,
            rounding_increment,
            smallest_unit,
            largest_unit,
            &mut (),
        )
    }
}

// NOTE(nekevss): The clone below should ideally not change the memory address, but that may
// not be true across all cases. I.e., it should be fine as long as the clone is simply a
// reference count increment. Need to test.
impl<C: CalendarProtocol> Date<C> {
    /// Returns the calendar year value with provided context.
    pub fn contextual_year(this: &C::Date, context: &mut C::Context) -> TemporalResult<i32> {
        this.get_calendar()
            .year(&CalendarDateLike::CustomDate(this.clone()), context)
    }

    /// Returns the calendar month value with provided context.
    pub fn contextual_month(this: &C::Date, context: &mut C::Context) -> TemporalResult<u8> {
        this.get_calendar()
            .month(&CalendarDateLike::CustomDate(this.clone()), context)
    }

    /// Returns the calendar month code value with provided context.
    pub fn contextual_month_code(
        this: &C::Date,
        context: &mut C::Context,
    ) -> TemporalResult<TinyAsciiStr<4>> {
        this.get_calendar()
            .month_code(&CalendarDateLike::CustomDate(this.clone()), context)
    }

    /// Returns the calendar day value with provided context.
    pub fn contextual_day(this: &C::Date, context: &mut C::Context) -> TemporalResult<u8> {
        this.get_calendar()
            .day(&CalendarDateLike::CustomDate(this.clone()), context)
    }

    /// Returns the calendar day of week value with provided context.
    pub fn contextual_day_of_week(this: &C::Date, context: &mut C::Context) -> TemporalResult<u16> {
        this.get_calendar()
            .day_of_week(&CalendarDateLike::CustomDate(this.clone()), context)
    }

    /// Returns the calendar day of year value with provided context.
    pub fn contextual_day_of_year(this: &C::Date, context: &mut C::Context) -> TemporalResult<u16> {
        this.get_calendar()
            .day_of_year(&CalendarDateLike::CustomDate(this.clone()), context)
    }

    /// Returns the calendar week of year value with provided context.
    pub fn contextual_week_of_year(
        this: &C::Date,
        context: &mut C::Context,
    ) -> TemporalResult<u16> {
        this.get_calendar()
            .week_of_year(&CalendarDateLike::CustomDate(this.clone()), context)
    }

    /// Returns the calendar year of week value with provided context.
    pub fn contextual_year_of_week(
        this: &C::Date,
        context: &mut C::Context,
    ) -> TemporalResult<i32> {
        this.get_calendar()
            .year_of_week(&CalendarDateLike::CustomDate(this.clone()), context)
    }

    /// Returns the calendar days in week value with provided context.
    pub fn contextual_days_in_week(
        this: &C::Date,
        context: &mut C::Context,
    ) -> TemporalResult<u16> {
        this.get_calendar()
            .days_in_week(&CalendarDateLike::CustomDate(this.clone()), context)
    }

    /// Returns the calendar days in month value with provided context.
    pub fn contextual_days_in_month(
        this: &C::Date,
        context: &mut C::Context,
    ) -> TemporalResult<u16> {
        this.get_calendar()
            .days_in_month(&CalendarDateLike::CustomDate(this.clone()), context)
    }

    /// Returns the calendar days in year value with provided context.
    pub fn contextual_days_in_year(
        this: &C::Date,
        context: &mut C::Context,
    ) -> TemporalResult<u16> {
        this.get_calendar()
            .days_in_year(&CalendarDateLike::CustomDate(this.clone()), context)
    }

    /// Returns the calendar months in year value with provided context.
    pub fn contextual_months_in_year(
        this: &C::Date,
        context: &mut C::Context,
    ) -> TemporalResult<u16> {
        this.get_calendar()
            .months_in_year(&CalendarDateLike::CustomDate(this.clone()), context)
    }

    /// Returns whether the date is in a leap year for the given calendar with provided context.
    pub fn contextual_in_leap_year(
        this: &C::Date,
        context: &mut C::Context,
    ) -> TemporalResult<bool> {
        this.get_calendar()
            .in_leap_year(&CalendarDateLike::CustomDate(this.clone()), context)
    }
}

impl<C: CalendarProtocol> GetCalendarSlot<C> for Date<C> {
    fn get_calendar(&self) -> CalendarSlot<C> {
        self.calendar.clone()
    }
}

impl<C: CalendarProtocol> IsoDateSlots for Date<C> {
    /// Returns the structs `IsoDate`
    fn iso_date(&self) -> IsoDate {
        self.iso
    }
}

// ==== Context based API ====

impl<C: CalendarProtocol> Date<C> {}

// ==== Trait impls ====

impl<C: CalendarProtocol> FromStr for Date<C> {
    type Err = TemporalError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parse_record = parse_date_time(s)?;

        let calendar = parse_record.calendar.unwrap_or("iso8601".to_owned());

        let date = IsoDate::new(
            parse_record.date.year,
            parse_record.date.month,
            parse_record.date.day,
            ArithmeticOverflow::Reject,
        )?;

        Ok(Self::new_unchecked(
            date,
            CalendarSlot::from_str(&calendar)?,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_date_add() {
        let base = Date::<()>::from_str("1976-11-18").unwrap();

        // Test 1
        let result = base
            .add(&Duration::from_str("P43Y").unwrap(), None)
            .unwrap();
        assert_eq!(
            result.iso,
            IsoDate {
                year: 2019,
                month: 11,
                day: 18,
            }
        );

        // Test 2
        let result = base.add(&Duration::from_str("P3M").unwrap(), None).unwrap();
        assert_eq!(
            result.iso,
            IsoDate {
                year: 1977,
                month: 2,
                day: 18,
            }
        );

        // Test 3
        let result = base
            .add(&Duration::from_str("P20D").unwrap(), None)
            .unwrap();
        assert_eq!(
            result.iso,
            IsoDate {
                year: 1976,
                month: 12,
                day: 8,
            }
        )
    }

    #[test]
    fn simple_date_subtract() {
        let base = Date::<()>::from_str("2019-11-18").unwrap();

        // Test 1
        let result = base
            .subtract(&Duration::from_str("P43Y").unwrap(), None)
            .unwrap();
        assert_eq!(
            result.iso,
            IsoDate {
                year: 1976,
                month: 11,
                day: 18,
            }
        );

        // Test 2
        let result = base
            .subtract(&Duration::from_str("P11M").unwrap(), None)
            .unwrap();
        assert_eq!(
            result.iso,
            IsoDate {
                year: 2018,
                month: 12,
                day: 18,
            }
        );

        // Test 3
        let result = base
            .subtract(&Duration::from_str("P20D").unwrap(), None)
            .unwrap();
        assert_eq!(
            result.iso,
            IsoDate {
                year: 2019,
                month: 10,
                day: 29,
            }
        )
    }

    #[test]
    fn simple_date_until() {
        let earlier = Date::<()>::from_str("1969-07-24").unwrap();
        let later = Date::<()>::from_str("1969-10-05").unwrap();
        let result = earlier.until(&later, None, None, None, None).unwrap();
        assert_eq!(result.days(), 73.0,);

        let later = Date::<()>::from_str("1996-03-03").unwrap();
        let result = earlier.until(&later, None, None, None, None).unwrap();
        assert_eq!(result.days(), 9719.0,);
    }

    #[test]
    fn simple_date_since() {
        let earlier = Date::<()>::from_str("1969-07-24").unwrap();
        let later = Date::<()>::from_str("1969-10-05").unwrap();
        let result = later.since(&earlier, None, None, None, None).unwrap();
        assert_eq!(result.days(), 73.0,);

        let later = Date::<()>::from_str("1996-03-03").unwrap();
        let result = later.since(&earlier, None, None, None, None).unwrap();
        assert_eq!(result.days(), 9719.0,);
    }
}
