//! The `TimeZoneProvider` trait.

use crate::TemporalError;

pub use timezone_provider::{TimeZoneOffset, TimeZoneProvider, TransitionDirection};

pub struct NeverProvider;

impl TimeZoneProvider for NeverProvider {
    type Error = TemporalError;
    fn check_identifier(&self, _: &str) -> bool {
        unreachable!()
    }

    fn get_possible_local_time_seconds(
        &self,
        _: &str,
        _: i64,
    ) -> Result<timezone_provider::PotentialLocalTime, Self::Error> {
        unreachable!()
    }

    fn get_time_zone_offset(
        &self,
        _: &str,
        _: i64,
    ) -> Result<timezone_provider::TimeZoneOffset, Self::Error> {
        unreachable!()
    }

    fn get_time_zone_transition(
        &self,
        _: &str,
        _: i64,
        _: timezone_provider::TransitionDirection,
    ) -> Result<Option<i64>, Self::Error> {
        unreachable!()
    }
}
