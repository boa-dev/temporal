use crate::builtins::{
    core::{Now, PlainDate, PlainDateTime, PlainTime},
    TZ_PROVIDER,
};
use crate::{TemporalError, TemporalResult, TimeZone};

impl Now {
    /// Returns the current system time as a [`PlainDateTime`] with an optional
    /// [`TimeZone`].
    ///
    /// Enable with the `compiled_data` feature flag.
    pub fn plain_datetime_iso(timezone: Option<TimeZone>) -> TemporalResult<PlainDateTime> {
        let provider = TZ_PROVIDER
            .lock()
            .map_err(|_| TemporalError::general("Unable to acquire lock"))?;
        Now::plain_datetime_iso_with_provider(timezone, &*provider).map(Into::into)
    }

    /// Returns the current system time as a [`PlainDate`] with an optional
    /// [`TimeZone`].
    ///
    /// Enable with the `compiled_data` feature flag.
    pub fn plain_date_iso(timezone: Option<TimeZone>) -> TemporalResult<PlainDate> {
        let provider = TZ_PROVIDER
            .lock()
            .map_err(|_| TemporalError::general("Unable to acquire lock"))?;
        Now::plain_date_iso_with_provider(timezone, &*provider).map(Into::into)
    }

    /// Returns the current system time as a [`PlainTime`] with an optional
    /// [`TimeZone`].
    ///
    /// Enable with the `compiled_data` feature flag.
    pub fn plain_time_iso(timezone: Option<TimeZone>) -> TemporalResult<PlainTime> {
        let provider = TZ_PROVIDER
            .lock()
            .map_err(|_| TemporalError::general("Unable to acquire lock"))?;
        Now::plain_time_iso_with_provider(timezone, &*provider).map(Into::into)
    }
}
