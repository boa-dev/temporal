//! This module implements `ZonedDateTime` and any directly related algorithms.

use tinystr::TinyAsciiStr;

use crate::{
    components::{calendar::Calendar, tz::TimeZone, Instant},
    TemporalResult,
};

use super::{calendar::CalendarDateLike, tz::TzProvider, PlainDateTime};

#[cfg(feature = "experimental")]
use crate::{components::tz::TZ_PROVIDER, TemporalError};
#[cfg(feature = "experimental")]
use std::ops::DerefMut;

/// The native Rust implementation of `Temporal.ZonedDateTime`.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ZonedDateTime {
    instant: Instant,
    calendar: Calendar,
    tz: TimeZone,
}

impl Ord for ZonedDateTime {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.instant.cmp(&other.instant)
    }
}

impl PartialOrd for ZonedDateTime {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
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
    pub fn new(nanos: i128, calendar: Calendar, tz: TimeZone) -> TemporalResult<Self> {
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
}

// ===== TzProvider APIs for ZonedDateTime =====

#[cfg(feature = "experimental")]
impl ZonedDateTime {
    pub fn year(&self) -> TemporalResult<i32> {
        let mut provider = TZ_PROVIDER
            .lock()
            .map_err(|_| TemporalError::general("Unable to acquire lock"))?;
        self.year_with_provider(provider.deref_mut())
    }

    pub fn month(&self) -> TemporalResult<u8> {
        let mut provider = TZ_PROVIDER
            .lock()
            .map_err(|_| TemporalError::general("Unable to acquire lock"))?;
        self.month_with_provider(provider.deref_mut())
    }

    pub fn month_code(&self) -> TemporalResult<TinyAsciiStr<4>> {
        let mut provider = TZ_PROVIDER
            .lock()
            .map_err(|_| TemporalError::general("Unable to acquire lock"))?;
        self.month_code_with_provider(provider.deref_mut())
    }

    pub fn day(&self) -> TemporalResult<u8> {
        let mut provider = TZ_PROVIDER
            .lock()
            .map_err(|_| TemporalError::general("Unable to acquire lock"))?;
        self.day_with_provider(provider.deref_mut())
    }

    pub fn hour(&self) -> TemporalResult<u8> {
        let mut provider = TZ_PROVIDER
            .lock()
            .map_err(|_| TemporalError::general("Unable to acquire lock"))?;
        self.hour_with_provider(provider.deref_mut())
    }

    pub fn minute(&self) -> TemporalResult<u8> {
        let mut provider = TZ_PROVIDER
            .lock()
            .map_err(|_| TemporalError::general("Unable to acquire lock"))?;
        self.minute_with_provider(provider.deref_mut())
    }

    pub fn second(&self) -> TemporalResult<u8> {
        let mut provider = TZ_PROVIDER
            .lock()
            .map_err(|_| TemporalError::general("Unable to acquire lock"))?;
        self.second_with_provider(provider.deref_mut())
    }
}

impl ZonedDateTime {
    /// Returns the `year` value for this `ZonedDateTime`.
    #[inline]
    pub fn year_with_provider(&self, provider: &mut impl TzProvider) -> TemporalResult<i32> {
        let iso = self.tz.get_iso_datetime_for(&self.instant, provider)?;
        let dt = PlainDateTime::new_unchecked(iso, self.calendar.clone());
        self.calendar.year(&CalendarDateLike::DateTime(&dt))
    }

    /// Returns the `month` value for this `ZonedDateTime`.
    pub fn month_with_provider(&self, provider: &mut impl TzProvider) -> TemporalResult<u8> {
        let iso = self.tz.get_iso_datetime_for(&self.instant, provider)?;
        let dt = PlainDateTime::new_unchecked(iso, self.calendar.clone());
        self.calendar.month(&CalendarDateLike::DateTime(&dt))
    }

    /// Returns the `monthCode` value for this `ZonedDateTime`.
    pub fn month_code_with_provider(
        &self,
        provider: &mut impl TzProvider,
    ) -> TemporalResult<TinyAsciiStr<4>> {
        let iso = self.tz.get_iso_datetime_for(&self.instant, provider)?;
        let dt = PlainDateTime::new_unchecked(iso, self.calendar.clone());
        self.calendar.month_code(&CalendarDateLike::DateTime(&dt))
    }

    /// Returns the `day` value for this `ZonedDateTime`.
    pub fn day_with_provider(&self, provider: &mut impl TzProvider) -> TemporalResult<u8> {
        let iso = self.tz.get_iso_datetime_for(&self.instant, provider)?;
        let dt = PlainDateTime::new_unchecked(iso, self.calendar.clone());
        self.calendar.day(&CalendarDateLike::DateTime(&dt))
    }

    /// Returns the `hour` value for this `ZonedDateTime`.
    pub fn hour_with_provider(&self, provider: &mut impl TzProvider) -> TemporalResult<u8> {
        let iso = self.tz.get_iso_datetime_for(&self.instant, provider)?;
        Ok(iso.time.hour)
    }

    /// Returns the `minute` value for this `ZonedDateTime`.
    pub fn minute_with_provider(&self, provider: &mut impl TzProvider) -> TemporalResult<u8> {
        let iso = self.tz.get_iso_datetime_for(&self.instant, provider)?;
        Ok(iso.time.minute)
    }

    /// Returns the `second` value for this `ZonedDateTime`.
    pub fn second_with_provider(&self, provider: &mut impl TzProvider) -> TemporalResult<u8> {
        let iso = self.tz.get_iso_datetime_for(&self.instant, provider)?;
        Ok(iso.time.second)
    }

    /// Returns the `millisecond` value for this `ZonedDateTime`.
    pub fn millisecond_with_provider(&self, provider: &mut impl TzProvider) -> TemporalResult<u16> {
        let iso = self.tz.get_iso_datetime_for(&self.instant, provider)?;
        Ok(iso.time.millisecond)
    }

    /// Returns the `microsecond` value for this `ZonedDateTime`.
    pub fn microsecond_with_provider(&self, provider: &mut impl TzProvider) -> TemporalResult<u16> {
        let iso = self.tz.get_iso_datetime_for(&self.instant, provider)?;
        Ok(iso.time.millisecond)
    }

    /// Returns the `nanosecond` value for this `ZonedDateTime`.
    pub fn nanosecond_with_provider(&self, provider: &mut impl TzProvider) -> TemporalResult<u16> {
        let iso = self.tz.get_iso_datetime_for(&self.instant, provider)?;
        Ok(iso.time.nanosecond)
    }
}

#[cfg(feature = "tzdb")]
#[cfg(test)]
mod tests {

    use core::str::FromStr;

    use crate::{components::calendar::Calendar, tzdb::FsTzdbProvider};

    use super::ZonedDateTime;

    #[test]
    fn basic_zdt_test() {
        let provider = &mut FsTzdbProvider::default();
        let nov_30_2023_utc = 1_701_308_952_000_000_000i128;

        let zdt = ZonedDateTime::new(
            nov_30_2023_utc,
            Calendar::from_str("iso8601").unwrap(),
            "Z".into(),
        )
        .unwrap();

        assert_eq!(zdt.year_with_provider(provider).unwrap(), 2023);
        assert_eq!(zdt.month_with_provider(provider).unwrap(), 11);
        assert_eq!(zdt.day_with_provider(provider).unwrap(), 30);
        assert_eq!(zdt.hour_with_provider(provider).unwrap(), 1);
        assert_eq!(zdt.minute_with_provider(provider).unwrap(), 49);
        assert_eq!(zdt.second_with_provider(provider).unwrap(), 12);

        let zdt_minus_five = ZonedDateTime::new(
            nov_30_2023_utc,
            Calendar::from_str("iso8601").unwrap(),
            "America/New_York".into(),
        )
        .unwrap();

        assert_eq!(zdt_minus_five.year_with_provider(provider).unwrap(), 2023);
        assert_eq!(zdt_minus_five.month_with_provider(provider).unwrap(), 11);
        assert_eq!(zdt_minus_five.day_with_provider(provider).unwrap(), 29);
        assert_eq!(zdt_minus_five.hour_with_provider(provider).unwrap(), 20);
        assert_eq!(zdt_minus_five.minute_with_provider(provider).unwrap(), 49);
        assert_eq!(zdt_minus_five.second_with_provider(provider).unwrap(), 12);

        let zdt_plus_eleven = ZonedDateTime::new(
            nov_30_2023_utc,
            Calendar::from_str("iso8601").unwrap(),
            "Australia/Sydney".into(),
        )
        .unwrap();

        assert_eq!(zdt_plus_eleven.year_with_provider(provider).unwrap(), 2023);
        assert_eq!(zdt_plus_eleven.month_with_provider(provider).unwrap(), 11);
        assert_eq!(zdt_plus_eleven.day_with_provider(provider).unwrap(), 30);
        assert_eq!(zdt_plus_eleven.hour_with_provider(provider).unwrap(), 12);
        assert_eq!(zdt_plus_eleven.minute_with_provider(provider).unwrap(), 49);
        assert_eq!(zdt_plus_eleven.second_with_provider(provider).unwrap(), 12);
    }

    #[cfg(all(feature = "experimental", not(target_os = "windows")))]
    #[test]
    fn static_tzdb_zdt_test() {
        let nov_30_2023_utc = 1_701_308_952_000_000_000i128;

        let zdt = ZonedDateTime::new(
            nov_30_2023_utc,
            Calendar::from_str("iso8601").unwrap(),
            "Z".into(),
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
            "America/New_York".into(),
        )
        .unwrap();

        assert_eq!(zdt_minus_five.year().unwrap(), 2023);
        assert_eq!(zdt_minus_five.month().unwrap(), 11);
        assert_eq!(zdt_minus_five.day().unwrap(), 29);
        assert_eq!(zdt_minus_five.hour().unwrap(), 20);
        assert_eq!(zdt_minus_five.minute().unwrap(), 49);
        assert_eq!(zdt_minus_five.second().unwrap(), 12);

        let zdt_plus_eleven = ZonedDateTime::new(
            nov_30_2023_utc,
            Calendar::from_str("iso8601").unwrap(),
            "Australia/Sydney".into(),
        )
        .unwrap();

        assert_eq!(zdt_plus_eleven.year().unwrap(), 2023);
        assert_eq!(zdt_plus_eleven.month().unwrap(), 11);
        assert_eq!(zdt_plus_eleven.day().unwrap(), 30);
        assert_eq!(zdt_plus_eleven.hour().unwrap(), 12);
        assert_eq!(zdt_plus_eleven.minute().unwrap(), 49);
        assert_eq!(zdt_plus_eleven.second().unwrap(), 12);
    }
}
