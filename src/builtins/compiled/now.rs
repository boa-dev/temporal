use crate::builtins::{
    core::{Now, PlainDate, PlainDateTime, PlainTime},
    TZ_PROVIDER,
};
use crate::{TemporalError, TemporalResult, TimeZone};

impl Now {
    pub fn plain_datetime_iso(timezone: Option<TimeZone>) -> TemporalResult<PlainDateTime> {
        let provider = TZ_PROVIDER
            .lock()
            .map_err(|_| TemporalError::general("Unable to acquire lock"))?;
        Now::plain_datetime_iso_with_provider(timezone, &*provider).map(Into::into)
    }

    pub fn plain_date_iso(timezone: Option<TimeZone>) -> TemporalResult<PlainDate> {
        let provider = TZ_PROVIDER
            .lock()
            .map_err(|_| TemporalError::general("Unable to acquire lock"))?;
        Now::plain_date_iso_with_provider(timezone, &*provider).map(Into::into)
    }

    pub fn plain_time_iso(timezone: Option<TimeZone>) -> TemporalResult<PlainTime> {
        let provider = TZ_PROVIDER
            .lock()
            .map_err(|_| TemporalError::general("Unable to acquire lock"))?;
        Now::plain_time_iso_with_provider(timezone, &*provider).map(Into::into)
    }
}
