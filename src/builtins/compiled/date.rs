use crate::{builtins::TZ_PROVIDER, TemporalError, TemporalResult, Date};

impl Date {

    /// Converts a `Date` to a `ZonedDateTime` in the UTC time zone.
    pub fn to_zoned_date_time (self, time_zone: &str) -> TemporalResult<crate::ZonedDateTime> {
        let provider = TZ_PROVIDER
            .lock()
            .map_err(|_| TemporalError::general("Unable to acquire lock"))?;
        self.to_zoned_date_time_with_provider(time_zone, &*provider)
    }
}