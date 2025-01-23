use crate::{builtins::TZ_PROVIDER, TemporalError, TemporalResult, TimeZone};

impl TimeZone {
    /// Attempts to parse `TimeZone` from either a UTC offset or IANA identifier.
    ///
    /// Enable with the `compiled_data` feature flag.
    pub fn try_from_str(source: &str) -> TemporalResult<Self> {
        let provider = TZ_PROVIDER
            .lock()
            .map_err(|_| TemporalError::general("Unable to acquire lock"))?;

        Self::try_from_str_with_provider(source, &*provider)
    }
}
