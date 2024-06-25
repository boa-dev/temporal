//! This module implements `YearMonth` and any directly related algorithms.

use std::str::FromStr;

use tinystr::TinyAsciiStr;

use crate::{
    components::calendar::Calendar,
    iso::{IsoDate, IsoDateSlots},
    options::ArithmeticOverflow,
    utils, TemporalError, TemporalFields, TemporalResult, TemporalUnwrap,
};

use super::calendar::GetTemporalCalendar;

// Subset of `TemporalFields` representing just the  `YearMonthFields`
pub struct YearMonthFields(i32, u8);

/// The native Rust implementation of `Temporal.YearMonth`.
#[non_exhaustive]
#[derive(Debug, Default, Clone)]
pub struct YearMonth {
    iso: IsoDate,
    calendar: Calendar,
}

impl YearMonth {
    /// Creates an unvalidated `YearMonth`.
    #[inline]
    #[must_use]
    pub(crate) fn new_unchecked(iso: IsoDate, calendar: Calendar) -> Self {
        Self { iso, calendar }
    }

    /// Creates a new valid `YearMonth`.
    #[inline]
    pub fn new(
        year: i32,
        month: i32,
        reference_day: Option<i32>,
        calendar: Calendar,
        overflow: ArithmeticOverflow,
    ) -> TemporalResult<Self> {
        let day = reference_day.unwrap_or(1);
        let iso = IsoDate::new(year, month, day, overflow)?;
        Ok(Self::new_unchecked(iso, calendar))
    }

    /// Returns the `year` value for this `YearMonth`.
    #[inline]
    #[must_use]
    pub fn year(&self) -> i32 {
        self.iso.year
    }

    /// Returns the `month` value for this `YearMonth`.
    #[inline]
    #[must_use]
    pub fn month(&self) -> u8 {
        self.iso.month
    }

    /// Returns the Calendar value.
    #[inline]
    #[must_use]
    pub fn in_leap_year(&self) -> bool {
        utils::mathematical_in_leap_year(utils::epoch_time_for_year(self.iso.year)) == 1
    }
}

// Contextual Methods
impl<C: CalendarProtocol> YearMonth<C> {
    pub fn contextual_get_days_in_year(
        this: &C::YearMonth,
        context: &mut C::Context,
    ) -> TemporalResult<u16> {
        this.get_calendar()
            .days_in_year(&CalendarDateLike::YearMonth(this.clone()), context)
    }

    pub fn contextual_get_days_in_month(
        this: &C::YearMonth,
        context: &mut C::Context,
    ) -> TemporalResult<u16> {
        this.get_calendar()
            .days_in_month(&CalendarDateLike::YearMonth(this.clone()), context)
    }

    pub fn contextual_month_code(
        this: &C::YearMonth,
        context: &mut C::Context,
    ) -> TemporalResult<TinyAsciiStr<4>> {
        this.get_calendar()
            .month_code(&CalendarDateLike::YearMonth(this.clone()), context)
    }

    /// Returns the Calendar value.
    #[inline]
    #[must_use]
    pub fn calendar(&self) -> &Calendar {
        &self.calendar
    }

    pub fn contextual_get_months_in_year(
        this: &C::YearMonth,
        context: &mut C::Context,
    ) -> TemporalResult<u16> {
        this.get_calendar()
            .months_in_year(&CalendarDateLike::YearMonth(this.clone()), context)
    }

    pub fn add_duration(
        this: &C::YearMonth,
        duration: Duration,
        overflow: ArithmeticOverflow,
        context: &mut C::Context,
    ) -> TemporalResult<YearMonth<C>> {
        Self::contextual_add_or_subtract_duration(true, this, duration, context, overflow)
    }

    pub fn subtract_duration(
        this: &C::YearMonth,
        duration: Duration,
        overflow: ArithmeticOverflow,
        context: &mut C::Context,
    ) -> TemporalResult<YearMonth<C>> {
        Self::contextual_add_or_subtract_duration(false, this, duration, context, overflow)
    }

    pub(crate) fn contextual_add_or_subtract_duration(
        addition: bool,
        this: &C::YearMonth,
        mut duration: Duration,
        context: &mut C::Context,
        overflow: ArithmeticOverflow,
    ) -> TemporalResult<YearMonth<C>> {
        if !addition {
            duration = duration.negated()
        }

        let mut fields = YearMonthFields(this.iso_date().year, this.iso_date().month).into();

        let mut intermediate_date =
            this.get_calendar()
                .date_from_fields(&mut fields, overflow, context)?;

        intermediate_date = intermediate_date.add_date(&duration, Some(overflow), context)?;

        let mut result_fields =
            YearMonthFields(intermediate_date.iso_year(), intermediate_date.iso_month()).into();

        this.get_calendar()
            .year_month_from_fields(&mut result_fields, overflow, context)
    }
}

impl GetTemporalCalendar for YearMonth {
    /// Returns a reference to `YearMonth`'s `CalendarSlot`
    fn get_calendar(&self) -> Calendar {
        self.calendar.clone()
    }
}

impl IsoDateSlots for YearMonth {
    #[inline]
    /// Returns this `YearMonth`'s `IsoDate`
    fn iso_date(&self) -> IsoDate {
        self.iso
    }
}

impl FromStr for YearMonth {
    type Err = TemporalError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let record = crate::parsers::parse_year_month(s)?;

        let calendar = record.calendar.unwrap_or("iso8601");

        let date = record.date.temporal_unwrap()?;

        Self::new(
            date.year,
            date.month.into(),
            None,
            Calendar::from_str(calendar)?,
            ArithmeticOverflow::Reject,
        )
    }
}

// Conversion to `TemporalFields`
impl From<YearMonthFields> for TemporalFields {
    fn from(value: YearMonthFields) -> Self {
        TemporalFields {
            bit_map: FieldMap::YEAR | FieldMap::MONTH,
            year: Some(value.0),
            month: Some(value.1.into()),
            ..Default::default()
        }
    }
}
