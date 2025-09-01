use zoneinfo64::{PossibleOffset, ZoneInfo64};

use crate::provider::{
    CandidateEpochNanoseconds, EpochNanosecondsAndOffset, GapEntryOffsets, IsoDateTime,
    TimeZoneProvider, TimeZoneProviderResult, TransitionDirection, UtcOffsetSeconds,
};
use crate::{
    epoch_nanoseconds::{seconds_to_nanoseconds, EpochNanoseconds, NS_IN_S},
    TimeZoneProviderError,
};
use alloc::borrow::Cow;
use icu_time::zone::UtcOffset;

impl From<UtcOffset> for UtcOffsetSeconds {
    fn from(other: UtcOffset) -> Self {
        Self(i64::from(other.to_seconds()))
    }
}

impl TimeZoneProvider for ZoneInfo64<'_> {
    fn normalize_identifier(&self, ident: &'_ [u8]) -> TimeZoneProviderResult<Cow<'_, str>> {
        crate::tzdb::normalize_identifier_with_compiled(ident)
    }
    fn canonicalize_identifier(&self, ident: &'_ [u8]) -> TimeZoneProviderResult<Cow<'_, str>> {
        crate::tzdb::canonicalize_identifier_with_compiled(ident)
    }
    fn get_named_tz_epoch_nanoseconds(
        &self,
        identifier: &str,
        local_datetime: IsoDateTime,
    ) -> TimeZoneProviderResult<CandidateEpochNanoseconds> {
        let Some(zone) = self.get(identifier) else {
            return Err(TimeZoneProviderError::Range("Unknown timezone identifier"));
        };
        let epoch_nanos = (local_datetime).as_nanoseconds();
        let possible_offset = zone.for_date_time(
            local_datetime.year,
            local_datetime.month,
            local_datetime.day,
            local_datetime.hour,
            local_datetime.minute,
            local_datetime.second,
        );
        let result = match possible_offset {
            // TODO(Manishearth) This is wrong
            PossibleOffset::None => CandidateEpochNanoseconds::Zero(GapEntryOffsets::default()),
            PossibleOffset::Single(o) => {
                let epoch_ns = EpochNanoseconds::from(
                    epoch_nanos.0 - seconds_to_nanoseconds(i64::from(o.offset.to_seconds())),
                );
                CandidateEpochNanoseconds::One(EpochNanosecondsAndOffset {
                    ns: epoch_ns,
                    offset: o.offset.into(),
                })
            }
            PossibleOffset::Ambiguous(first, second) => {
                let first_epoch_ns = EpochNanoseconds::from(
                    epoch_nanos.0 - seconds_to_nanoseconds(i64::from(first.offset.to_seconds())),
                );
                let second_epoch_ns = EpochNanoseconds::from(
                    epoch_nanos.0 - seconds_to_nanoseconds(i64::from(first.offset.to_seconds())),
                );
                CandidateEpochNanoseconds::Two([
                    EpochNanosecondsAndOffset {
                        ns: first_epoch_ns,
                        offset: first.offset.into(),
                    },
                    EpochNanosecondsAndOffset {
                        ns: second_epoch_ns,
                        offset: second.offset.into(),
                    },
                ])
            }
        };
        Ok(result)
    }

    fn get_named_tz_offset_nanoseconds(
        &self,
        identifier: &str,
        utc_epoch: i128,
    ) -> TimeZoneProviderResult<UtcOffsetSeconds> {
        let Some(zone) = self.get(identifier) else {
            return Err(TimeZoneProviderError::Range("Unknown timezone identifier"));
        };

        let Ok(mut seconds) = i64::try_from(utc_epoch / NS_IN_S) else {
            return Err(TimeZoneProviderError::Range(
                "Epoch nanoseconds out of range",
            ));
        };
        // The rounding is inexact. Transitions are only at second
        // boundaries, so the offset at N s is the same as the offset at N.001,
        // but the offset at -Ns is not the same as the offset at -N.001,
        // the latter matches -N - 1 s instead.
        if seconds < 0 && utc_epoch % NS_IN_S != 0 {
            seconds -= 1;
        }
        let offset = zone.for_timestamp(seconds);

        Ok(offset.offset.into())
    }

    fn get_named_tz_transition(
        &self,
        identifier: &str,
        epoch_nanoseconds: i128,
        direction: TransitionDirection,
    ) -> TimeZoneProviderResult<Option<EpochNanoseconds>> {
        let Some(zone) = self.get(identifier) else {
            return Err(TimeZoneProviderError::Range("Unknown timezone identifier"));
        };
        let Ok(seconds) = i64::try_from(epoch_nanoseconds / NS_IN_S) else {
            return Err(TimeZoneProviderError::Range(
                "Epoch nanoseconds out of range",
            ));
        };

        let transition = match direction {
            TransitionDirection::Previous => {
                let seconds_is_exact = (epoch_nanoseconds % NS_IN_S) == 0;
                zone.prev_transition(
                    seconds,
                    seconds_is_exact,
                    /* require_offset_change */ true,
                )
            }
            TransitionDirection::Next => {
                zone.next_transition(seconds, /* require_offset_change */ true)
            }
        };

        Ok(transition.map(|transition| EpochNanoseconds::from_seconds(transition.since)))
    }
}
