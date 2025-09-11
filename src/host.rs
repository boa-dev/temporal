//! Trait definitions for accessing values from the host environment.
//!
//! NOTE: This is a power user API.

use timezone_provider::{epoch_nanoseconds::EpochNanoseconds, provider::TimeZoneProvider};

use crate::{TemporalResult, TimeZone, UtcOffset};

/// The `HostClock` trait defines an accessor to the host's clock.
pub trait HostClock {
    fn get_host_epoch_nanoseconds(&self) -> TemporalResult<EpochNanoseconds>;
}

/// The `HostTimeZone` trait defines the host's time zone.
pub trait HostTimeZone {
    fn get_host_time_zone(&self, provider: &impl TimeZoneProvider) -> TemporalResult<TimeZone>;
}

/// `HostHooks` marks whether a trait implements the required host hooks with some
/// system methods.
pub trait HostHooks: HostClock + HostTimeZone {
    fn get_system_epoch_nanoseconds(&self) -> TemporalResult<EpochNanoseconds> {
        self.get_host_epoch_nanoseconds()
    }

    fn get_system_time_zone(&self, provider: &impl TimeZoneProvider) -> TemporalResult<TimeZone> {
        self.get_host_time_zone(provider)
    }
}

// Implement empty providers

impl HostClock for () {
    fn get_host_epoch_nanoseconds(&self) -> TemporalResult<EpochNanoseconds> {
        Ok(EpochNanoseconds::from_seconds(0))
    }
}

impl HostTimeZone for () {
    fn get_host_time_zone(&self, _: &impl TimeZoneProvider) -> TemporalResult<TimeZone> {
        Ok(TimeZone::from(UtcOffset::default()))
    }
}

impl HostHooks for () {}
