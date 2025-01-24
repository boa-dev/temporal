use alloc::string::String;

use crate::{time::EpochNanoseconds, TemporalResult};

#[cfg(feature = "sys")]
use crate::TemporalError;
#[cfg(feature = "sys")]
use alloc::string::ToString;
#[cfg(feature = "sys")]
use web_time::{SystemTime, UNIX_EPOCH};

// TODO: Need to implement SystemTime handling for non_std.

pub trait SystemHooks {
    /// Returns the current system time in `EpochNanoseconds`.
    fn get_system_nanoseconds(&self) -> TemporalResult<EpochNanoseconds>;
    /// Returns the current system IANA Time Zone Identifier.
    fn get_system_time_zone(&self) -> TemporalResult<String>;
}

#[cfg(feature = "sys")]
pub struct DefaultSystemHooks;

#[cfg(feature = "sys")]
impl SystemHooks for DefaultSystemHooks {
    fn get_system_nanoseconds(&self) -> TemporalResult<EpochNanoseconds> {
        EpochNanoseconds::try_from(get_system_nanoseconds()?)
    }

    fn get_system_time_zone(&self) -> TemporalResult<String> {
        iana_time_zone::get_timezone().map_err(|e| TemporalError::general(e.to_string()))
    }
}

/// Returns the system time in nanoseconds.
#[cfg(feature = "sys")]
pub(crate) fn get_system_nanoseconds() -> TemporalResult<u128> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| TemporalError::general(e.to_string()))
        .map(|d| d.as_nanos())
}
