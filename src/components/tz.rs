//! This module implements the Temporal `TimeZone` and components.

use alloc::borrow::ToOwned;
use alloc::string::String;
use alloc::vec::Vec;
use core::{iter::Peekable, str::Chars};

use num_traits::ToPrimitive;

use crate::{components::Instant, iso::IsoDateTime, TemporalError, TemporalResult};

#[cfg(all(feature = "tzdb", not(target_os = "windows")))]
use crate::tzdb::FsTzdbProvider;
#[cfg(feature = "experimental")]
use std::sync::{LazyLock, Mutex};

#[cfg(feature = "experimental")]
pub static TZ_PROVIDER: LazyLock<Mutex<FsTzdbProvider>> =
    LazyLock::new(|| Mutex::new(FsTzdbProvider::default()));

use super::ZonedDateTime;

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
    pub fn from_str(s: &'a str, provider: &mut impl TzProvider) -> TemporalResult<Self> {
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
        provider: &mut impl TzProvider,
    ) -> TemporalResult<IsoDateTime> {
        let nanos = self.get_offset_nanos_for(instant.epoch_nanos, provider)?;
        IsoDateTime::from_epoch_nanos(&instant.epoch_nanos, nanos.to_f64().unwrap_or(0.0))
    }
}

impl TimeZone {
    /// Get the offset for this current `TimeZoneSlot`.
    pub fn get_offset_nanos_for(
        &self,
        epoch_ns: i128,
        provider: &mut impl TzProvider,
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

    /// Get the possible `Instant`s for this `TimeZoneSlot`.
    pub fn get_possible_instant_for(&self) -> TemporalResult<Vec<Instant>> {
        Err(TemporalError::general("Not yet implemented."))
    }

    /// Returns the current `TimeZoneSlot`'s identifier.
    pub fn id(&self) -> TemporalResult<String> {
        Err(TemporalError::range().with_message("Not yet implemented."))
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
