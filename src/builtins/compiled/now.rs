use crate::builtins::{
    core::{Now, PlainDate, PlainDateTime, PlainTime, ZonedDateTime},
    TZ_PROVIDER,
};
use crate::sys;
use crate::{time::EpochNanoseconds, TemporalError, TemporalResult, TimeZone};

#[cfg(feature = "sys")]
impl Now {
    /// Returns the current system time as a [`PlainDateTime`] with an optional
    /// [`TimeZone`].
    pub fn zoneddatetime_iso(timezone: Option<TimeZone>) -> TemporalResult<ZonedDateTime> {
        let timezone =
            timezone.unwrap_or(TimeZone::IanaIdentifier(crate::sys::get_system_timezone()?));
        let system_nanos = sys::get_system_nanoseconds()?;
        let epoch_nanos = EpochNanoseconds::try_from(system_nanos)?;
        Now::zoneddatetime_iso_with_system_values(epoch_nanos, timezone)
    }

    /// Returns the current system time as a [`PlainDateTime`] with an optional
    /// [`TimeZone`].
    ///
    /// Enable with the `compiled_data` feature flag.
    pub fn plain_datetime_iso(timezone: Option<TimeZone>) -> TemporalResult<PlainDateTime> {
        let provider = TZ_PROVIDER
            .lock()
            .map_err(|_| TemporalError::general("Unable to acquire lock"))?;
        let timezone = timezone.unwrap_or(TimeZone::IanaIdentifier(sys::get_system_timezone()?));
        let system_nanos = sys::get_system_nanoseconds()?;
        let epoch_nanos = EpochNanoseconds::try_from(system_nanos)?;
        Now::plain_datetime_iso_with_provider(epoch_nanos, timezone, &*provider)
    }

    /// Returns the current system time as a [`PlainDate`] with an optional
    /// [`TimeZone`].
    ///
    /// Enable with the `compiled_data` feature flag.
    pub fn plain_date_iso(timezone: Option<TimeZone>) -> TemporalResult<PlainDate> {
        let provider = TZ_PROVIDER
            .lock()
            .map_err(|_| TemporalError::general("Unable to acquire lock"))?;
        let timezone = timezone.unwrap_or(TimeZone::IanaIdentifier(sys::get_system_timezone()?));
        let system_nanos = sys::get_system_nanoseconds()?;
        let epoch_nanos = EpochNanoseconds::try_from(system_nanos)?;
        Now::plain_date_iso_with_provider(epoch_nanos, timezone, &*provider)
    }

    /// Returns the current system time as a [`PlainTime`] with an optional
    /// [`TimeZone`].
    ///
    /// Enable with the `compiled_data` feature flag.
    pub fn plain_time_iso(timezone: Option<TimeZone>) -> TemporalResult<PlainTime> {
        let provider = TZ_PROVIDER
            .lock()
            .map_err(|_| TemporalError::general("Unable to acquire lock"))?;
        let timezone = timezone.unwrap_or(TimeZone::IanaIdentifier(sys::get_system_timezone()?));
        let system_nanos = sys::get_system_nanoseconds()?;
        let epoch_nanos = EpochNanoseconds::try_from(system_nanos)?;
        Now::plain_time_iso_with_provider(epoch_nanos, timezone, &*provider)
    }
}
