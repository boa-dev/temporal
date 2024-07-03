//! This module implements `MonthDay` and any directly related algorithms.

use std::str::FromStr;

use crate::{
    components::calendar::Calendar,
    iso::{IsoDate, IsoDateSlots},
    options::ArithmeticOverflow,
    TemporalError, TemporalResult, TemporalUnwrap,
};

use super::calendar::GetTemporalCalendar;

/// The native Rust implementation of `Temporal.PlainMonthDay`
#[non_exhaustive]
#[derive(Debug, Default, Clone)]
pub struct MonthDay {
    iso: IsoDate,
    calendar: Calendar,
}

impl MonthDay {
    /// Creates a new unchecked `MonthDay`
    #[inline]
    #[must_use]
    pub(crate) fn new_unchecked(iso: IsoDate, calendar: Calendar) -> Self {
        Self { iso, calendar }
    }

    /// Creates a new valid `MonthDay`.
    #[inline]
    pub fn new(
        month: i32,
        day: i32,
        calendar: Calendar,
        overflow: ArithmeticOverflow,
    ) -> TemporalResult<Self> {
        let iso = IsoDate::new(1972, month, day, overflow)?;
        Ok(Self::new_unchecked(iso, calendar))
    }

    /// Returns the `month` value of `MonthDay`.
    #[inline]
    #[must_use]
    pub fn month(&self) -> u8 {
        self.iso.month
    }

    /// Returns the `day` value of `MonthDay`.
    #[inline]
    #[must_use]
    pub fn day(&self) -> u8 {
        self.iso.day
    }

    /// Returns a reference to `MonthDay`'s `CalendarSlot`
    #[inline]
    #[must_use]
    pub fn calendar(&self) -> &Calendar {
        &self.calendar
    }
}

impl GetTemporalCalendar for MonthDay {
    fn get_calendar(&self) -> Calendar {
        self.calendar.clone()
    }
}

impl IsoDateSlots for MonthDay {
    #[inline]
    /// Returns this structs `IsoDate`.
    fn iso_date(&self) -> IsoDate {
        self.iso
    }
}

impl FromStr for MonthDay {
    type Err = TemporalError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let record = crate::parsers::parse_month_day(s)?;

        let calendar = record.calendar.unwrap_or("iso8601");

        let date = record.date;

        let date = date.temporal_unwrap()?;

        Self::new(
            date.month.into(),
            date.day.into(),
            Calendar::from_str(calendar)?,
            ArithmeticOverflow::Reject,
        )
    }
}
