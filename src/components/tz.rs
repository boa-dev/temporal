//! This module implements the Temporal `TimeZone` and components.

use alloc::borrow::ToOwned;
use alloc::string::String;
use alloc::{vec, vec::Vec};
use core::{iter::Peekable, str::Chars};

use num_traits::ToPrimitive;

use crate::components::instant::EpochNanoseconds;
use crate::{
    components::{duration::normalized::NormalizedTimeDuration, Instant},
    iso::{IsoDate, IsoDateTime},
    options::Disambiguation,
    TemporalError, TemporalResult,
};

#[cfg(feature = "experimental")]
use crate::tzdb::FsTzdbProvider;
#[cfg(feature = "experimental")]
use std::sync::{LazyLock, Mutex};

#[cfg(feature = "experimental")]
pub static TZ_PROVIDER: LazyLock<Mutex<FsTzdbProvider>> =
    LazyLock::new(|| Mutex::new(FsTzdbProvider::default()));

use super::{instant::is_valid_epoch_nanos, ZonedDateTime};

pub trait TzProvider {
    fn check_identifier(&self, identifier: &str) -> bool;

    fn get_named_tz_epoch_nanoseconds(
        &self,
        identifier: &str,
        iso_datetime: IsoDateTime,
    ) -> TemporalResult<Vec<i128>>;

    fn get_named_tz_offset_nanoseconds(
        &self,
        identifier: &str,
        epoch_nanoseconds: i128,
    ) -> TemporalResult<i128>;
}

/// A Temporal `TimeZone`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ParsedTimeZone<'a> {
    IanaIdentifier { identifier: &'a str },
    Offset { minutes: i16 },
}

impl<'a> ParsedTimeZone<'a> {
    pub fn from_str(s: &'a str, provider: &impl TzProvider) -> TemporalResult<Self> {
        if s == "Z" {
            return Ok(Self::Offset { minutes: 0 });
        }
        let mut cursor = s.chars().peekable();
        if cursor.peek().map_or(false, is_ascii_sign) {
            return parse_offset(&mut cursor);
        } else if provider.check_identifier(s) {
            return Ok(Self::IanaIdentifier { identifier: s });
        }
        Err(TemporalError::range().with_message("Valid time zone was not provided."))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TimeZone(pub String);

impl Default for TimeZone {
    fn default() -> Self {
        Self("UTC".into())
    }
}

impl From<&ZonedDateTime> for TimeZone {
    fn from(value: &ZonedDateTime) -> Self {
        value.tz().clone()
    }
}

impl From<String> for TimeZone {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<&str> for TimeZone {
    fn from(value: &str) -> Self {
        Self(value.to_owned())
    }
}

impl TimeZone {
    pub(crate) fn get_iso_datetime_for(
        &self,
        instant: &Instant,
        provider: &impl TzProvider,
    ) -> TemporalResult<IsoDateTime> {
        let nanos = self.get_offset_nanos_for(instant.as_i128(), provider)?;
        IsoDateTime::from_epoch_nanos(&instant.as_i128(), nanos.to_f64().unwrap_or(0.0))
    }
}

impl TimeZone {
    /// Get the offset for this current `TimeZoneSlot`.
    pub fn get_offset_nanos_for(
        &self,
        epoch_ns: i128,
        provider: &impl TzProvider,
    ) -> TemporalResult<i128> {
        // 1. Let parseResult be ! ParseTimeZoneIdentifier(timeZone).
        let parsed = ParsedTimeZone::from_str(&self.0, provider)?;
        match parsed {
            // 2. If parseResult.[[OffsetMinutes]] is not empty, return parseResult.[[OffsetMinutes]] × (60 × 10**9).
            ParsedTimeZone::Offset { minutes } => Ok(i128::from(minutes) * 60_000_000_000i128),
            // 3. Return GetNamedTimeZoneOffsetNanoseconds(parseResult.[[Name]], epochNs).
            ParsedTimeZone::IanaIdentifier { identifier } => {
                provider.get_named_tz_offset_nanoseconds(identifier, epoch_ns)
            }
        }
    }

    pub fn get_epoch_nanoseconds_for(
        &self,
        iso: IsoDateTime,
        disambiguation: Disambiguation,
        provider: &impl TzProvider,
    ) -> TemporalResult<EpochNanoseconds> {
        // 1. Let possibleEpochNs be ? GetPossibleEpochNanoseconds(timeZone, isoDateTime).
        let possible_nanos = self.get_possible_epoch_ns_for(iso, provider)?;
        // 2. Return ? DisambiguatePossibleEpochNanoseconds(possibleEpochNs, timeZone, isoDateTime, disambiguation).
        self.disambiguate_possible_epoch_nanos(possible_nanos, iso, disambiguation, provider)
    }

    /// Get the possible `Instant`s for this `TimeZoneSlot`.
    pub fn get_possible_epoch_ns_for(
        &self,
        iso: IsoDateTime,
        provider: &impl TzProvider,
    ) -> TemporalResult<Vec<i128>> {
        // 1.Let parseResult be ! ParseTimeZoneIdentifier(timeZone).
        let possible_nanoseconds = match ParsedTimeZone::from_str(&self.0, provider)? {
            // 2. If parseResult.[[OffsetMinutes]] is not empty, then
            ParsedTimeZone::Offset { minutes } => {
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
                    iso.date.year,
                    iso.date.month.into(),
                    iso.date.day.into(),
                    iso.time.hour.into(),
                    (i16::from(iso.time.minute) - minutes).into(),
                    iso.time.second.into(),
                    iso.time.millisecond.into(),
                    iso.time.microsecond.into(),
                    iso.time.nanosecond.into(),
                );
                // b. Perform ? CheckISODaysRange(balanced.[[ISODate]]).
                balanced.date.is_valid_day_range()?;
                // c. Let epochNanoseconds be GetUTCEpochNanoseconds(balanced).
                let epoch_ns = balanced
                    .as_nanoseconds()
                    .expect("conversion should be in a valid range. Option is result of BigInt");
                // d. Let possibleEpochNanoseconds be « epochNanoseconds ».
                vec![epoch_ns]
            }
            // 3. Else,
            ParsedTimeZone::IanaIdentifier { identifier } => {
                // a. Perform ? CheckISODaysRange(isoDateTime.[[ISODate]]).
                iso.date.is_valid_day_range()?;
                // b. Let possibleEpochNanoseconds be
                // GetNamedTimeZoneEpochNanoseconds(parseResult.[[Name]],
                // isoDateTime).
                provider.get_named_tz_epoch_nanoseconds(identifier, iso)?
            }
        };
        // 4. For each value epochNanoseconds in possibleEpochNanoseconds, do
        for ns in &possible_nanoseconds {
            // a . If IsValidEpochNanoseconds(epochNanoseconds) is false, throw a RangeError exception.
            if !is_valid_epoch_nanos(ns) {
                return Err(TemporalError::range()
                    .with_message("A possible nanosecond exceeded valid range."));
            }
        }
        // 5. Return possibleEpochNanoseconds.
        Ok(possible_nanoseconds)
    }

    /// Returns the current `TimeZoneSlot`'s identifier.
    pub fn id(&self) -> TemporalResult<String> {
        Err(TemporalError::range().with_message("Not yet implemented."))
    }
}

impl TimeZone {
    // TODO: This can be optimized by just not using a vec.
    pub(crate) fn disambiguate_possible_epoch_nanos(
        &self,
        nanos: Vec<i128>,
        iso: IsoDateTime,
        disambiguation: Disambiguation,
        provider: &impl TzProvider,
    ) -> TemporalResult<EpochNanoseconds> {
        // 1. Let n be possibleEpochNs's length.
        let n = nanos.len();
        // 2. If n = 1, then
        if n == 1 {
            // a. Return possibleEpochNs[0].
            return EpochNanoseconds::try_from(nanos[0]);
        // 3. If n ≠ 0, then
        } else if n != 0 {
            match disambiguation {
                // a. If disambiguation is earlier or compatible, then
                // i. Return possibleEpochNs[0].
                Disambiguation::Compatible | Disambiguation::Earlier => {
                    return EpochNanoseconds::try_from(nanos[0])
                }
                // b. If disambiguation is later, then
                // i. Return possibleEpochNs[n - 1].
                Disambiguation::Later => return EpochNanoseconds::try_from(nanos[n - 1]),
                // c. Assert: disambiguation is reject.
                // d. Throw a RangeError exception.
                Disambiguation::Reject => {
                    return Err(
                        TemporalError::range().with_message("Rejecting ambiguous time zones.")
                    )
                }
            }
        }
        // 4. Assert: n = 0.
        // 5. If disambiguation is reject, then
        if disambiguation == Disambiguation::Reject {
            // a. Throw a RangeError exception.
            return Err(TemporalError::range().with_message("Rejecting ambiguous time zones."));
        }

        // NOTE: Below is rather greedy, but should in theory work.
        //
        // Primarily moving hour +/-3 to account Australia/Troll as
        // the precision of before/after does not entirely matter as
        // long is it is distinctly before / after any transition.

        // 6. Let before be the latest possible ISO Date-Time Record for
        //    which CompareISODateTime(before, isoDateTime) = -1 and !
        //    GetPossibleEpochNanoseconds(timeZone, before) is not
        //    empty.
        let mut before = iso;
        before.time.hour -= 3;
        // 7. Let after be the earliest possible ISO Date-Time Record
        //    for which CompareISODateTime(after, isoDateTime) = 1 and !
        //    GetPossibleEpochNanoseconds(timeZone, after) is not empty.
        let mut after = iso;
        after.time.hour += 3;

        // 8. Let beforePossible be !
        //    GetPossibleEpochNanoseconds(timeZone, before).
        // 9. Assert: beforePossible's length is 1.
        let before_possible = self.get_possible_epoch_ns_for(before, provider)?;
        debug_assert_eq!(before_possible.len(), 1);
        // 10. Let afterPossible be !
        //     GetPossibleEpochNanoseconds(timeZone, after).
        // 11. Assert: afterPossible's length is 1.
        let after_possible = self.get_possible_epoch_ns_for(after, provider)?;
        debug_assert_eq!(after_possible.len(), 1);
        // 12. Let offsetBefore be GetOffsetNanosecondsFor(timeZone,
        //     beforePossible[0]).
        let offset_before = self.get_offset_nanos_for(before_possible[0], provider)?;
        // 13. Let offsetAfter be GetOffsetNanosecondsFor(timeZone,
        //     afterPossible[0]).
        let offset_after = self.get_offset_nanos_for(after_possible[0], provider)?;
        // 14. Let nanoseconds be offsetAfter - offsetBefore.
        let nanoseconds = offset_after - offset_before;
        // 15. Assert: abs(nanoseconds) ≤ nsPerDay.
        // 16. If disambiguation is earlier, then
        if disambiguation == Disambiguation::Earlier {
            // a. Let timeDuration be TimeDurationFromComponents(0, 0, 0, 0, 0, -nanoseconds).
            let time_duration = NormalizedTimeDuration(-nanoseconds);
            // b. Let earlierTime be AddTime(isoDateTime.[[Time]], timeDuration).
            let earlier_time = iso.time.add(time_duration);
            // c. Let earlierDate be BalanceISODate(isoDateTime.[[ISODate]].[[Year]],
            // isoDateTime.[[ISODate]].[[Month]],
            // isoDateTime.[[ISODate]].[[Day]] + earlierTime.[[Days]]).
            let earlier_date = IsoDate::balance(
                iso.date.year,
                iso.date.month.into(),
                i32::from(iso.date.day) + earlier_time.0,
            );

            // d. Let earlierDateTime be
            // CombineISODateAndTimeRecord(earlierDate, earlierTime).
            let earlier = IsoDateTime::new_unchecked(earlier_date, earlier_time.1);
            // e. Set possibleEpochNs to ? GetPossibleEpochNanoseconds(timeZone, earlierDateTime).
            let possible = self.get_possible_epoch_ns_for(earlier, provider)?;
            // f. Assert: possibleEpochNs is not empty.
            // g. Return possibleEpochNs[0].
            return EpochNanoseconds::try_from(possible[0]);
        }
        // 17. Assert: disambiguation is compatible or later.
        // 18. Let timeDuration be TimeDurationFromComponents(0, 0, 0, 0, 0, nanoseconds).
        let time_duration = NormalizedTimeDuration(nanoseconds);
        // 19. Let laterTime be AddTime(isoDateTime.[[Time]], timeDuration).
        let later_time = iso.time.add(time_duration);
        // 20. Let laterDate be BalanceISODate(isoDateTime.[[ISODate]].[[Year]],
        // isoDateTime.[[ISODate]].[[Month]], isoDateTime.[[ISODate]].[[Day]] + laterTime.[[Days]]).
        let later_date = IsoDate::balance(
            iso.date.year,
            iso.date.month.into(),
            i32::from(iso.date.day) + later_time.0,
        );
        // 21. Let laterDateTime be CombineISODateAndTimeRecord(laterDate, laterTime).
        let later = IsoDateTime::new_unchecked(later_date, later_time.1);
        // 22. Set possibleEpochNs to ? GetPossibleEpochNanoseconds(timeZone, laterDateTime).
        let possible = self.get_possible_epoch_ns_for(later, provider)?;
        // 23. Set n to possibleEpochNs's length.
        let n = possible.len();
        // 24. Assert: n ≠ 0.
        // 25. Return possibleEpochNs[n - 1].
        EpochNanoseconds::try_from(possible[n - 1])
    }
}

#[inline]
fn parse_offset<'a>(chars: &mut Peekable<Chars<'_>>) -> TemporalResult<ParsedTimeZone<'a>> {
    let sign = chars.next().map_or(1, |c| if c == '+' { 1 } else { -1 });
    // First offset portion
    let hours = parse_digit_pair(chars)?;

    let sep = chars.peek().map_or(false, |ch| *ch == ':');
    if sep {
        let _ = chars.next();
    }

    let digit_peek = chars.peek().map(|ch| ch.is_ascii_digit());

    let minutes = match digit_peek {
        Some(true) => parse_digit_pair(chars)?,
        Some(false) => return Err(non_ascii_digit()),
        None => 0,
    };

    Ok(ParsedTimeZone::Offset {
        minutes: (hours * 60 + minutes) * sign,
    })
}

fn parse_digit_pair(chars: &mut Peekable<Chars<'_>>) -> TemporalResult<i16> {
    let valid = chars
        .peek()
        .map_or(Err(abrupt_end()), |ch| Ok(ch.is_ascii_digit()))?;
    let first = if valid {
        chars.next().expect("validated.")
    } else {
        return Err(non_ascii_digit());
    };
    let valid = chars
        .peek()
        .map_or(Err(abrupt_end()), |ch| Ok(ch.is_ascii_digit()))?;
    let second = if valid {
        chars.next().expect("validated.")
    } else {
        return Err(non_ascii_digit());
    };

    let tens = (first.to_digit(10).expect("validated") * 10) as i16;
    let ones = second.to_digit(10).expect("validated") as i16;

    Ok(tens + ones)
}

// NOTE: Spec calls for throwing a RangeError when parse node is a list of errors for timezone.

fn abrupt_end() -> TemporalError {
    TemporalError::range().with_message("Abrupt end while parsing offset string")
}

fn non_ascii_digit() -> TemporalError {
    TemporalError::range().with_message("Non ascii digit found while parsing offset string")
}

fn is_ascii_sign(ch: &char) -> bool {
    *ch == '+' || *ch == '-'
}
