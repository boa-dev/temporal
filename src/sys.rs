use alloc::string::{String, ToString};

use crate::{TemporalError, TemporalResult};

use std::time::{SystemTime, UNIX_EPOCH};

// TODO: Need to implement system handling for non_std.

/// Returns the system time in nanoseconds.
pub(crate) fn get_system_nanoseconds() -> TemporalResult<u128> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| TemporalError::general(e.to_string()))
        .map(|d| d.as_nanos())
}

/// Returns the system tz identifier
pub(crate) fn get_system_tz_identifier() -> TemporalResult<String> {
    iana_time_zone::get_timezone().map_err(|e| TemporalError::general(e.to_string()))
}
