//! The Temporal Now component

use crate::provider::TimeZoneProvider;
use crate::sys::{SystemClock, SystemTimeZone};
use crate::time::EpochNanoseconds;
use crate::TemporalResult;
use crate::{iso::IsoDateTime, TemporalError};
use alloc::string::ToString;

use super::{
    calendar::Calendar, timezone::TimeZone, Instant, PlainDate, PlainDateTime, PlainTime,
    ZonedDateTime,
};

#[derive(Debug, Default)]
pub struct EmptySystemClock;

impl SystemClock for EmptySystemClock {
    type Error = TemporalError;
    fn get_system_epoch_nanoseconds(&self) -> Result<EpochNanoseconds, Self::Error> {
        Err(TemporalError::general("System clock is not defined"))
    }
}

#[derive(Default)]
pub struct EmptySystemZone;

impl SystemTimeZone for EmptySystemZone {
    type Error = TemporalError;
    fn get_system_time_zone(&self) -> Result<TimeZone, Self::Error> {
        Ok(TimeZone::default())
    }
}

/// The Temporal Now object.
#[derive(Default)]
#[non_exhaustive] // Now cannot be constructed with a struct expression
pub struct Now<Clock: SystemClock = EmptySystemClock, TimeZone: SystemTimeZone = EmptySystemZone> {
    pub(crate) clock: Clock,
    pub(crate) system_zone: TimeZone,
}

impl<C: SystemClock, T: SystemTimeZone> Now<C, T> {
    pub(crate) fn clock(&self) -> TemporalResult<EpochNanoseconds> {
        self.clock
            .get_system_epoch_nanoseconds()
            .map_err(|e| TemporalError::general(e.to_string()))
    }

    /// Returns the current system `DateTime` based off the provided system args
    pub(crate) fn system_datetime_with_provider(
        &self,
        time_zone: Option<TimeZone>,
        provider: &impl TimeZoneProvider,
    ) -> TemporalResult<IsoDateTime> {
        // 1. If temporalTimeZoneLike is undefined, then
        // a. Let timeZone be SystemTimeZoneIdentifier().
        // 2. Else,
        // a. Let timeZone be ? ToTemporalTimeZoneIdentifier(temporalTimeZoneLike).
        // 3. Let epochNs be SystemUTCEpochNanoseconds().
        // 4. Return GetISODateTimeFor(timeZone, epochNs).
        let time_zone = time_zone.unwrap_or(
            self.system_zone
                .get_system_time_zone()
                .map_err(|e| TemporalError::general(e.to_string()))?,
        );
        time_zone.get_iso_datetime_for(&Instant::from(self.clock()?), provider)
    }
}

impl<C: SystemClock, T: SystemTimeZone> Now<C, T> {
    /// Set `Now`'s system clock
    pub fn with_system_clock(mut self, clock: C) -> Self {
        self.clock = clock;
        self
    }

    /// Set `Now`'s system time zone fetcher
    pub fn with_system_time_zone(mut self, time_zone: T) -> Self {
        self.system_zone = time_zone;
        self
    }

    /// Return's the system time zone
    pub fn time_zone(&self) -> TemporalResult<TimeZone> {
        self.system_zone
            .get_system_time_zone()
            .map_err(|e| TemporalError::general(e.to_string()))
    }

    /// Returns the current instant
    ///
    /// Enable with the `sys` feature flag.
    pub fn instant(&self) -> TemporalResult<Instant> {
        let epoch_nanos = self.clock()?;
        Ok(Instant::from(epoch_nanos))
    }

    /// Returns the current system time as a [`PlainDateTime`] with an optional
    /// [`TimeZone`].
    ///
    /// Enable with the `sys` feature flag.
    pub fn zoned_date_time_iso(
        &self,
        time_zone: Option<TimeZone>,
    ) -> TemporalResult<ZonedDateTime> {
        let time_zone = time_zone.unwrap_or(
            self.system_zone
                .get_system_time_zone()
                .map_err(|e| TemporalError::general(e.to_string()))?,
        );
        Ok(ZonedDateTime::new_unchecked(
            Instant::from(self.clock()?),
            Calendar::default(),
            time_zone,
        ))
    }
}

impl<C: SystemClock, T: SystemTimeZone> Now<C, T> {
    /// Returns the current system time as a `PlainDateTime` with an ISO8601 calendar.
    pub fn plain_date_time_iso_with_provider(
        &self,
        time_zone: Option<TimeZone>,
        provider: &impl TimeZoneProvider,
    ) -> TemporalResult<PlainDateTime> {
        let iso = self.system_datetime_with_provider(time_zone, provider)?;
        Ok(PlainDateTime::new_unchecked(iso, Calendar::default()))
    }

    /// Returns the current system time as a `PlainDate` with an ISO8601 calendar.
    pub fn plain_date_iso_with_provider(
        &self,
        time_zone: Option<TimeZone>,
        provider: &impl TimeZoneProvider,
    ) -> TemporalResult<PlainDate> {
        let iso = self.system_datetime_with_provider(time_zone, provider)?;
        Ok(PlainDate::new_unchecked(iso.date, Calendar::default()))
    }

    /// Returns the current system time as a `PlainTime` according to an ISO8601 calendar.
    pub fn plain_time_iso_with_provider(
        &self,
        time_zone: Option<TimeZone>,
        provider: &impl TimeZoneProvider,
    ) -> TemporalResult<PlainTime> {
        let iso = self.system_datetime_with_provider(time_zone, provider)?;
        Ok(PlainTime::new_unchecked(iso.time))
    }
}

#[cfg(test)]
mod tests {

    #[cfg(feature = "tzdb")]
    use crate::options::DifferenceSettings;

    #[cfg(feature = "tzdb")]
    #[test]
    fn mocked_datetime() {
        use crate::{
            sys::{SystemClock, SystemTimeZone},
            time::EpochNanoseconds,
            tzdb::FsTzdbProvider,
            Now, TemporalError, TimeZone,
        };

        let provider = FsTzdbProvider::default();

        // 2025-03-11T10:47-06:00
        const TIME_BASE: u128 = 1_741_751_188_077_363_694;

        let cdt = TimeZone::try_from_identifier_str("-05:00").unwrap();

        #[derive(Default)]
        struct StaticTime;

        #[cfg(feature = "tzdb")]
        impl SystemClock for StaticTime {
            type Error = TemporalError;
            fn get_system_epoch_nanoseconds(&self) -> Result<EpochNanoseconds, Self::Error> {
                Ok(EpochNanoseconds::try_from(TIME_BASE).unwrap())
            }
        }

        #[derive(Default)]
        struct StaticUsChi;

        #[cfg(feature = "tzdb")]
        impl SystemTimeZone for StaticUsChi {
            type Error = TemporalError;
            fn get_system_time_zone(&self) -> Result<crate::TimeZone, Self::Error> {
                let uschi = TimeZone::try_from_identifier_str("America/Chicago").unwrap();
                Ok(uschi)
            }
        }

        let uschi_now = Now::default()
            .with_system_clock(StaticTime)
            .with_system_time_zone(StaticUsChi);

        let cdt_zdt = uschi_now.zoned_date_time_iso(Some(cdt)).unwrap();
        assert_eq!(cdt_zdt.timezone().identifier().unwrap(), "-05:00");
        assert_eq!(cdt_zdt.year_with_provider(&provider).unwrap(), 2025);
        assert_eq!(cdt_zdt.month_with_provider(&provider).unwrap(), 3);
        assert_eq!(
            cdt_zdt
                .month_code_with_provider(&provider)
                .unwrap()
                .as_str(),
            "M03"
        );
        assert_eq!(cdt_zdt.day_with_provider(&provider).unwrap(), 11);
        assert_eq!(cdt_zdt.hour_with_provider(&provider).unwrap(), 22);
        assert_eq!(cdt_zdt.minute_with_provider(&provider).unwrap(), 46);
        assert_eq!(cdt_zdt.second_with_provider(&provider).unwrap(), 28);
        assert_eq!(cdt_zdt.millisecond_with_provider(&provider).unwrap(), 77);
        assert_eq!(cdt_zdt.microsecond_with_provider(&provider).unwrap(), 363);
        assert_eq!(cdt_zdt.nanosecond_with_provider(&provider).unwrap(), 694);

        let uschi_zdt = uschi_now.zoned_date_time_iso(None).unwrap();
        let uschi_pdt = uschi_zdt
            .to_plain_datetime_with_provider(&provider)
            .unwrap();
        assert_eq!(
            uschi_zdt.timezone().identifier().unwrap(),
            "America/Chicago"
        );
        assert_eq!(
            uschi_pdt,
            cdt_zdt.to_plain_datetime_with_provider(&provider).unwrap()
        );

        #[derive(Default)]
        struct StaticTimePlusFive;

        #[cfg(feature = "tzdb")]
        impl SystemClock for StaticTimePlusFive {
            type Error = TemporalError;
            fn get_system_epoch_nanoseconds(&self) -> Result<EpochNanoseconds, Self::Error> {
                let plus_5_secs = TIME_BASE + (5 * 1_000_000_000);
                Ok(EpochNanoseconds::try_from(plus_5_secs).unwrap())
            }
        }

        let now_plus_five = Now::default()
            .with_system_clock(StaticTimePlusFive)
            .with_system_time_zone(StaticUsChi);
        let plus_five_pdt = now_plus_five
            .plain_date_time_iso_with_provider(None, &provider)
            .unwrap();
        assert_eq!(plus_five_pdt.second(), 33);

        let duration = uschi_pdt
            .until(&plus_five_pdt, DifferenceSettings::default())
            .unwrap();
        assert_eq!(duration.hours(), 0);
        assert_eq!(duration.minutes(), 0);
        assert_eq!(duration.seconds(), 5);
        assert_eq!(duration.milliseconds(), 0);
    }

    #[cfg(all(feature = "sys", feature = "compiled_data"))]
    #[test]
    fn now_datetime_test() {
        use crate::sys::Temporal;
        use std::thread;
        use std::time::Duration as StdDuration;

        let sleep = 2;

        let before = Temporal::now().plain_date_time_iso(None).unwrap();
        thread::sleep(StdDuration::from_secs(sleep));
        let after = Temporal::now().plain_date_time_iso(None).unwrap();

        let diff = after.since(&before, DifferenceSettings::default()).unwrap();

        let sleep_base = sleep as i64;
        let tolerable_range = sleep_base..=sleep_base + 5;

        // We assert a tolerable range of sleep + 5 because std::thread::sleep
        // is only guaranteed to be >= the value to sleep. So to prevent sporadic
        // errors, we only assert a range.
        assert!(tolerable_range.contains(&diff.seconds()));
        assert_eq!(diff.hours(), 0);
        assert_eq!(diff.minutes(), 0);
    }
}
