use crate::{
    builtins::TZ_PROVIDER,
    options::{RelativeTo, RoundingOptions},
    Duration, TemporalError, TemporalResult,
};

#[cfg(test)]
mod tests;

impl Duration {
    /// Rounds the current [`Duration`] according to the provided [`RoundingOptions`] and an optional
    /// [`RelativeTo`]
    ///
    /// Enable with the `compiled_data` feature flag.
    pub fn round(
        &self,
        options: RoundingOptions,
        relative_to: Option<RelativeTo>,
    ) -> TemporalResult<Self> {
        let provider = TZ_PROVIDER
            .lock()
            .map_err(|_| TemporalError::general("Unable to acquire lock"))?;
        self.round_with_provider(options, relative_to, &*provider)
            .map(Into::into)
    }
}