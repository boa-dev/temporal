//! This module implements `ZonedDateTime` and any directly related algorithms.

use num_bigint::BigInt;
use tinystr::TinyStr4;

use crate::{
    components::{calendar::Calendar, tz::TimeZone, Instant},
    TemporalResult,
};

use super::calendar::CalendarDateLike;

/// The native Rust implementation of `Temporal.ZonedDateTime`.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ZonedDateTime {
    instant: Instant,
    calendar: Calendar,
    tz: TimeZone,
}

impl Ord for ZonedDateTime {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.instant.cmp(&other.instant)
    }
}

impl PartialOrd for ZonedDateTime {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

// ==== Private API ====

impl ZonedDateTime {
    /// Creates a `ZonedDateTime` without validating the input.
    #[inline]
    #[must_use]
    pub(crate) fn new_unchecked(instant: Instant, calendar: Calendar, tz: TimeZone) -> Self {
        Self {
            instant,
            calendar,
            tz,
        }
    }
}

// ==== Public API ====

impl ZonedDateTime {
    /// Creates a new valid `ZonedDateTime`.
    #[inline]
    pub fn new(nanos: BigInt, calendar: Calendar, tz: TimeZone) -> TemporalResult<Self> {
        let instant = Instant::new(nanos)?;
        Ok(Self::new_unchecked(instant, calendar, tz))
    }

    /// Returns `ZonedDateTime`'s Calendar.
    #[inline]
    #[must_use]
    pub fn calendar(&self) -> &Calendar {
        &self.calendar
    }

    /// Returns `ZonedDateTime`'s `TimeZone` slot.
    #[inline]
    #[must_use]
    pub fn tz(&self) -> &TimeZone {
        &self.tz
    }

    /// Returns the `epochSeconds` value of this `ZonedDateTime`.
    #[must_use]
    pub fn epoch_seconds(&self) -> f64 {
        self.instant.epoch_seconds()
    }

    /// Returns the `epochMilliseconds` value of this `ZonedDateTime`.
    #[must_use]
    pub fn epoch_milliseconds(&self) -> f64 {
        self.instant.epoch_milliseconds()
    }

    /// Returns the `epochMicroseconds` value of this `ZonedDateTime`.
    #[must_use]
    pub fn epoch_microseconds(&self) -> f64 {
        self.instant.epoch_microseconds()
    }

    /// Returns the `epochNanoseconds` value of this `ZonedDateTime`.
    #[must_use]
    pub fn epoch_nanoseconds(&self) -> f64 {
        self.instant.epoch_nanoseconds()
    }

    /// Returns the `year` value for this `ZonedDateTime`.
    #[inline]
    pub fn year(&self) -> TemporalResult<i32> {
        let dt = self.tz.get_datetime_for(&self.instant, &self.calendar)?;
        self.calendar.year(&CalendarDateLike::DateTime(dt))
    }

    /// Returns the `month` value for this `ZonedDateTime`.
    pub fn month(&self) -> TemporalResult<u8> {
        let dt = self.tz.get_datetime_for(&self.instant, &self.calendar)?;
        self.calendar.month(&CalendarDateLike::DateTime(dt))
    }

    /// Returns the `monthCode` value for this `ZonedDateTime`.
    pub fn month_code(&self) -> TemporalResult<TinyStr4> {
        let dt = self.tz.get_datetime_for(&self.instant, &self.calendar)?;
        self.calendar.month_code(&CalendarDateLike::DateTime(dt))
    }

    /// Returns the `day` value for this `ZonedDateTime`.
    pub fn day(&self) -> TemporalResult<u8> {
        let dt = self.tz.get_datetime_for(&self.instant, &self.calendar)?;
        self.calendar.day(&CalendarDateLike::DateTime(dt))
    }

    /// Returns the `hour` value for this `ZonedDateTime`.
    pub fn hour(&self) -> TemporalResult<u8> {
        let dt = self.tz.get_datetime_for(&self.instant, &self.calendar)?;
        Ok(dt.hour())
    }

    /// Returns the `minute` value for this `ZonedDateTime`.
    pub fn minute(&self) -> TemporalResult<u8> {
        let dt = self.tz.get_datetime_for(&self.instant, &self.calendar)?;
        Ok(dt.minute())
    }

    /// Returns the `second` value for this `ZonedDateTime`.
    pub fn second(&self) -> TemporalResult<u8> {
        let dt = self.tz.get_datetime_for(&self.instant, &self.calendar)?;
        Ok(dt.second())
    }

    /// Returns the `millisecond` value for this `ZonedDateTime`.
    pub fn millisecond(&self) -> TemporalResult<u16> {
        let dt = self.tz.get_datetime_for(&self.instant, &self.calendar)?;
        Ok(dt.millisecond())
    }

    /// Returns the `microsecond` value for this `ZonedDateTime`.
    pub fn microsecond(&self) -> TemporalResult<u16> {
        let dt = self.tz.get_datetime_for(&self.instant, &self.calendar)?;
        Ok(dt.millisecond())
    }

    /// Returns the `nanosecond` value for this `ZonedDateTime`.
    pub fn nanosecond(&self) -> TemporalResult<u16> {
        let dt = self.tz.get_datetime_for(&self.instant, &self.calendar)?;
        Ok(dt.nanosecond())
    }
}

#[cfg(test)]
mod tests {

    use std::str::FromStr;

    use crate::components::{calendar::Calendar, tz::TimeZone};
    use num_bigint::BigInt;

    use super::ZonedDateTime;

    #[test]
    fn basic_zdt_test() {
        let nov_30_2023_utc = BigInt::from(1_701_308_952_000_000_000i64);

        let zdt = ZonedDateTime::new(
            nov_30_2023_utc.clone(),
            Calendar::from_str("iso8601").unwrap(),
            TimeZone {
                iana: None,
                offset: Some(0),
            },
        )
        .unwrap();

        assert_eq!(zdt.year().unwrap(), 2023);
        assert_eq!(zdt.month().unwrap(), 11);
        assert_eq!(zdt.day().unwrap(), 30);
        assert_eq!(zdt.hour().unwrap(), 1);
        assert_eq!(zdt.minute().unwrap(), 49);
        assert_eq!(zdt.second().unwrap(), 12);

        let zdt_minus_five = ZonedDateTime::new(
            nov_30_2023_utc,
            Calendar::from_str("iso8601").unwrap(),
            TimeZone {
                iana: None,
                offset: Some(-300),
            },
        )
        .unwrap();

        assert_eq!(zdt_minus_five.year().unwrap(), 2023);
        assert_eq!(zdt_minus_five.month().unwrap(), 11);
        assert_eq!(zdt_minus_five.day().unwrap(), 29);
        assert_eq!(zdt_minus_five.hour().unwrap(), 20);
        assert_eq!(zdt_minus_five.minute().unwrap(), 49);
        assert_eq!(zdt_minus_five.second().unwrap(), 12);
    }
}
