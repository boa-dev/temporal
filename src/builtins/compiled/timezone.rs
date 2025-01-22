use crate::{builtins::TZ_PROVIDER, TemporalError, TemporalResult, TimeZone};

impl TimeZone {
    pub fn try_from_str(source: &str) -> TemporalResult<Self> {
        let provider = TZ_PROVIDER
            .lock()
            .map_err(|_| TemporalError::general("Unable to acquire lock"))?;

        Self::try_from_str_with_provider(source, &*provider)
    }
}
