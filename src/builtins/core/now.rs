//! The Temporal Now component

use crate::iso::IsoDateTime;
use crate::provider::TimeZoneProvider;
use crate::sys::SystemHooks;
use crate::TemporalResult;
use alloc::string::String;

#[cfg(feature = "sys")]
use crate::sys::DefaultSystemHooks;

use super::{
    calendar::Calendar, timezone::TimeZone, Instant, PlainDate, PlainDateTime, PlainTime,
    ZonedDateTime,
};

/// The Temporal Now object.
pub struct Now;

impl Now {
    pub fn instant_with_hooks(system_hooks: &impl SystemHooks) -> TemporalResult<Instant> {
        let epoch_nanoseconds = system_hooks.get_system_nanoseconds()?;
        Ok(Instant::from(epoch_nanoseconds))
    }

    pub fn system_time_zone_identifier_with_hooks(
        system_hooks: &impl SystemHooks,
    ) -> TemporalResult<String> {
        system_hooks.get_system_time_zone()
    }

    pub fn system_datetime_with_hooks_and_provider(
        time_zone: Option<TimeZone>,
        system_hooks: &impl SystemHooks,
        provider: &impl TimeZoneProvider,
    ) -> TemporalResult<IsoDateTime> {
        // 1. If temporalTimeZoneLike is undefined, then
        // a. Let timeZone be SystemTimeZoneIdentifier().
        // 2. Else,
        // a. Let timeZone be ? ToTemporalTimeZoneIdentifier(temporalTimeZoneLike).
        let tz = time_zone.unwrap_or(TimeZone::IanaIdentifier(
            system_hooks.get_system_time_zone()?,
        ));
        // 3. Let epochNs be SystemUTCEpochNanoseconds().
        let epoch_ns = system_hooks.get_system_nanoseconds()?;
        // 4. Return GetISODateTimeFor(timeZone, epochNs).
        tz.get_iso_datetime_for(&Instant::from(epoch_ns), provider)
    }

    /// Returns the current system time as a `ZonedDateTime` with an ISO8601 calendar.
    ///
    /// The time zone will be set to either the `TimeZone` if a value is provided, or
    /// according to the system timezone if no value is provided.
    pub fn zoneddatetime_iso_with_hooks(
        timezone: Option<TimeZone>,
        system_hooks: &impl SystemHooks,
    ) -> TemporalResult<ZonedDateTime> {
        let timezone = timezone.unwrap_or(TimeZone::IanaIdentifier(
            system_hooks.get_system_time_zone()?,
        ));
        let instant = Self::instant_with_hooks(system_hooks)?;
        Ok(ZonedDateTime::new_unchecked(
            instant,
            Calendar::default(),
            timezone,
        ))
    }
}

#[cfg(feature = "sys")]
impl Now {
    /// Returns the current instant
    pub fn instant() -> TemporalResult<Instant> {
        Self::instant_with_hooks(&DefaultSystemHooks)
    }

    /// Returns the current time zone.
    pub fn time_zone_identifier() -> TemporalResult<String> {
        Self::system_time_zone_identifier_with_hooks(&DefaultSystemHooks)
    }
}

impl Now {
    /// Returns the current system time as a `PlainDateTime` with an ISO8601 calendar.
    ///
    /// The time zone used to calculate the `PlainDateTime` will be set to either the
    /// `TimeZone` if a value is provided, or according to the system timezone if no
    /// value is provided.
    pub fn plain_datetime_iso_with_hooks_and_provider(
        timezone: Option<TimeZone>,
        system_hooks: &impl SystemHooks,
        provider: &impl TimeZoneProvider,
    ) -> TemporalResult<PlainDateTime> {
        let iso = Self::system_datetime_with_hooks_and_provider(timezone, system_hooks, provider)?;
        Ok(PlainDateTime::new_unchecked(iso, Calendar::default()))
    }

    /// Returns the current system time as a `PlainDate` with an ISO8601 calendar.
    ///
    /// The time zone used to calculate the `PlainDate` will be set to either the
    /// `TimeZone` if a value is provided, or according to the system timezone if no
    /// value is provided.
    pub fn plain_date_iso_with_hooks_and_provider(
        timezone: Option<TimeZone>,
        system_hooks: &impl SystemHooks,
        provider: &impl TimeZoneProvider,
    ) -> TemporalResult<PlainDate> {
        let iso = Self::system_datetime_with_hooks_and_provider(timezone, system_hooks, provider)?;
        Ok(PlainDate::new_unchecked(iso.date, Calendar::default()))
    }

    /// Returns the current system time as a `PlainTime` according to an ISO8601 calendar.
    ///
    /// The time zone used to calculate the `PlainTime` will be set to either the
    /// `TimeZone` if a value is provided, or according to the system timezone if no
    /// value is provided.
    pub fn plain_time_iso_with_hooks_and_provider(
        timezone: Option<TimeZone>,
        system_hooks: &impl SystemHooks,
        provider: &impl TimeZoneProvider,
    ) -> TemporalResult<PlainTime> {
        let iso = Self::system_datetime_with_hooks_and_provider(timezone, system_hooks, provider)?;
        Ok(PlainTime::new_unchecked(iso.time))
    }
}

#[cfg(all(feature = "tzdb", feature = "sys"))]
#[cfg(test)]
mod tests {
    use std::thread;
    use std::time::Duration as StdDuration;

    use crate::builtins::core::Now;
    use crate::{options::DifferenceSettings, sys::DefaultSystemHooks, tzdb::FsTzdbProvider};

    #[test]
    fn now_datetime_test() {
        let provider = &FsTzdbProvider::default();
        let system_hooks = DefaultSystemHooks;
        let sleep = 2;

        let before =
            Now::plain_datetime_iso_with_hooks_and_provider(None, &system_hooks, provider).unwrap();
        thread::sleep(StdDuration::from_secs(sleep));
        let after =
            Now::plain_datetime_iso_with_hooks_and_provider(None, &system_hooks, provider).unwrap();

        let diff = after.since(&before, DifferenceSettings::default()).unwrap();

        let sleep_base = sleep as f64;
        let tolerable_range = sleep_base..=sleep_base + 5.0;

        // We assert a tolerable range of sleep + 5 because std::thread::sleep
        // is only guaranteed to be >= the value to sleep. So to prevent sporadic
        // errors, we only assert a range.
        assert!(tolerable_range.contains(&diff.seconds().as_inner()));
        assert!(diff.hours().is_zero());
        assert!(diff.minutes().is_zero());
    }
}
