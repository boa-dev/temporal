use super::COMPILED_ZONEINFO_PROVIDER;
use crate::{
    common::LocalTimeRecordResult,
    epoch_nanoseconds::{seconds_to_nanoseconds, EpochNanoseconds, NS_IN_S},
    experimental_tzif::ZeroTzif,
    provider::{
        CandidateEpochNanoseconds, EpochNanosecondsAndOffset, NormalizerAndResolver, ResolvedId,
        TimeZoneProviderResult, TimeZoneResolver, TransitionDirection,
    },
    CompiledNormalizer, TimeZoneProviderError,
};
use zerofrom::ZeroFrom;

/// A zerocopy compiled zone info provider
pub type ZeroZoneInfoProvider<'a> = NormalizerAndResolver<CompiledNormalizer, ZeroZoneInfo>;

#[derive(Debug, Default)]
pub struct ZeroZoneInfo;

impl ZeroZoneInfo {
    pub fn zero_tzif(&self, resolved_id: ResolvedId) -> TimeZoneProviderResult<ZeroTzif<'_>> {
        COMPILED_ZONEINFO_PROVIDER
            .tzifs
            .get(resolved_id.0)
            .map(ZeroTzif::zero_from)
            .ok_or(TimeZoneProviderError::Range(
                "tzif data not found for resolved id",
            ))
    }
}

impl TimeZoneResolver for ZeroZoneInfo {
    fn get_id(&self, normalized_identifier: &[u8]) -> TimeZoneProviderResult<ResolvedId> {
        COMPILED_ZONEINFO_PROVIDER
            .ids
            .get(normalized_identifier)
            .map(ResolvedId)
            .ok_or(TimeZoneProviderError::Range("identifier does not exist."))
    }

    fn candidate_nanoseconds_for_local_epoch_nanoseconds(
        &self,
        identifier: ResolvedId,
        local_datetime: crate::provider::IsoDateTime,
    ) -> TimeZoneProviderResult<crate::provider::CandidateEpochNanoseconds> {
        let tzif = self.zero_tzif(identifier)?;

        let epoch_nanos = (local_datetime).as_nanoseconds();
        let mut seconds = (epoch_nanos.0 / NS_IN_S) as i64;

        // We just rounded our ns value to seconds.
        // This is fine for positive ns: timezones do not transition at sub-second offsets,
        // so the offset at N seconds is always the offset at N.0001 seconds.
        //
        // However, for negative epochs, the offset at -N seconds might be different
        // from that at -N.001 seconds. Instead, we calculate the offset at (-N-1) seconds.
        if seconds < 0 {
            let remainder = epoch_nanos.0 % NS_IN_S;
            if remainder != 0 {
                seconds -= 1;
            }
        }

        let local_time_record_result = tzif.search_candidate_offset(seconds)?;
        let result = match local_time_record_result {
            LocalTimeRecordResult::Empty(bounds) => CandidateEpochNanoseconds::Zero(bounds),
            LocalTimeRecordResult::Single(r) => {
                let epoch_ns = EpochNanoseconds::from(epoch_nanos.0 - seconds_to_nanoseconds(r.0));
                CandidateEpochNanoseconds::One(EpochNanosecondsAndOffset {
                    ns: epoch_ns,
                    offset: r,
                })
            }
            LocalTimeRecordResult::Ambiguous { first, second } => {
                let first_epoch_ns =
                    EpochNanoseconds::from(epoch_nanos.0 - seconds_to_nanoseconds(first.0));
                let second_epoch_ns =
                    EpochNanoseconds::from(epoch_nanos.0 - seconds_to_nanoseconds(second.0));
                CandidateEpochNanoseconds::Two([
                    EpochNanosecondsAndOffset {
                        ns: first_epoch_ns,
                        offset: first,
                    },
                    EpochNanosecondsAndOffset {
                        ns: second_epoch_ns,
                        offset: second,
                    },
                ])
            }
        };
        Ok(result)
    }

    fn transition_nanoseconds_for_utc_epoch_nanoseconds(
        &self,
        identifier: ResolvedId,
        epoch_nanoseconds: i128,
    ) -> TimeZoneProviderResult<crate::provider::UtcOffsetSeconds> {
        let tzif = self.zero_tzif(identifier)?;

        let mut seconds = (epoch_nanoseconds / NS_IN_S) as i64;
        // The rounding is inexact. Transitions are only at second
        // boundaries, so the offset at N s is the same as the offset at N.001,
        // but the offset at -Ns is not the same as the offset at -N.001,
        // the latter matches -N - 1 s instead.
        if seconds < 0 && epoch_nanoseconds % NS_IN_S != 0 {
            seconds -= 1;
        }
        tzif.get(seconds).map(|t| t.offset)
    }

    fn get_time_zone_transition(
        &self,
        identifier: ResolvedId,
        epoch_nanoseconds: i128,
        direction: TransitionDirection,
    ) -> TimeZoneProviderResult<Option<crate::epoch_nanoseconds::EpochNanoseconds>> {
        let tzif = self.zero_tzif(identifier)?;
        tzif.get_time_zone_transition(epoch_nanoseconds, direction)
    }
}
