use crate::{
    builtins::TZ_PROVIDER, options::ToStringRoundingOptions, Instant, TemporalResult, TimeZone,
};
use alloc::string::String;

impl Instant {
    /// Returns the RFC9557 (IXDTF) string for this `Instant` with the
    /// provided options
    ///
    /// Enable with the `compiled_data` feature flag.
    pub fn to_ixdtf_string(
        &self,
        timezone: Option<&TimeZone>,
        options: ToStringRoundingOptions,
    ) -> TemporalResult<String> {
        self.to_ixdtf_string_with_provider(timezone, options, &*TZ_PROVIDER)
    }
}
