//! The `TimeZoneProvider` trait.

use core::str::FromStr;

// use crate::UtcOffset;
use crate::utils;
use crate::{epoch_nanoseconds::EpochNanoseconds, TimeZoneProviderError};
use alloc::borrow::Cow;

pub(crate) type TimeZoneProviderResult<T> = Result<T, TimeZoneProviderError>;

/// `UtcOffsetSeconds` represents the amount of seconds we need to add to the UTC to reach the local time.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct UtcOffsetSeconds(pub i64);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IsoDateTime {
    pub year: i32,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
    pub millisecond: u16,
    pub microsecond: u16,
    pub nanosecond: u16,
}

impl IsoDateTime {
    fn to_epoch_days(self) -> i32 {
        // NOTE: cast to i32 is safe as IsoDate is in a valid range.
        utils::epoch_days_from_gregorian_date(self.year, self.month, self.day) as i32
    }
    /// `IsoTimeToEpochMs`
    ///
    /// Note: This method is library specific and not in spec
    ///
    /// Functionally the same as Date's `MakeTime`
    fn time_to_epoch_ms(self) -> i64 {
        ((i64::from(self.hour) * utils::MS_PER_HOUR
            + i64::from(self.minute) * utils::MS_PER_MINUTE)
            + i64::from(self.second) * 1000i64)
            + i64::from(self.millisecond)
    }

    /// Convert this datetime to nanoseconds since the Unix epoch
    pub fn as_nanoseconds(&self) -> EpochNanoseconds {
        let time_ms = self.time_to_epoch_ms();
        let epoch_ms = utils::epoch_days_to_epoch_ms(self.to_epoch_days() as i64, time_ms);
        EpochNanoseconds(
            epoch_ms as i128 * 1_000_000
                + self.microsecond as i128 * 1_000
                + self.nanosecond as i128,
        )
    }
}

#[cfg(feature = "tzif")]
use tzif::data::{posix::TimeZoneVariantInfo, tzif::LocalTimeTypeRecord};

#[cfg(feature = "tzif")]
impl From<&TimeZoneVariantInfo> for UtcOffsetSeconds {
    fn from(value: &TimeZoneVariantInfo) -> Self {
        // The POSIX tz string stores offsets as negative offsets;
        // i.e. "seconds that must be added to reach UTC"
        Self(-value.offset.0)
    }
}

#[cfg(feature = "tzif")]
impl From<LocalTimeTypeRecord> for UtcOffsetSeconds {
    fn from(value: LocalTimeTypeRecord) -> Self {
        Self(value.utoff.0)
    }
}

/// An EpochNanoseconds and a UTC offset
#[derive(Copy, Clone, Debug)]
pub struct EpochNanosecondsAndOffset {
    /// The resolved nanoseconds value
    pub ns: EpochNanoseconds,
    /// The resolved time zone offset corresponding
    /// to the nanoseconds value, in the given time zone
    pub offset: UtcOffsetSeconds,
}

/// `TimeZoneTransitionInfo` represents information about a timezone transition.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimeZoneTransitionInfo {
    /// The transition time epoch at which the offset needs to be applied.
    pub transition_epoch: Option<i64>,
    /// The time zone offset in seconds.
    pub offset: UtcOffsetSeconds,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TransitionDirection {
    Next,
    Previous,
}

#[derive(Debug, Clone, Copy)]
pub struct ParseDirectionError;

impl core::fmt::Display for ParseDirectionError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("provided string was not a valid direction.")
    }
}

impl FromStr for TransitionDirection {
    type Err = ParseDirectionError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "next" => Ok(Self::Next),
            "previous" => Ok(Self::Previous),
            _ => Err(ParseDirectionError),
        }
    }
}

impl core::fmt::Display for TransitionDirection {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Next => "next",
            Self::Previous => "previous",
        }
        .fmt(f)
    }
}

/// Used in disambiguate_possible_epoch_nanos
///
/// When we have a LocalTimeRecordResult::Empty,
/// it is useful to know the offsets before and after.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct GapEntryOffsets {
    pub offset_before: UtcOffsetSeconds,
    pub offset_after: UtcOffsetSeconds,
    pub transition_epoch: EpochNanoseconds,
}

/// The potential candidates for a given local datetime
#[derive(Copy, Clone, Debug)]
pub enum CandidateEpochNanoseconds {
    Zero(GapEntryOffsets),
    One(EpochNanosecondsAndOffset),
    Two([EpochNanosecondsAndOffset; 2]),
}

impl CandidateEpochNanoseconds {
    pub fn as_slice(&self) -> &[EpochNanosecondsAndOffset] {
        match *self {
            Self::Zero(..) => &[],
            Self::One(ref one) => core::slice::from_ref(one),
            Self::Two(ref multiple) => &multiple[..],
        }
    }

    #[allow(unused)] // Used in tests in some feature configurations
    pub fn is_empty(&self) -> bool {
        matches!(*self, Self::Zero(..))
    }

    #[allow(unused)] // Used in tests in some feature configurations
    pub fn len(&self) -> usize {
        match *self {
            Self::Zero(..) => 0,
            Self::One(..) => 1,
            Self::Two(..) => 2,
        }
    }

    pub fn first(&self) -> Option<EpochNanosecondsAndOffset> {
        match *self {
            Self::Zero(..) => None,
            Self::One(one) | Self::Two([one, _]) => Some(one),
        }
    }

    pub fn last(&self) -> Option<EpochNanosecondsAndOffset> {
        match *self {
            Self::Zero(..) => None,
            Self::One(last) | Self::Two([_, last]) => Some(last),
        }
    }
}

// NOTE: It may be a good idea to eventually move this into it's
// own individual crate rather than having it tied directly into `temporal_rs`
/// The `TimeZoneProvider` trait provides methods required for a provider
/// to implement in order to source time zone data from that provider.
pub trait TimeZoneProvider {
    fn normalize_identifier(&self, ident: &'_ [u8]) -> Result<Cow<'_, str>, TimeZoneProviderError>;

    fn canonicalize_identifier(
        &self,
        ident: &'_ [u8],
    ) -> Result<Cow<'_, str>, TimeZoneProviderError>;

    fn get_named_tz_epoch_nanoseconds(
        &self,
        identifier: &str,
        local_datetime: IsoDateTime,
    ) -> Result<CandidateEpochNanoseconds, TimeZoneProviderError>;

    fn get_named_tz_offset_nanoseconds(
        &self,
        identifier: &str,
        epoch_nanoseconds: i128,
    ) -> Result<TimeZoneTransitionInfo, TimeZoneProviderError>;

    fn get_named_tz_transition(
        &self,
        identifier: &str,
        epoch_nanoseconds: i128,
        direction: TransitionDirection,
    ) -> Result<Option<EpochNanoseconds>, TimeZoneProviderError>;
}

pub struct NeverProvider;

impl TimeZoneProvider for NeverProvider {
    fn normalize_identifier(
        &self,
        _ident: &'_ [u8],
    ) -> Result<Cow<'_, str>, TimeZoneProviderError> {
        unimplemented!()
    }
    fn canonicalize_identifier(
        &self,
        _ident: &'_ [u8],
    ) -> Result<Cow<'_, str>, TimeZoneProviderError> {
        unimplemented!()
    }
    fn get_named_tz_epoch_nanoseconds(
        &self,
        _: &str,
        _: IsoDateTime,
    ) -> Result<CandidateEpochNanoseconds, TimeZoneProviderError> {
        unimplemented!()
    }

    fn get_named_tz_offset_nanoseconds(
        &self,
        _: &str,
        _: i128,
    ) -> Result<TimeZoneTransitionInfo, TimeZoneProviderError> {
        unimplemented!()
    }

    fn get_named_tz_transition(
        &self,
        _: &str,
        _: i128,
        _: TransitionDirection,
    ) -> Result<Option<EpochNanoseconds>, TimeZoneProviderError> {
        unimplemented!()
    }
}
