//! The `TimeZoneProvider` trait.

use crate::{iso::IsoDateTime, unix_time::EpochNanoseconds, TemporalResult};
use alloc::borrow::Cow;

pub use timezone_provider::provider::{
    CandidateEpochNanoseconds, EpochNanosecondsAndOffset, GapEntryOffsets, ParseDirectionError,
    TimeZoneTransitionInfo, TransitionDirection, UtcOffsetSeconds,
};

// NOTE: It may be a good idea to eventually move this into it's
// own individual crate rather than having it tied directly into `temporal_rs`
/// The `TimeZoneProvider` trait provides methods required for a provider
/// to implement in order to source time zone data from that provider.
pub trait TimeZoneProvider {
    fn normalize_identifier(&self, ident: &'_ [u8]) -> TemporalResult<Cow<'_, str>>;

    fn canonicalize_identifier(&self, ident: &'_ [u8]) -> TemporalResult<Cow<'_, str>>;

    fn get_named_tz_epoch_nanoseconds(
        &self,
        identifier: &str,
        local_datetime: IsoDateTime,
    ) -> TemporalResult<CandidateEpochNanoseconds>;

    fn get_named_tz_offset_nanoseconds(
        &self,
        identifier: &str,
        epoch_nanoseconds: i128,
    ) -> TemporalResult<TimeZoneTransitionInfo>;

    fn get_named_tz_transition(
        &self,
        identifier: &str,
        epoch_nanoseconds: i128,
        direction: TransitionDirection,
    ) -> TemporalResult<Option<EpochNanoseconds>>;
}

pub struct NeverProvider;

impl TimeZoneProvider for NeverProvider {
    fn normalize_identifier(&self, _ident: &'_ [u8]) -> TemporalResult<Cow<'_, str>> {
        unimplemented!()
    }
    fn canonicalize_identifier(&self, _ident: &'_ [u8]) -> TemporalResult<Cow<'_, str>> {
        unimplemented!()
    }
    fn get_named_tz_epoch_nanoseconds(
        &self,
        _: &str,
        _: IsoDateTime,
    ) -> TemporalResult<CandidateEpochNanoseconds> {
        unimplemented!()
    }

    fn get_named_tz_offset_nanoseconds(
        &self,
        _: &str,
        _: i128,
    ) -> TemporalResult<TimeZoneTransitionInfo> {
        unimplemented!()
    }

    fn get_named_tz_transition(
        &self,
        _: &str,
        _: i128,
        _: TransitionDirection,
    ) -> TemporalResult<Option<EpochNanoseconds>> {
        unimplemented!()
    }
}
