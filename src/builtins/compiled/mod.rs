//! This module implements native Rust wrappers for the Temporal builtins.

mod date;
mod duration;
mod instant;
mod now;
mod plain_date_time;
mod zoneddatetime;

mod options {
    use crate::{builtins::TZ_PROVIDER, options::RelativeTo, TemporalResult};

    impl RelativeTo {
        pub fn try_from_str(source: &str) -> TemporalResult<Self> {
            Self::try_from_str_with_provider(source, &*TZ_PROVIDER)
        }
    }
}
