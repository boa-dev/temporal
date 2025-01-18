
use alloc::string::String;
use crate::{builtins::core, TemporalError, TemporalResult, TimeZone, Instant};
use crate::builtins::std::{ZonedDateTime, PlainDateTime, PlainDate, PlainTime};

use super::timezone::TZ_PROVIDER;

pub struct Now;

impl Now {
    /// Returns the current instant
    pub fn instant() -> TemporalResult<Instant> {
        Now::instant()
    }

    /// Returns the current time zone.
    pub fn time_zone_id() -> TemporalResult<String> {
        Now::time_zone_id()
    }

    /// Returns the current system time as a `ZonedDateTime` with an ISO8601 calendar.
    ///
    /// The time zone will be set to either the `TimeZone` if a value is provided, or
    /// according to the system timezone if no value is provided.
    pub fn zoneddatetime_iso(timezone: Option<TimeZone>) -> TemporalResult<ZonedDateTime> {
        Now::zoneddatetime_iso(timezone).map(Into::into)
    }

    pub fn plain_datetime_iso(timezone: Option<TimeZone>) -> TemporalResult<PlainDateTime> {
        let provider = TZ_PROVIDER
            .lock()
            .map_err(|_| TemporalError::general("Unable to acquire lock"))?;
        core::Now::plain_datetime_iso_with_provider(timezone, &*provider).map(Into::into)
    }

    pub fn plain_date_iso(timezone: Option<TimeZone>) -> TemporalResult<PlainDate> {
        let provider = TZ_PROVIDER
            .lock()
            .map_err(|_| TemporalError::general("Unable to acquire lock"))?;
        core::Now::plain_date_iso_with_provider(timezone, &*provider).map(Into::into)
    }

    pub fn plain_time_iso(timezone: Option<TimeZone>) -> TemporalResult<PlainTime> {
        let provider = TZ_PROVIDER
            .lock()
            .map_err(|_| TemporalError::general("Unable to acquire lock"))?;
        core::Now::plain_time_iso_with_provider(timezone, &*provider).map(Into::into)
    }
}



