//! This module implements the Temporal `TimeZone` and components.

use alloc::string::{String, ToString};

use ixdtf::encoding::Utf8;
use ixdtf::{
    parsers::TimeZoneParser,
    records::{MinutePrecisionOffset, TimeZoneRecord, UtcOffsetRecord},
};
use num_traits::ToPrimitive;

use crate::error::ErrorMessage;
use crate::parsers::{
    parse_allowed_timezone_formats, parse_identifier, FormattableOffset, FormattableTime, Precision,
};
use crate::provider::{CandidateEpochNanoseconds, TimeZoneProvider};
use crate::Sign;
use crate::{
    builtins::core::{duration::normalized::TimeDuration, Instant},
    iso::{IsoDate, IsoDateTime, IsoTime},
    options::Disambiguation,
    TemporalError, TemporalResult, TemporalUnwrap, ZonedDateTime,
};

use crate::provider::EpochNanosecondsAndOffset;

const NS_IN_S: i64 = 1_000_000_000;
const NS_IN_MIN: i64 = 60_000_000_000;

/// A UTC time zone offset stored in nanoseconds
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UtcOffset(i64);

impl UtcOffset {
    pub(crate) fn from_ixdtf_minute_record(record: MinutePrecisionOffset) -> Self {
        // NOTE: ixdtf parser restricts minute/second to 0..=60
        let minutes = i16::from(record.hour) * 60 + record.minute as i16;
        let minutes = minutes * i16::from(record.sign as i8);
        Self::from_minutes(minutes)
    }
    pub(crate) fn from_ixdtf_record(record: UtcOffsetRecord) -> TemporalResult<Self> {
        let hours = i64::from(record.hour());
        let minutes = 60 * hours + i64::from(record.minute());
        let sign = record.sign() as i64;

        if let Some(second) = record.second() {
            let seconds = 60 * minutes + i64::from(second);

            let mut ns = seconds * NS_IN_S;

            if let Some(frac) = record.fraction() {
                ns += i64::from(
                    frac.to_nanoseconds().ok_or(
                        TemporalError::range()
                            .with_enum(ErrorMessage::FractionalTimeMoreThanNineDigits),
                    )?,
                );
            }

            Ok(Self(ns * sign))
        } else {
            Ok(Self(minutes * sign * NS_IN_MIN))
        }
    }

    pub fn from_utf8(source: &[u8]) -> TemporalResult<Self> {
        let record = TimeZoneParser::from_utf8(source)
            .parse_offset()
            .map_err(|e| TemporalError::range().with_message(e.to_string()))?;
        Self::from_ixdtf_record(record)
    }

    #[allow(clippy::inherent_to_string)]
    pub fn to_string(&self) -> String {
        let sign = if self.0 < 0 {
            Sign::Negative
        } else {
            Sign::Positive
        };
        let nanoseconds_total = self.0.abs();

        let nanosecond = u32::try_from(nanoseconds_total % NS_IN_S).unwrap_or(0);
        let seconds_left = nanoseconds_total / NS_IN_S;

        let second = u8::try_from(seconds_left % 60).unwrap_or(0);
        let minutes_left = seconds_left / 60;

        let minute = u8::try_from(minutes_left % 60).unwrap_or(0);
        let hour = u8::try_from(minutes_left / 60).unwrap_or(0);

        let precision = if nanosecond == 0 && second == 0 {
            Precision::Minute
        } else {
            Precision::Auto
        };
        let formattable_offset = FormattableOffset {
            sign,
            time: FormattableTime {
                hour,
                minute,
                second,
                nanosecond,
                precision,
                include_sep: true,
            },
        };
        formattable_offset.to_string()
    }

    pub fn from_minutes(minutes: i16) -> Self {
        Self(i64::from(minutes) * NS_IN_MIN)
    }

    pub(crate) fn from_seconds(seconds: i64) -> Self {
        Self(seconds * crate::builtins::core::instant::NANOSECONDS_PER_SECOND)
    }
    pub(crate) fn from_nanos(nanos: i64) -> Self {
        Self(nanos)
    }
    pub fn minutes(&self) -> i16 {
        i16::try_from(self.0 / NS_IN_MIN).unwrap_or(0)
    }

    pub fn nanoseconds(&self) -> i64 {
        self.0
    }

    pub fn is_sub_minute(&self) -> bool {
        self.0 % NS_IN_MIN != 0
    }

    /// Partial implementation of GetISODateTimeFor for a cached offset
    pub(crate) fn get_iso_datetime_for(&self, instant: &Instant) -> TemporalResult<IsoDateTime> {
        // 2. Let result be GetISOPartsFromEpoch(ℝ(epochNs)).
        // 3. Return BalanceISODateTime(result.[[ISODate]].[[Year]], result.[[ISODate]].[[Month]], result.[[ISODate]].[[Day]],
        // result.[[Time]].[[Hour]], result.[[Time]].[[Minute]], result.[[Time]].[[Second]], result.[[Time]].[[Millisecond]],
        // result.[[Time]].[[Microsecond]], result.[[Time]].[[Nanosecond]] + offsetNanoseconds).
        IsoDateTime::from_epoch_nanos(instant.epoch_nanoseconds(), self.nanoseconds())
    }
}

impl core::str::FromStr for UtcOffset {
    type Err = TemporalError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_utf8(s.as_bytes())
    }
}

// TODO: Potentially migrate to Cow<'a, str>
// TODO: There may be an argument to have Offset minutes be a (Cow<'a, str>,, i16) to
// prevent allocations / writing, TBD
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TimeZone {
    IanaIdentifier(String),
    UtcOffset(UtcOffset),
}

impl TimeZone {
    // Create a `TimeZone` from an ixdtf `TimeZoneRecord`.
    #[inline]
    pub(crate) fn from_time_zone_record(
        record: TimeZoneRecord<Utf8>,
        provider: &impl TimeZoneProvider,
    ) -> TemporalResult<Self> {
        let timezone = match record {
            TimeZoneRecord::Name(name) => {
                TimeZone::IanaIdentifier(provider.normalize_identifier(name)?.into())
            }
            TimeZoneRecord::Offset(offset_record) => {
                let offset = UtcOffset::from_ixdtf_minute_record(offset_record);
                TimeZone::UtcOffset(offset)
            }
            // TimeZoneRecord is non_exhaustive, but all current branches are matching.
            _ => return Err(TemporalError::assert()),
        };

        Ok(timezone)
    }

    /// Parses a `TimeZone` from a provided `&str`.
    pub fn try_from_identifier_str_with_provider(
        identifier: &str,
        provider: &impl TimeZoneProvider,
    ) -> TemporalResult<Self> {
        parse_identifier(identifier).map(|tz| match tz {
            TimeZoneRecord::Name(name) => Ok(TimeZone::IanaIdentifier(
                provider.normalize_identifier(name)?.into(),
            )),
            TimeZoneRecord::Offset(minute_precision_offset) => Ok(TimeZone::UtcOffset(
                UtcOffset::from_ixdtf_minute_record(minute_precision_offset),
            )),
            _ => Err(TemporalError::range().with_message("Invalid TimeZone Identifier")),
        })?
    }

    #[cfg(feature = "compiled_data")]
    pub fn try_from_identifier_str(src: &str) -> TemporalResult<Self> {
        Self::try_from_identifier_str_with_provider(src, &*crate::builtins::TZ_PROVIDER)
    }
    /// Parse a `TimeZone` from a `&str`
    ///
    /// This is the equivalent to [`ParseTemporalTimeZoneString`](https://tc39.es/proposal-temporal/#sec-temporal-parsetemporaltimezonestring)
    pub fn try_from_str_with_provider(
        src: &str,
        provider: &impl TimeZoneProvider,
    ) -> TemporalResult<Self> {
        if let Ok(timezone) = Self::try_from_identifier_str_with_provider(src, provider) {
            return Ok(timezone);
        }
        parse_allowed_timezone_formats(src, provider)
            .ok_or_else(|| TemporalError::range().with_message("Not a valid time zone string"))
    }

    #[cfg(feature = "compiled_data")]
    pub fn try_from_str(src: &str) -> TemporalResult<Self> {
        Self::try_from_str_with_provider(src, &*crate::builtins::TZ_PROVIDER)
    }

    /// Returns the current `TimeZoneSlot`'s identifier.
    pub fn identifier(&self) -> String {
        match self {
            TimeZone::IanaIdentifier(s) => s.clone(),
            TimeZone::UtcOffset(offset) => offset.to_string(),
        }
    }

    /// Get the primary identifier for this timezone
    pub fn primary_identifier_with_provider(
        &self,
        provider: &impl TimeZoneProvider,
    ) -> TemporalResult<Self> {
        Ok(match self {
            TimeZone::IanaIdentifier(s) => {
                TimeZone::IanaIdentifier(provider.canonicalize_identifier(s.as_bytes())?.into())
            }
            TimeZone::UtcOffset(offset) => TimeZone::UtcOffset(*offset),
        })
    }

    /// Get the primary identifier for this timezone
    #[cfg(feature = "compiled_data")]
    pub fn primary_identifier(&self) -> TemporalResult<Self> {
        self.primary_identifier_with_provider(&*crate::builtins::TZ_PROVIDER)
    }

    // TimeZoneEquals, which compares primary identifiers
    pub(crate) fn time_zone_equals_with_provider(
        &self,
        other: &Self,
        provider: &impl TimeZoneProvider,
    ) -> TemporalResult<bool> {
        Ok(match (self, other) {
            (TimeZone::IanaIdentifier(one), TimeZone::IanaIdentifier(two)) => {
                let one = provider.canonicalize_identifier(one.as_bytes())?;
                let two = provider.canonicalize_identifier(two.as_bytes())?;
                one == two
            }
            (&TimeZone::UtcOffset(one), &TimeZone::UtcOffset(two)) => one == two,
            _ => false,
        })
    }
}

impl Default for TimeZone {
    fn default() -> Self {
        Self::IanaIdentifier("UTC".into())
    }
}

impl From<&ZonedDateTime> for TimeZone {
    fn from(value: &ZonedDateTime) -> Self {
        value.timezone().clone()
    }
}

impl TimeZone {
    pub(crate) fn get_iso_datetime_for(
        &self,
        instant: &Instant,
        provider: &impl TimeZoneProvider,
    ) -> TemporalResult<IsoDateTime> {
        // 1. Let offsetNanoseconds be GetOffsetNanosecondsFor(timeZone, epochNs).
        let nanos = self.get_offset_nanos_for(instant.as_i128(), provider)?;
        // 2. Let result be GetISOPartsFromEpoch(ℝ(epochNs)).
        // 3. Return BalanceISODateTime(result.[[ISODate]].[[Year]], result.[[ISODate]].[[Month]], result.[[ISODate]].[[Day]],
        // result.[[Time]].[[Hour]], result.[[Time]].[[Minute]], result.[[Time]].[[Second]], result.[[Time]].[[Millisecond]],
        // result.[[Time]].[[Microsecond]], result.[[Time]].[[Nanosecond]] + offsetNanoseconds).
        IsoDateTime::from_epoch_nanos(instant.epoch_nanoseconds(), nanos.to_i64().unwrap_or(0))
    }

    /// Get the offset for this current `TimeZoneSlot`.
    pub(crate) fn get_offset_nanos_for(
        &self,
        utc_epoch: i128,
        provider: &impl TimeZoneProvider,
    ) -> TemporalResult<i128> {
        // 1. Let parseResult be ! ParseTimeZoneIdentifier(timeZone).
        match self {
            // 2. If parseResult.[[OffsetMinutes]] is not empty, return parseResult.[[OffsetMinutes]] × (60 × 10**9).
            Self::UtcOffset(offset) => Ok(i128::from(offset.nanoseconds())),
            // 3. Return GetNamedTimeZoneOffsetNanoseconds(parseResult.[[Name]], epochNs).
            Self::IanaIdentifier(identifier) => provider
                .get_named_tz_offset_nanoseconds(identifier, utc_epoch)
                .map(|transition| i128::from(transition.offset.0) * 1_000_000_000),
        }
    }

    /// Get the offset for this current `TimeZoneSlot` as a `UtcOffset`
    pub(crate) fn get_utcoffset_for(
        &self,
        utc_epoch: i128,
        provider: &impl TimeZoneProvider,
    ) -> TemporalResult<UtcOffset> {
        let offset = self.get_offset_nanos_for(utc_epoch, provider)?;
        let offset = i64::try_from(offset).ok().temporal_unwrap()?;
        Ok(UtcOffset(offset))
    }

    pub(crate) fn get_epoch_nanoseconds_for(
        &self,
        local_iso: IsoDateTime,
        disambiguation: Disambiguation,
        provider: &impl TimeZoneProvider,
    ) -> TemporalResult<EpochNanosecondsAndOffset> {
        // 1. Let possibleEpochNs be ? GetPossibleEpochNanoseconds(timeZone, isoDateTime).
        let possible_nanos = self.get_possible_epoch_ns_for(local_iso, provider)?;
        // 2. Return ? DisambiguatePossibleEpochNanoseconds(possibleEpochNs, timeZone, isoDateTime, disambiguation).
        self.disambiguate_possible_epoch_nanos(possible_nanos, local_iso, disambiguation, provider)
    }

    /// Get the possible `Instant`s for this `TimeZoneSlot`.
    pub(crate) fn get_possible_epoch_ns_for(
        &self,
        local_iso: IsoDateTime,
        provider: &impl TimeZoneProvider,
    ) -> TemporalResult<CandidateEpochNanoseconds> {
        // 1.Let parseResult be ! ParseTimeZoneIdentifier(timeZone).
        let possible_nanoseconds = match self {
            // 2. If parseResult.[[OffsetMinutes]] is not empty, then
            Self::UtcOffset(offset) => {
                // This routine should not be hit with sub-minute offsets
                //
                // > ...takes arguments timeZone (an available time zone identifier)
                // >
                // > An available time zone identifier is either an available named time zone identifier or an
                // > offset time zone identifier.
                // >
                // > Offset time zone identifiers are compared using the number of minutes represented (not as a String),
                // > and are accepted as input in any the formats specified by UTCOffset[~SubMinutePrecision]
                debug_assert!(
                    !offset.is_sub_minute(),
                    "Called get_possible_epoch_ns_for on a sub-minute-precision offset"
                );
                // a. Let balanced be
                // BalanceISODateTime(isoDateTime.[[ISODate]].[[Year]],
                // isoDateTime.[[ISODate]].[[Month]],
                // isoDateTime.[[ISODate]].[[Day]],
                // isoDateTime.[[Time]].[[Hour]],
                // isoDateTime.[[Time]].[[Minute]] -
                // parseResult.[[OffsetMinutes]],
                // isoDateTime.[[Time]].[[Second]],
                // isoDateTime.[[Time]].[[Millisecond]],
                // isoDateTime.[[Time]].[[Microsecond]],
                // isoDateTime.[[Time]].[[Nanosecond]]).
                let balanced = IsoDateTime::balance(
                    local_iso.date.year,
                    local_iso.date.month.into(),
                    local_iso.date.day.into(),
                    local_iso.time.hour.into(),
                    (i16::from(local_iso.time.minute) - offset.minutes()).into(),
                    local_iso.time.second.into(),
                    local_iso.time.millisecond.into(),
                    local_iso.time.microsecond.into(),
                    local_iso.time.nanosecond.into(),
                );
                // b. Perform ? CheckISODaysRange(balanced.[[ISODate]]).
                balanced.date.is_valid_day_range()?;
                // c. Let epochNanoseconds be GetUTCEpochNanoseconds(balanced).
                let epoch_ns = balanced.as_nanoseconds();
                // d. Let possibleEpochNanoseconds be « epochNanoseconds ».
                CandidateEpochNanoseconds::One(EpochNanosecondsAndOffset {
                    offset: *offset,
                    ns: epoch_ns,
                })
            }
            // 3. Else,
            Self::IanaIdentifier(identifier) => {
                // a. Perform ? CheckISODaysRange(isoDateTime.[[ISODate]]).
                local_iso.date.is_valid_day_range()?;
                // b. Let possibleEpochNanoseconds be
                // GetNamedTimeZoneEpochNanoseconds(parseResult.[[Name]],
                // isoDateTime).
                provider.get_named_tz_epoch_nanoseconds(identifier, local_iso)?
            }
        };
        // 4. For each value epochNanoseconds in possibleEpochNanoseconds, do
        // a . If IsValidEpochNanoseconds(epochNanoseconds) is false, throw a RangeError exception.
        for ns in possible_nanoseconds.as_slice() {
            ns.ns.check_validity()?;
        }
        // 5. Return possibleEpochNanoseconds.
        Ok(possible_nanoseconds)
    }
}

impl TimeZone {
    // TODO: This can be optimized by just not using a vec.
    pub(crate) fn disambiguate_possible_epoch_nanos(
        &self,
        nanos: CandidateEpochNanoseconds,
        iso: IsoDateTime,
        disambiguation: Disambiguation,
        provider: &impl TimeZoneProvider,
    ) -> TemporalResult<EpochNanosecondsAndOffset> {
        // 1. Let n be possibleEpochNs's length.
        let valid_bounds = match nanos {
            // 2. If n = 1, then
            CandidateEpochNanoseconds::One(ns) => {
                // a. Return possibleEpochNs[0].
                return Ok(ns);
            }
            // 3. If n ≠ 0, then
            CandidateEpochNanoseconds::Two([one, two]) => {
                match disambiguation {
                    // a. If disambiguation is earlier or compatible, then
                    // i. Return possibleEpochNs[0].
                    Disambiguation::Compatible | Disambiguation::Earlier => return Ok(one),
                    // b. If disambiguation is later, then
                    // i. Return possibleEpochNs[n - 1].
                    Disambiguation::Later => return Ok(two),
                    // c. Assert: disambiguation is reject.
                    // d. Throw a RangeError exception.
                    Disambiguation::Reject => {
                        return Err(
                            TemporalError::range().with_message("Rejecting ambiguous time zones.")
                        )
                    }
                }
            }
            CandidateEpochNanoseconds::Zero(vb) => vb,
        };

        // 4. Assert: n = 0.
        // 5. If disambiguation is reject, then
        if disambiguation == Disambiguation::Reject {
            // a. Throw a RangeError exception.
            return Err(TemporalError::range().with_message("Rejecting ambiguous time zones."));
        }

        // Instead of calculating the latest/earliest possible ISO datetime record,
        // the GapEntryOffsets from CandidateEpochNanoseconds::Zero already has
        // the offsets before and after the gap transition. We can use that directly,
        // instead of doing a bunch of additional work.
        //
        // 6. Let before be the latest possible ISO Date-Time Record for
        //    which CompareISODateTime(before, isoDateTime) = -1 and !
        //    GetPossibleEpochNanoseconds(timeZone, before) is not
        //    empty.
        // 7. Let after be the earliest possible ISO Date-Time Record
        //    for which CompareISODateTime(after, isoDateTime) = 1 and !
        // 8. Let beforePossible be !
        //    GetPossibleEpochNanoseconds(timeZone, before).
        // 9. Assert: beforePossible's length is 1.
        // 10. Let afterPossible be !
        //     GetPossibleEpochNanoseconds(timeZone, after).
        // 11. Assert: afterPossible's length is 1.

        // 12. Let offsetBefore be GetOffsetNanosecondsFor(timeZone,
        //     beforePossible[0]).
        let offset_before = valid_bounds.offset_before;
        // 13. Let offsetAfter be GetOffsetNanosecondsFor(timeZone,
        //     afterPossible[0]).
        let offset_after = valid_bounds.offset_after;
        // 14. Let nanoseconds be offsetAfter - offsetBefore.
        let seconds = offset_after.0 - offset_before.0;
        let nanoseconds = seconds as i128 * 1_000_000_000;
        // 15. Assert: abs(nanoseconds) ≤ nsPerDay.
        // 16. If disambiguation is earlier, then
        if disambiguation == Disambiguation::Earlier {
            // a. Let timeDuration be TimeDurationFromComponents(0, 0, 0, 0, 0, -nanoseconds).
            let time_duration = TimeDuration(-nanoseconds);
            // b. Let earlierTime be AddTime(isoDateTime.[[Time]], timeDuration).
            let earlier_time = iso.time.add(time_duration);
            // c. Let earlierDate be BalanceISODate(isoDateTime.[[ISODate]].[[Year]],
            // isoDateTime.[[ISODate]].[[Month]],
            // isoDateTime.[[ISODate]].[[Day]] + earlierTime.[[Days]]).
            let earlier_date = IsoDate::try_balance(
                iso.date.year,
                iso.date.month.into(),
                i64::from(iso.date.day) + earlier_time.0,
            )?;

            // d. Let earlierDateTime be
            // CombineISODateAndTimeRecord(earlierDate, earlierTime).
            let earlier = IsoDateTime::new_unchecked(earlier_date, earlier_time.1);
            // e. Set possibleEpochNs to ? GetPossibleEpochNanoseconds(timeZone, earlierDateTime).
            let possible = self.get_possible_epoch_ns_for(earlier, provider)?;
            // f. Assert: possibleEpochNs is not empty.
            // g. Return possibleEpochNs[0].
            return possible.first().temporal_unwrap();
        }
        // 17. Assert: disambiguation is compatible or later.
        // 18. Let timeDuration be TimeDurationFromComponents(0, 0, 0, 0, 0, nanoseconds).
        let time_duration = TimeDuration(nanoseconds);
        // 19. Let laterTime be AddTime(isoDateTime.[[Time]], timeDuration).
        let later_time = iso.time.add(time_duration);
        // 20. Let laterDate be BalanceISODate(isoDateTime.[[ISODate]].[[Year]],
        // isoDateTime.[[ISODate]].[[Month]], isoDateTime.[[ISODate]].[[Day]] + laterTime.[[Days]]).
        let later_date = IsoDate::try_balance(
            iso.date.year,
            iso.date.month.into(),
            i64::from(iso.date.day) + later_time.0,
        )?;
        // 21. Let laterDateTime be CombineISODateAndTimeRecord(laterDate, laterTime).
        let later = IsoDateTime::new_unchecked(later_date, later_time.1);
        // 22. Set possibleEpochNs to ? GetPossibleEpochNanoseconds(timeZone, laterDateTime).
        let possible = self.get_possible_epoch_ns_for(later, provider)?;
        // 23. Set n to possibleEpochNs's length.
        // 24. Assert: n ≠ 0.
        // 25. Return possibleEpochNs[n - 1].
        possible.last().temporal_unwrap()
    }

    pub(crate) fn get_start_of_day(
        &self,
        iso_date: &IsoDate,
        provider: &impl TimeZoneProvider,
    ) -> TemporalResult<EpochNanosecondsAndOffset> {
        // 1. Let isoDateTime be CombineISODateAndTimeRecord(isoDate, MidnightTimeRecord()).
        let iso = IsoDateTime::new_unchecked(*iso_date, IsoTime::default());
        // 2. Let possibleEpochNs be ? GetPossibleEpochNanoseconds(timeZone, isoDateTime).
        let possible_nanos = self.get_possible_epoch_ns_for(iso, provider)?;
        // 3. If possibleEpochNs is not empty, return possibleEpochNs[0].
        let gap = match possible_nanos {
            CandidateEpochNanoseconds::One(first) | CandidateEpochNanoseconds::Two([first, _]) => {
                return Ok(first)
            }
            CandidateEpochNanoseconds::Zero(gap) => gap,
        };
        // 5. Let possibleEpochNsAfter be GetNamedTimeZoneEpochNanoseconds(timeZone, isoDateTimeAfter), where
        // isoDateTimeAfter is the ISO Date-Time Record for which ! DifferenceISODateTime(isoDateTime,
        // isoDateTimeAfter, "iso8601", hour).[[Time]] is the smallest possible value > 0 for which
        // possibleEpochNsAfter is not empty (i.e., isoDateTimeAfter represents the first local time
        // after the transition).

        // For a gap entry, possibleEpochNsAfter will just be the transition time. We don't
        // actually need to calculate an isoDateTimeAfter and do all that rigmarole; we already
        // know the transition time.

        // 6. Assert: possibleEpochNsAfter's length = 1.
        // 7. Return possibleEpochNsAfter[0].

        Ok(EpochNanosecondsAndOffset {
            offset: gap.offset_after.into(),
            ns: gap.transition_epoch,
        })
    }
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "compiled_data")]
    use super::TimeZone;

    #[test]
    #[cfg(feature = "compiled_data")]
    fn from_and_to_string() {
        let src = "+09:30";
        let tz = TimeZone::try_from_identifier_str(src).unwrap();
        assert_eq!(tz.identifier(), src);

        let src = "-09:30";
        let tz = TimeZone::try_from_identifier_str(src).unwrap();
        assert_eq!(tz.identifier(), src);

        let src = "-12:30";
        let tz = TimeZone::try_from_identifier_str(src).unwrap();
        assert_eq!(tz.identifier(), src);

        let src = "America/New_York";
        let tz = TimeZone::try_from_identifier_str(src).unwrap();
        assert_eq!(tz.identifier(), src);
    }

    #[test]
    #[cfg(feature = "compiled_data")]
    fn canonicalize_equals() {
        let calcutta = TimeZone::try_from_identifier_str("Asia/Calcutta").unwrap();
        let kolkata = TimeZone::try_from_identifier_str("Asia/Kolkata").unwrap();
        assert!(calcutta
            .time_zone_equals_with_provider(&kolkata, &*crate::builtins::TZ_PROVIDER)
            .unwrap());
    }
}
