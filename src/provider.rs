//! The `TimeZoneProvider` trait.

use crate::{iso::IsoDateTime, time::EpochNanoseconds, TemporalResult};
use alloc::vec::Vec;

/// `TimeZoneOffset` represents the number of seconds to be added to UT in order to determine local time.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimeZoneOffset {
    /// The transition time epoch at which the offset needs to be applied.
    pub transition_epoch: Option<i64>,
    /// The time zone offset in seconds.
    pub offset: i64,
}

// NOTE: It may be a good idea to eventually move this into it's
// own individual crate rather than having it tied directly into `temporal_rs`
/// The `TimeZoneProvider` trait provides methods required for a provider
/// to implement in order to source time zone data from that provider.
pub trait TimeZoneProvider {
    fn check_identifier(&self, identifier: &str) -> bool;

    fn get_named_tz_epoch_nanoseconds(
        &self,
        identifier: &str,
        local_datetime: IsoDateTime,
    ) -> TemporalResult<Vec<EpochNanoseconds>>;

    fn get_named_tz_offset_nanoseconds(
        &self,
        identifier: &str,
        utc_epoch: i128,
    ) -> TemporalResult<TimeZoneOffset>;
}

pub struct NeverProvider;

impl TimeZoneProvider for NeverProvider {
    fn check_identifier(&self, _: &str) -> bool {
        unimplemented!()
    }

    fn get_named_tz_epoch_nanoseconds(
        &self,
        _: &str,
        _: IsoDateTime,
    ) -> TemporalResult<Vec<EpochNanoseconds>> {
        unimplemented!()
    }

    fn get_named_tz_offset_nanoseconds(&self, _: &str, _: i128) -> TemporalResult<TimeZoneOffset> {
        unimplemented!()
    }
}
