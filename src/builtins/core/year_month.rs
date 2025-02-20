//! This module implements `YearMonth` and any directly related algorithms.

use alloc::string::String;
use core::{cmp::Ordering, str::FromStr};

use tinystr::TinyAsciiStr;

use crate::{
    iso::IsoDate,
    options::{ArithmeticOverflow, DisplayCalendar},
    parsers::{FormattableCalendar, FormattableDate, FormattableYearMonth},
    utils::pad_iso_year,
    Calendar, TemporalError, TemporalResult, TemporalUnwrap,
};

use super::{Duration, PartialDate};

/// The native Rust implementation of `Temporal.YearMonth`.
#[non_exhaustive]
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct PlainYearMonth {
    pub(crate) iso: IsoDate,
    calendar: Calendar,
}

impl core::fmt::Display for PlainYearMonth {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(&self.to_ixdtf_string(DisplayCalendar::Auto))
    }
}

impl PlainYearMonth {
    /// Creates an unvalidated `YearMonth`.
    #[inline]
    #[must_use]
    pub(crate) fn new_unchecked(iso: IsoDate, calendar: Calendar) -> Self {
        Self { iso, calendar }
    }

    /// Creates a new valid `YearMonth`.
    #[inline]
    pub fn new_with_overflow(
        year: i32,
        month: u8,
        reference_day: Option<u8>,
        calendar: Calendar,
        overflow: ArithmeticOverflow,
    ) -> TemporalResult<Self> {
        let day = reference_day.unwrap_or(1);
        let iso = IsoDate::new_with_overflow(year, month, day, overflow)?;
        Ok(Self::new_unchecked(iso, calendar))
    }

    /// Returns the iso year value for this `YearMonth`.
    #[inline]
    #[must_use]
    pub fn iso_year(&self) -> i32 {
        self.iso.year
    }

    /// Returns the padded ISO year string
    #[inline]
    #[must_use]
    pub fn padded_iso_year_string(&self) -> String {
        pad_iso_year(self.iso.year)
    }

    /// Returns the iso month value for this `YearMonth`.
    #[inline]
    #[must_use]
    pub fn iso_month(&self) -> u8 {
        self.iso.month
    }

    /// Returns the calendar era of the current `PlainYearMonth`
    pub fn era(&self) -> TemporalResult<Option<TinyAsciiStr<16>>> {
        self.calendar().era(&self.iso)
    }

    /// Returns the calendar era year of the current `PlainYearMonth`
    pub fn era_year(&self) -> TemporalResult<Option<i32>> {
        self.calendar().era_year(&self.iso)
    }

    /// Returns the calendar year of the current `PlainYearMonth`
    pub fn year(&self) -> TemporalResult<i32> {
        self.calendar().year(&self.iso)
    }

    /// Returns the calendar month of the current `PlainYearMonth`
    pub fn month(&self) -> TemporalResult<u8> {
        self.calendar().month(&self.iso)
    }

    /// Returns the calendar month code of the current `PlainYearMonth`
    pub fn month_code(&self) -> TemporalResult<TinyAsciiStr<4>> {
        self.calendar().month_code(&self.iso)
    }

    /// Returns the days in the calendar year of the current `PlainYearMonth`.
    pub fn days_in_year(&self) -> TemporalResult<u16> {
        self.calendar().days_in_year(&self.iso)
    }

    /// Returns the days in the calendar month of the current `PlainYearMonth`.
    pub fn days_in_month(&self) -> TemporalResult<u16> {
        self.calendar().days_in_month(&self.iso)
    }

    /// Returns the months in the calendar year of the current `PlainYearMonth`.
    pub fn months_in_year(&self) -> TemporalResult<u16> {
        self.calendar().months_in_year(&self.iso)
    }

    #[inline]
    #[must_use]
    /// Returns a boolean representing whether the current `PlainYearMonth` is in a leap year.
    pub fn in_leap_year(&self) -> bool {
        self.calendar()
            .in_leap_year(&self.iso)
            .is_ok_and(|is_leap_year| is_leap_year)
    }
}

impl PlainYearMonth {
    /// Returns the Calendar value.
    #[inline]
    #[must_use]
    pub fn calendar(&self) -> &Calendar {
        &self.calendar
    }

    /// Returns the string identifier for the current calendar used.
    #[inline]
    #[must_use]
    pub fn calendar_id(&self) -> &'static str {
        self.calendar.identifier()
    }

    /// Returns the calendar day value.
    pub fn day(&self) -> TemporalResult<u8> {
        self.calendar.day(&self.iso)
    }

    pub fn with(
        &self,
        partial: PartialDate,
        overflow: Option<ArithmeticOverflow>,
    ) -> TemporalResult<Self> {
        // 1. Let yearMonth be the this value.
        // 2. Perform ? RequireInternalSlot(yearMonth, [[InitializedTemporalYearMonth]]).
        // 3. If ? IsPartialTemporalObject(temporalYearMonthLike) is false, throw a TypeError exception.
        if partial.is_empty() {
            return Err(TemporalError::r#type().with_message("A PartialDate must have a field."));
        };
        // 4. Let calendar be yearMonth.[[Calendar]].
        // 5. Let fields be ISODateToFields(calendar, yearMonth.[[ISODate]], year-month).
        // 6. Let partialYearMonth be ? PrepareCalendarFields(calendar, temporalYearMonthLike, « year, month, month-code », « », partial).
        // 7. Set fields to CalendarMergeFields(calendar, fields, partialYearMonth).
        // 8. Let resolvedOptions be ? GetOptionsObject(options).
        // 9. Let overflow be ? GetTemporalOverflowOption(resolvedOptions).
        // 10. Let isoDate be ? CalendarYearMonthFromFields(calendar, fields, overflow).
        // 11. Return ! CreateTemporalYearMonth(isoDate, calendar).
        self.calendar.year_month_from_partial(
            &partial.with_fallback_year_month(self)?,
            overflow.unwrap_or(ArithmeticOverflow::Constrain),
        )
    }

    /// Compares one `PlainYearMonth` to another `PlainYearMonth` using their
    /// `IsoDate` representation.
    ///
    /// # Note on Ordering.
    ///
    /// `temporal_rs` does not implement `PartialOrd`/`Ord` as `PlainYearMonth` does
    /// not fulfill all the conditions required to implement the traits. However,
    /// it is possible to compare `PlainDate`'s as their `IsoDate` representation.
    #[inline]
    #[must_use]
    pub fn compare_iso(&self, other: &Self) -> Ordering {
        self.iso.cmp(&other.iso)
    }

    pub fn add_duration(
        &self,
        duration: &Duration,
        overflow: ArithmeticOverflow,
    ) -> TemporalResult<Self> {
        self.add_or_subtract_duration(duration, overflow)
    }

    pub fn subtract_duration(
        &self,
        duration: &Duration,
        overflow: ArithmeticOverflow,
    ) -> TemporalResult<Self> {
        self.add_or_subtract_duration(&duration.negated(), overflow)
    }

    pub(crate) fn add_or_subtract_duration(
        &self,
        duration: &Duration,
        overflow: ArithmeticOverflow,
    ) -> TemporalResult<Self> {
        let partial = PartialDate::try_from_year_month(self)?;

        let mut intermediate_date = self.calendar().date_from_partial(&partial, overflow)?;

        intermediate_date = intermediate_date.add_date(duration, Some(overflow))?;

        let result_fields = PartialDate::default().with_fallback_date(&intermediate_date)?;

        self.calendar()
            .year_month_from_partial(&result_fields, overflow)
    }

    pub fn to_ixdtf_string(&self, display_calendar: DisplayCalendar) -> String {
        let ixdtf = FormattableYearMonth {
            date: FormattableDate(self.iso_year(), self.iso_month(), self.iso.day),
            calendar: FormattableCalendar {
                show: display_calendar,
                calendar: self.calendar().identifier(),
            },
        };
        ixdtf.to_string()
    }
}

impl FromStr for PlainYearMonth {
    type Err = TemporalError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let record = crate::parsers::parse_year_month(s)?;

        let calendar = record
            .calendar
            .map(Calendar::from_utf8)
            .transpose()?
            .unwrap_or_default();

        let date = record.date.temporal_unwrap()?;

        Self::new_with_overflow(
            date.year,
            date.month,
            None,
            calendar,
            ArithmeticOverflow::Reject,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tinystr::tinystr;

    #[test]
    fn test_plain_year_month_with() {
        let base = PlainYearMonth::new_with_overflow(
            2025,
            3,
            None,
            Calendar::default(),
            ArithmeticOverflow::Reject,
        )
        .unwrap();

        // Year
        let partial = PartialDate {
            year: Some(2001),
            ..Default::default()
        };

        let with_year = base.with(partial, None).unwrap();
        assert_eq!(with_year.iso_year(), 2001); // year is changed
        assert_eq!(with_year.iso_month(), 3); // month is not changed
        assert_eq!(
            with_year.month_code().unwrap(),
            TinyAsciiStr::<4>::from_str("M03").unwrap()
        ); // assert month code has been initialized correctly

        // Month
        let partial = PartialDate {
            month: Some(2),
            ..Default::default()
        };
        let with_month = base.with(partial, None).unwrap();
        assert_eq!(with_month.iso_year(), 2025); // year is not changed
        assert_eq!(with_month.iso_month(), 2); // month is changed
        assert_eq!(
            with_month.month_code().unwrap(),
            TinyAsciiStr::<4>::from_str("M02").unwrap()
        ); // assert month code has changed as well as month

        // Month Code
        let partial = PartialDate {
            month_code: Some(tinystr!(4, "M05")), // change month to May (5)
            ..Default::default()
        };
        let with_month_code = base.with(partial, None).unwrap();
        assert_eq!(with_month_code.iso_year(), 2025); // year is not changed
        assert_eq!(
            with_month_code.month_code().unwrap(),
            TinyAsciiStr::<4>::from_str("M05").unwrap()
        ); // assert month code has changed
        assert_eq!(with_month_code.iso_month(), 5); // month is changed as well

        // Day
        let partial = PartialDate {
            day: Some(15),
            ..Default::default()
        };
        let with_day = base.with(partial, None).unwrap();
        assert_eq!(with_day.iso_year(), 2025); // year is not changed
        assert_eq!(with_day.iso_month(), 3); // month is not changed
        assert_eq!(with_day.iso.day, 15); // day is changed

        // All
        let partial = PartialDate {
            year: Some(2001),
            month: Some(2),
            day: Some(15),
            ..Default::default()
        };
        let with_all = base.with(partial, None).unwrap();
        assert_eq!(with_all.iso_year(), 2001); // year is changed
        assert_eq!(with_all.iso_month(), 2); // month is changed
        assert_eq!(with_all.iso.day, 15); // day is changed

        /*
        // ArithmeticOverflow for PlainYearMonth, the test currently fails
        let partial = PartialDate {
            month: Some(13), // invalid month
            ..Default::default()
        };
        // Constrained behavior
        let with_overflow_constrain = base
            .with(partial.clone(), Some(ArithmeticOverflow::Constrain))
            .unwrap();
        assert_eq!(with_overflow_constrain.iso_year(), 2025); // year is not changed
        assert_eq!(with_overflow_constrain.iso_month(), 12); // month is constrained to December

        // Reject behavior
        let with_overflow_reject = base
            .with(partial.clone(), Some(ArithmeticOverflow::Reject))
            .unwrap();
        */
    }
}
