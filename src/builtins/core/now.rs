//! The Temporal Now component

use crate::iso::IsoDateTime;
use crate::provider::TimeZoneProvider;
use crate::time::EpochNanoseconds;
use crate::TemporalResult;

#[cfg(feature = "sys")]
use alloc::string::String;

use super::{
    calendar::Calendar, timezone::TimeZone, Instant, PlainDate, PlainDateTime, PlainTime,
    ZonedDateTime,
};

/// The Temporal Now object.
pub struct Now;

impl Now {
    pub fn instant_with_epoch_nanoseconds(epoch_nanoseconds: EpochNanoseconds) -> Instant {
        Instant::from(epoch_nanoseconds)
    }

    /// Returns the current system `DateTime` based off the provided system args
    ///
    /// ## Order of operations
    ///
    /// The order of operations for this method requires the `GetSystemTimeZone` call
    /// to occur prior to calling system time and resolving the `EpochNanoseconds`
    /// value.
    ///
    /// A correct implementation will follow the following steps:
    ///
    ///   1. Resolve user input `TimeZone` with the `SystemTimeZone`.
    ///   2. Get the `SystemNanoseconds`
    ///
    /// For an example implementation see [`Self::zoneddatetime_iso`]
    ///
    pub(crate) fn system_datetime_with_provider(
        epoch_nanoseconds: EpochNanoseconds,
        timezone: TimeZone,
        provider: &impl TimeZoneProvider,
    ) -> TemporalResult<IsoDateTime> {
        // 1. If temporalTimeZoneLike is undefined, then
        // a. Let timeZone be SystemTimeZoneIdentifier().
        // 2. Else,
        // a. Let timeZone be ? ToTemporalTimeZoneIdentifier(temporalTimeZoneLike).
        // 3. Let epochNs be SystemUTCEpochNanoseconds().
        // 4. Return GetISODateTimeFor(timeZone, epochNs).
        timezone.get_iso_datetime_for(&Instant::from(epoch_nanoseconds), provider)
    }

    /// Returns the current system time as a `ZonedDateTime` with an ISO8601 calendar.
    ///
    /// The time zone will be set to either the `TimeZone` if a value is provided, or
    /// according to the system timezone if no value is provided.
    ///
    /// ## Order of operations
    ///
    /// The order of operations for this method requires the `GetSystemTimeZone` call
    /// to occur prior to calling system time and resolving the `EpochNanoseconds`
    /// value.
    ///
    /// A correct implementation will follow the following steps:
    ///
    ///   1. Resolve user input `TimeZone` with the `SystemTimeZone`.
    ///   2. Get the `SystemNanoseconds`
    ///
    /// For an example implementation see [`Self::zoneddatetime_iso`]
    pub fn zoneddatetime_iso_with_system_values(
        epoch_nanos: EpochNanoseconds,
        timezone: TimeZone,
    ) -> TemporalResult<ZonedDateTime> {
        let instant = Self::instant_with_epoch_nanoseconds(epoch_nanos);
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
        let system_nanos = crate::sys::get_system_nanoseconds()?;
        let epoch_nanos = EpochNanoseconds::try_from(system_nanos)?;
        Ok(Self::instant_with_epoch_nanoseconds(epoch_nanos))
    }

    /// Returns the current time zone.
    pub fn time_zone_identifier() -> TemporalResult<String> {
        crate::sys::get_system_timezone()
    }
}

impl Now {
    /// Returns the current system time as a `PlainDateTime` with an ISO8601 calendar.
    ///
    /// ## Order of operations
    ///
    /// The order of operations for this method requires the `GetSystemTimeZone` call
    /// to occur prior to calling system time and resolving the `EpochNanoseconds`
    /// value.
    ///
    /// A correct implementation will follow the following steps:
    ///
    ///   1. Resolve user input `TimeZone` with the `SystemTimeZone`.
    ///   2. Get the `SystemNanoseconds`
    ///
    /// For an example implementation see [`Self::plain_datetime_iso`]
    pub fn plain_datetime_iso_with_provider(
        epoch_nanos: EpochNanoseconds,
        timezone: TimeZone,
        provider: &impl TimeZoneProvider,
    ) -> TemporalResult<PlainDateTime> {
        let iso = Self::system_datetime_with_provider(epoch_nanos, timezone, provider)?;
        Ok(PlainDateTime::new_unchecked(iso, Calendar::default()))
    }

    /// Returns the current system time as a `PlainDate` with an ISO8601 calendar.
    ///
    /// ## Order of operations
    ///
    /// The order of operations for this method requires the `GetSystemTimeZone` call
    /// to occur prior to calling system time and resolving the `EpochNanoseconds`
    /// value.
    ///
    /// A correct implementation will follow the following steps:
    ///
    ///   1. Resolve user input `TimeZone` with the `SystemTimeZone`.
    ///   2. Get the `SystemNanoseconds`
    ///
    /// For an example implementation see [`Self::plain_date_iso`]
    pub fn plain_date_iso_with_provider(
        epoch_nanos: EpochNanoseconds,
        timezone: TimeZone,
        provider: &impl TimeZoneProvider,
    ) -> TemporalResult<PlainDate> {
        let iso = Self::system_datetime_with_provider(epoch_nanos, timezone, provider)?;
        Ok(PlainDate::new_unchecked(iso.date, Calendar::default()))
    }

    /// Returns the current system time as a `PlainTime` according to an ISO8601 calendar.
    ///
    /// ## Order of operations
    ///
    /// The order of operations for this method requires the `GetSystemTimeZone` call
    /// to occur prior to calling system time and resolving the `EpochNanoseconds`
    /// value.
    ///
    /// A correct implementation will follow the following steps:
    ///
    ///   1. Resolve user input `TimeZone` with the `SystemTimeZone`.
    ///   2. Get the `SystemNanoseconds`
    ///
    /// For an example implementation see [`Self::plain_time_iso`]
    pub fn plain_time_iso_with_provider(
        epoch_nanos: EpochNanoseconds,
        timezone: TimeZone,
        provider: &impl TimeZoneProvider,
    ) -> TemporalResult<PlainTime> {
        let iso = Self::system_datetime_with_provider(epoch_nanos, timezone, provider)?;
        Ok(PlainTime::new_unchecked(iso.time))
    }
}

#[cfg(all(feature = "tzdb", feature = "sys"))]
#[cfg(test)]
mod tests {
    use crate::builtins::core::Now;
    use std::thread;
    use std::time::Duration as StdDuration;

    use crate::options::DifferenceSettings;

    #[test]
    fn now_datetime_test() {
        let sleep = 2;

        let before = Now::plain_datetime_iso(None).unwrap();
        thread::sleep(StdDuration::from_secs(sleep));
        let after = Now::plain_datetime_iso(None).unwrap();

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
