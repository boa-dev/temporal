//! This module implements `ZonedDateTime` and any directly related algorithms.

use alloc::{borrow::ToOwned, string::String};
use ixdtf::parsers::records::TimeZoneRecord;
use tinystr::TinyAsciiStr;

use crate::{
    components::{calendar::CalendarDateLike, tz::TzProvider},
    iso::{IsoDate, IsoDateTime, IsoTime},
    options::{ArithmeticOverflow, Disambiguation, OffsetDisambiguation, TemporalRoundingMode},
    parsers,
    partial::{PartialDate, PartialTime},
    rounding::{IncrementRounder, Round},
    temporal_assert, Calendar, Duration, Instant, PlainDate, PlainDateTime, Sign, TemporalError,
    TemporalResult, TimeZone,
};

/// A struct representing a partial `ZonedDateTime`.
pub struct PartialZonedDateTime {
    /// The `PartialDate` portion of a `PartialDateTime`
    pub date: PartialDate,
    /// The `PartialTime` portion of a `PartialDateTime`
    pub time: PartialTime,
    /// An optional offset string
    pub offset: Option<String>,
    /// The time zone value of a partial time zone.
    pub timezone: TimeZone,
}

#[cfg(feature = "experimental")]
use crate::components::tz::TZ_PROVIDER;
use core::{num::NonZeroU128, str::FromStr};
#[cfg(feature = "experimental")]
use std::ops::Deref;

use super::{tz::parse_offset, EpochNanoseconds};

/// The native Rust implementation of `Temporal.ZonedDateTime`.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ZonedDateTime {
    instant: Instant,
    calendar: Calendar,
    tz: TimeZone,
}

impl Ord for ZonedDateTime {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.instant.cmp(&other.instant)
    }
}

impl PartialOrd for ZonedDateTime {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

// ==== Private API ====

impl ZonedDateTime {
    /// Creates a `ZonedDateTime` without validating the input.
    #[inline]
    #[must_use]
    pub(crate) fn new_unchecked(instant: Instant, calendar: Calendar, tz: TimeZone) -> Self {
        Self {
            instant,
            calendar,
            tz,
        }
    }

    pub(crate) fn add_as_instant(
        &self,
        duration: &Duration,
        overflow: ArithmeticOverflow,
        provider: &impl TzProvider,
    ) -> TemporalResult<Instant> {
        // 1. If DateDurationSign(duration.[[Date]]) = 0, then
        if duration.date().sign() == Sign::Zero {
            // a. Return ? AddInstant(epochNanoseconds, duration.[[Time]]).
            return self.instant.add_to_instant(duration.time());
        }
        // 2. Let isoDateTime be GetISODateTimeFor(timeZone, epochNanoseconds).
        let iso_datetime = self.tz.get_iso_datetime_for(&self.instant, provider)?;
        // 3. Let addedDate be ? CalendarDateAdd(calendar, isoDateTime.[[ISODate]], duration.[[Date]], overflow).
        let added_date = self.calendar().date_add(
            &PlainDate::new_unchecked(iso_datetime.date, self.calendar().clone()),
            duration,
            overflow,
        )?;
        // 4. Let intermediateDateTime be CombineISODateAndTimeRecord(addedDate, isoDateTime.[[Time]]).
        let intermediate = IsoDateTime::new_unchecked(added_date.iso, iso_datetime.time);
        // 5. If ISODateTimeWithinLimits(intermediateDateTime) is false, throw a RangeError exception.
        if !intermediate.is_within_limits() {
            return Err(TemporalError::range()
                .with_message("Intermediate ISO datetime was not within a valid range."));
        }
        // 6. Let intermediateNs be ! GetEpochNanosecondsFor(timeZone, intermediateDateTime, compatible).
        let intermediate_ns = self.timezone().get_epoch_nanoseconds_for(
            intermediate,
            Disambiguation::Compatible,
            provider,
        )?;

        // 7. Return ? AddInstant(intermediateNs, duration.[[Time]]).
        Instant::from(intermediate_ns).add_to_instant(duration.time())
    }

    #[inline]
    /// Adds a duration to the current `ZonedDateTime`, returning the resulting `ZonedDateTime`.
    ///
    /// Aligns with Abstract Operation 6.5.10 and 6.5.5
    pub(crate) fn add_internal(
        &self,
        duration: &Duration,
        overflow: ArithmeticOverflow,
        provider: &impl TzProvider,
    ) -> TemporalResult<Self> {
        // 1. Let duration be ? ToTemporalDuration(temporalDurationLike).
        // 2. If operation is subtract, set duration to CreateNegatedTemporalDuration(duration).
        // 3. Let resolvedOptions be ? GetOptionsObject(options).
        // 4. Let overflow be ? GetTemporalOverflowOption(resolvedOptions).
        // 5. Let calendar be zonedDateTime.[[Calendar]].
        // 6. Let timeZone be zonedDateTime.[[TimeZone]].
        // 7. Let internalDuration be ToInternalDurationRecord(duration).
        // 8. Let epochNanoseconds be ? AddZonedDateTime(zonedDateTime.[[EpochNanoseconds]], timeZone, calendar, internalDuration, overflow).
        let epoch_ns = self.add_as_instant(duration, overflow, provider)?;
        // 9. Return ! CreateTemporalZonedDateTime(epochNanoseconds, timeZone, calendar).
        Ok(Self::new_unchecked(
            epoch_ns,
            self.calendar().clone(),
            self.timezone().clone(),
        ))
    }
}

// ==== Public API ====

impl ZonedDateTime {
    /// Creates a new valid `ZonedDateTime`.
    #[inline]
    pub fn try_new(nanos: i128, calendar: Calendar, tz: TimeZone) -> TemporalResult<Self> {
        let instant = Instant::try_new(nanos)?;
        Ok(Self::new_unchecked(instant, calendar, tz))
    }

    /// Returns `ZonedDateTime`'s Calendar.
    #[inline]
    #[must_use]
    pub fn calendar(&self) -> &Calendar {
        &self.calendar
    }

    /// Returns `ZonedDateTime`'s `TimeZone` slot.
    #[inline]
    #[must_use]
    pub fn timezone(&self) -> &TimeZone {
        &self.tz
    }

    #[inline]
    pub fn from_partial_with_provider(
        partial: PartialZonedDateTime,
        calendar: Option<Calendar>,
        overflow: Option<ArithmeticOverflow>,
        disambiguation: Disambiguation,
        offset_option: OffsetDisambiguation,
        provider: &impl TzProvider,
    ) -> TemporalResult<Self> {
        let calendar = calendar.unwrap_or_default();
        let overflow = overflow.unwrap_or(ArithmeticOverflow::Constrain);
        let date = calendar.date_from_partial(&partial.date, overflow)?.iso;
        let time = if !partial.time.is_empty() {
            Some(IsoTime::default().with(partial.time, overflow)?)
        } else {
            None
        };

        // Handle time zones
        let offset = partial
            .offset
            .map(|offset| {
                let mut cursor = offset.chars().peekable();
                parse_offset(&mut cursor)
            })
            .transpose()?;

        let offset_nanos = match offset {
            Some(TimeZone::OffsetMinutes(minutes)) => Some(i64::from(minutes) * 60_000_000_000),
            None => None,
            _ => unreachable!(),
        };

        let epoch_nanos = interpret_isodatetime_offset(
            date,
            time,
            offset_nanos,
            &partial.timezone,
            disambiguation,
            offset_option,
            true,
            provider,
        )?;

        Ok(Self::new_unchecked(Instant::from(epoch_nanos), calendar, partial.timezone))
    }

    /// Returns the `epochSeconds` value of this `ZonedDateTime`.
    #[must_use]
    pub fn epoch_seconds(&self) -> i128 {
        self.instant.epoch_seconds()
    }

    /// Returns the `epochMilliseconds` value of this `ZonedDateTime`.
    #[must_use]
    pub fn epoch_milliseconds(&self) -> i128 {
        self.instant.epoch_milliseconds()
    }

    /// Returns the `epochMicroseconds` value of this `ZonedDateTime`.
    #[must_use]
    pub fn epoch_microseconds(&self) -> i128 {
        self.instant.epoch_microseconds()
    }

    /// Returns the `epochNanoseconds` value of this `ZonedDateTime`.
    #[must_use]
    pub fn epoch_nanoseconds(&self) -> i128 {
        self.instant.epoch_nanoseconds()
    }
}

// ===== TzProvider APIs for ZonedDateTime =====

#[cfg(feature = "experimental")]
impl ZonedDateTime {
    pub fn year(&self) -> TemporalResult<i32> {
        let provider = TZ_PROVIDER
            .lock()
            .map_err(|_| TemporalError::general("Unable to acquire lock"))?;
        self.year_with_provider(provider.deref())
    }

    pub fn month(&self) -> TemporalResult<u8> {
        let provider = TZ_PROVIDER
            .lock()
            .map_err(|_| TemporalError::general("Unable to acquire lock"))?;
        self.month_with_provider(provider.deref())
    }

    pub fn month_code(&self) -> TemporalResult<TinyAsciiStr<4>> {
        let provider = TZ_PROVIDER
            .lock()
            .map_err(|_| TemporalError::general("Unable to acquire lock"))?;
        self.month_code_with_provider(provider.deref())
    }

    pub fn day(&self) -> TemporalResult<u8> {
        let provider = TZ_PROVIDER
            .lock()
            .map_err(|_| TemporalError::general("Unable to acquire lock"))?;
        self.day_with_provider(provider.deref())
    }

    pub fn hour(&self) -> TemporalResult<u8> {
        let provider = TZ_PROVIDER
            .lock()
            .map_err(|_| TemporalError::general("Unable to acquire lock"))?;
        self.hour_with_provider(provider.deref())
    }

    pub fn minute(&self) -> TemporalResult<u8> {
        let provider = TZ_PROVIDER
            .lock()
            .map_err(|_| TemporalError::general("Unable to acquire lock"))?;
        self.minute_with_provider(provider.deref())
    }

    pub fn second(&self) -> TemporalResult<u8> {
        let provider = TZ_PROVIDER
            .lock()
            .map_err(|_| TemporalError::general("Unable to acquire lock"))?;
        self.second_with_provider(provider.deref())
    }

    pub fn millisecond(&self) -> TemporalResult<u16> {
        let provider = TZ_PROVIDER
            .lock()
            .map_err(|_| TemporalError::general("Unable to acquire lock"))?;
        self.millisecond_with_provider(provider.deref())
    }

    pub fn microsecond(&self) -> TemporalResult<u16> {
        let provider = TZ_PROVIDER
            .lock()
            .map_err(|_| TemporalError::general("Unable to acquire lock"))?;
        self.millisecond_with_provider(provider.deref())
    }

    pub fn nanosecond(&self) -> TemporalResult<u16> {
        let provider = TZ_PROVIDER
            .lock()
            .map_err(|_| TemporalError::general("Unable to acquire lock"))?;

        self.millisecond_with_provider(provider.deref())
    }

    pub fn add(
        &self,
        duration: &Duration,
        overflow: Option<ArithmeticOverflow>,
    ) -> TemporalResult<Self> {
        let provider = TZ_PROVIDER
            .lock()
            .map_err(|_| TemporalError::general("Unable to acquire lock"))?;

        self.add_internal(
            duration,
            overflow.unwrap_or(ArithmeticOverflow::Constrain),
            provider.deref(),
        )
    }

    pub fn subtract(
        &self,
        duration: &Duration,
        overflow: Option<ArithmeticOverflow>,
    ) -> TemporalResult<Self> {
        let provider = TZ_PROVIDER
            .lock()
            .map_err(|_| TemporalError::general("Unable to acquire lock"))?;
        self.add_internal(
            &duration.negated(),
            overflow.unwrap_or(ArithmeticOverflow::Constrain),
            provider.deref(),
        )
    }
}

impl ZonedDateTime {
    /// Returns the `year` value for this `ZonedDateTime`.
    #[inline]
    pub fn year_with_provider(&self, provider: &impl TzProvider) -> TemporalResult<i32> {
        let iso = self.tz.get_iso_datetime_for(&self.instant, provider)?;
        let dt = PlainDateTime::new_unchecked(iso, self.calendar.clone());
        self.calendar.year(&CalendarDateLike::DateTime(&dt))
    }

    /// Returns the `month` value for this `ZonedDateTime`.
    pub fn month_with_provider(&self, provider: &impl TzProvider) -> TemporalResult<u8> {
        let iso = self.tz.get_iso_datetime_for(&self.instant, provider)?;
        let dt = PlainDateTime::new_unchecked(iso, self.calendar.clone());
        self.calendar.month(&CalendarDateLike::DateTime(&dt))
    }

    /// Returns the `monthCode` value for this `ZonedDateTime`.
    pub fn month_code_with_provider(
        &self,
        provider: &impl TzProvider,
    ) -> TemporalResult<TinyAsciiStr<4>> {
        let iso = self.tz.get_iso_datetime_for(&self.instant, provider)?;
        let dt = PlainDateTime::new_unchecked(iso, self.calendar.clone());
        self.calendar.month_code(&CalendarDateLike::DateTime(&dt))
    }

    /// Returns the `day` value for this `ZonedDateTime`.
    pub fn day_with_provider(&self, provider: &impl TzProvider) -> TemporalResult<u8> {
        let iso = self.tz.get_iso_datetime_for(&self.instant, provider)?;
        let dt = PlainDateTime::new_unchecked(iso, self.calendar.clone());
        self.calendar.day(&CalendarDateLike::DateTime(&dt))
    }

    /// Returns the `hour` value for this `ZonedDateTime`.
    pub fn hour_with_provider(&self, provider: &impl TzProvider) -> TemporalResult<u8> {
        let iso = self.tz.get_iso_datetime_for(&self.instant, provider)?;
        Ok(iso.time.hour)
    }

    /// Returns the `minute` value for this `ZonedDateTime`.
    pub fn minute_with_provider(&self, provider: &impl TzProvider) -> TemporalResult<u8> {
        let iso = self.tz.get_iso_datetime_for(&self.instant, provider)?;
        Ok(iso.time.minute)
    }

    /// Returns the `second` value for this `ZonedDateTime`.
    pub fn second_with_provider(&self, provider: &impl TzProvider) -> TemporalResult<u8> {
        let iso = self.tz.get_iso_datetime_for(&self.instant, provider)?;
        Ok(iso.time.second)
    }

    /// Returns the `millisecond` value for this `ZonedDateTime`.
    pub fn millisecond_with_provider(&self, provider: &impl TzProvider) -> TemporalResult<u16> {
        let iso = self.tz.get_iso_datetime_for(&self.instant, provider)?;
        Ok(iso.time.millisecond)
    }

    /// Returns the `microsecond` value for this `ZonedDateTime`.
    pub fn microsecond_with_provider(&self, provider: &impl TzProvider) -> TemporalResult<u16> {
        let iso = self.tz.get_iso_datetime_for(&self.instant, provider)?;
        Ok(iso.time.millisecond)
    }

    /// Returns the `nanosecond` value for this `ZonedDateTime`.
    pub fn nanosecond_with_provider(&self, provider: &impl TzProvider) -> TemporalResult<u16> {
        let iso = self.tz.get_iso_datetime_for(&self.instant, provider)?;
        Ok(iso.time.nanosecond)
    }

    pub fn add_with_provider(
        &self,
        duration: &Duration,
        overflow: Option<ArithmeticOverflow>,
        provider: &impl TzProvider,
    ) -> TemporalResult<Self> {
        self.add_internal(
            duration,
            overflow.unwrap_or(ArithmeticOverflow::Constrain),
            provider,
        )
    }

    pub fn subtract_with_provider(
        &self,
        duration: &Duration,
        overflow: Option<ArithmeticOverflow>,
        provider: &impl TzProvider,
    ) -> TemporalResult<Self> {
        self.add_internal(
            &duration.negated(),
            overflow.unwrap_or(ArithmeticOverflow::Constrain),
            provider,
        )
    }

    // TODO: Should IANA Identifier be prechecked or allow potentially invalid IANA Identifer values here?
    pub fn from_str_with_provider(
        source: &str,
        disambiguation: Disambiguation,
        offset_option: OffsetDisambiguation,
        provider: &impl TzProvider,
    ) -> TemporalResult<Self> {
        let parse_result = parsers::parse_date_time(source)?;

        let Some(annotation) = parse_result.tz else {
            return Err(TemporalError::r#type()
                .with_message("Time zone annotation is required for ZonedDateTime string."));
        };

        let timezone = match annotation.tz {
            TimeZoneRecord::Name(s) => TimeZone::IanaIdentifier(s.to_owned()),
            TimeZoneRecord::Offset(offset_record) => {
                // NOTE: ixdtf parser restricts minute/second to 0..=60
                let minutes = i16::from((offset_record.hour * 60) + offset_record.minute);
                TimeZone::OffsetMinutes(minutes * i16::from(offset_record.sign as i8))
            }
            // TimeZoneRecord is non_exhaustive, but all current branches are matching.
            _ => return Err(TemporalError::assert()),
        };

        let offset_nanos = parse_result.offset.map(|record| {
            let hours_in_ns = i64::from(record.hour) * 3_600_000_000_000_i64;
            let minutes_in_ns = i64::from(record.minute) * 60_000_000_000_i64;
            let seconds_in_ns = i64::from(record.minute) * 1_000_000_000_i64;
            (hours_in_ns + minutes_in_ns + seconds_in_ns + i64::from(record.nanosecond))
                * i64::from(record.sign as i8)
        });

        let calendar = Calendar::from_str(parse_result.calendar.unwrap_or("iso8601"))?;

        let time = parse_result
            .time
            .map(|time| {
                IsoTime::from_components(
                    i32::from(time.hour),
                    i32::from(time.minute),
                    i32::from(time.second),
                    f64::from(time.nanosecond),
                )
            })
            .transpose()?;

        let Some(parsed_date) = parse_result.date else {
            return Err(
                TemporalError::range().with_message("No valid DateRecord Parse Node was found.")
            );
        };

        let date = IsoDate::new_with_overflow(
            parsed_date.year,
            parsed_date.month.into(),
            parsed_date.day.into(),
            ArithmeticOverflow::Reject,
        )?;

        let epoch_nanos = interpret_isodatetime_offset(
            date,
            time,
            offset_nanos,
            &timezone,
            disambiguation,
            offset_option,
            true,
            provider,
        )?;

        Ok(Self::new_unchecked(
            Instant::from(epoch_nanos),
            calendar,
            timezone,
        ))
    }
}

#[allow(clippy::too_many_arguments)]
pub fn interpret_isodatetime_offset(
    date: IsoDate,
    time: Option<IsoTime>,
    offset_nanos: Option<i64>,
    timezone: &TimeZone,
    disambiguation: Disambiguation,
    offset_option: OffsetDisambiguation,
    match_minutes: bool,
    provider: &impl TzProvider,
) -> TemporalResult<EpochNanoseconds> {
    // 1.  If time is start-of-day, then
    let Some(time) = time else {
        // a. Assert: offsetBehaviour is wall.
        // b. Assert: offsetNanoseconds is 0.
        temporal_assert!(offset_nanos.is_none());
        // c. Return ? GetStartOfDay(timeZone, isoDate).
        return timezone.get_start_of_day(&date, provider);
    };

    // 2. Let isoDateTime be CombineISODateAndTimeRecord(isoDate, time).
    // TODO: Deal with offsetBehavior == wall.
    match offset_nanos {
        // 4. If offsetBehaviour is exact, or offsetBehaviour is option and offsetOption is use, then
        Some(offset) if offset_option == OffsetDisambiguation::Use => {
            // a. Let balanced be BalanceISODateTime(isoDate.[[Year]], isoDate.[[Month]],
            // isoDate.[[Day]], time.[[Hour]], time.[[Minute]], time.[[Second]], time.[[Millisecond]],
            // time.[[Microsecond]], time.[[Nanosecond]] - offsetNanoseconds).
            let iso = IsoDateTime::balance(
                date.year,
                date.month.into(),
                date.day.into(),
                time.hour.into(),
                time.minute.into(),
                time.second.into(),
                time.millisecond.into(),
                time.microsecond.into(),
                i64::from(time.nanosecond) - offset,
            );

            // b. Perform ? CheckISODaysRange(balanced.[[ISODate]]).
            iso.date.is_valid_day_range()?;

            // c. Let epochNanoseconds be GetUTCEpochNanoseconds(balanced).
            // d. If IsValidEpochNanoseconds(epochNanoseconds) is false, throw a RangeError exception.
            // e. Return epochNanoseconds.
            iso.as_nanoseconds()
        }
        // 5. Assert: offsetBehaviour is option.
        // 6. Assert: offsetOption is prefer or reject.
        Some(offset)
            if offset_option == OffsetDisambiguation::Prefer
                || offset_option == OffsetDisambiguation::Reject =>
        {
            // 7. Perform ? CheckISODaysRange(isoDate).
            date.is_valid_day_range()?;
            let iso = IsoDateTime::new_unchecked(date, time);
            // 8. Let utcEpochNanoseconds be GetUTCEpochNanoseconds(isoDateTime).
            let utc_epochs = iso.as_nanoseconds()?;
            // 9. Let possibleEpochNs be ? GetPossibleEpochNanoseconds(timeZone, isoDateTime).
            let possible_nanos = timezone.get_possible_epoch_ns_for(iso, provider)?;
            // 10. For each element candidate of possibleEpochNs, do
            for candidate in &possible_nanos {
                // a. Let candidateOffset be utcEpochNanoseconds - candidate.
                let candidate_offset = utc_epochs.0 - candidate;
                // b. If candidateOffset = offsetNanoseconds, then
                if candidate_offset == offset.into() {
                    // i. Return candidate.
                    return EpochNanoseconds::try_from(*candidate);
                }
                // c. If matchBehaviour is match-minutes, then
                if match_minutes {
                    // i. Let roundedCandidateNanoseconds be RoundNumberToIncrement(candidateOffset, 60 × 10**9, half-expand).
                    let rounded_candidate = IncrementRounder::from_potentially_negative_parts(
                        candidate_offset,
                        unsafe { NonZeroU128::new_unchecked(60_000_000_000) },
                    )?
                    .round(TemporalRoundingMode::HalfExpand);
                    // ii. If roundedCandidateNanoseconds = offsetNanoseconds, then
                    if rounded_candidate == offset.into() {
                        // 1. Return candidate.
                        return EpochNanoseconds::try_from(*candidate);
                    }
                }
            }

            // 11. If offsetOption is reject, throw a RangeError exception.
            if offset_option == OffsetDisambiguation::Reject {
                return Err(TemporalError::range()
                    .with_message("Offsets could not be determined without disambiguation"));
            }
            // 12. Return ? DisambiguatePossibleEpochNanoseconds(possibleEpochNs, timeZone, isoDateTime, disambiguation).
            timezone.disambiguate_possible_epoch_nanos(
                possible_nanos,
                iso,
                disambiguation,
                provider,
            )
        }
        // NOTE: This is inverted as the logic works better for matching against
        // 3. If offsetBehaviour is wall, or offsetBehaviour is option and offsetOption is ignore, then
        _ => {
            // a. Return ? GetEpochNanosecondsFor(timeZone, isoDateTime, disambiguation).
            let iso = IsoDateTime::new_unchecked(date, time);
            timezone.get_epoch_nanoseconds_for(iso, disambiguation, provider)
        }
    }
}

#[cfg(feature = "tzdb")]
#[cfg(test)]
mod tests {

    use core::str::FromStr;

    use crate::{tzdb::FsTzdbProvider, Calendar};

    #[cfg(not(target_os = "windows"))]
    use crate::{Duration, TimeZone};

    use super::ZonedDateTime;

    #[test]
    fn basic_zdt_test() {
        let provider = &FsTzdbProvider::default();
        let nov_30_2023_utc = 1_701_308_952_000_000_000i128;

        let zdt = ZonedDateTime::try_new(
            nov_30_2023_utc,
            Calendar::from_str("iso8601").unwrap(),
            "Z".into(),
        )
        .unwrap();

        assert_eq!(zdt.year_with_provider(provider).unwrap(), 2023);
        assert_eq!(zdt.month_with_provider(provider).unwrap(), 11);
        assert_eq!(zdt.day_with_provider(provider).unwrap(), 30);
        assert_eq!(zdt.hour_with_provider(provider).unwrap(), 1);
        assert_eq!(zdt.minute_with_provider(provider).unwrap(), 49);
        assert_eq!(zdt.second_with_provider(provider).unwrap(), 12);

        let zdt_minus_five = ZonedDateTime::try_new(
            nov_30_2023_utc,
            Calendar::from_str("iso8601").unwrap(),
            "America/New_York".into(),
        )
        .unwrap();

        assert_eq!(zdt_minus_five.year_with_provider(provider).unwrap(), 2023);
        assert_eq!(zdt_minus_five.month_with_provider(provider).unwrap(), 11);
        assert_eq!(zdt_minus_five.day_with_provider(provider).unwrap(), 29);
        assert_eq!(zdt_minus_five.hour_with_provider(provider).unwrap(), 20);
        assert_eq!(zdt_minus_five.minute_with_provider(provider).unwrap(), 49);
        assert_eq!(zdt_minus_five.second_with_provider(provider).unwrap(), 12);

        let zdt_plus_eleven = ZonedDateTime::try_new(
            nov_30_2023_utc,
            Calendar::from_str("iso8601").unwrap(),
            "Australia/Sydney".into(),
        )
        .unwrap();

        assert_eq!(zdt_plus_eleven.year_with_provider(provider).unwrap(), 2023);
        assert_eq!(zdt_plus_eleven.month_with_provider(provider).unwrap(), 11);
        assert_eq!(zdt_plus_eleven.day_with_provider(provider).unwrap(), 30);
        assert_eq!(zdt_plus_eleven.hour_with_provider(provider).unwrap(), 12);
        assert_eq!(zdt_plus_eleven.minute_with_provider(provider).unwrap(), 49);
        assert_eq!(zdt_plus_eleven.second_with_provider(provider).unwrap(), 12);
    }

    #[cfg(all(feature = "experimental", not(target_os = "windows")))]
    #[test]
    fn static_tzdb_zdt_test() {
        let nov_30_2023_utc = 1_701_308_952_000_000_000i128;

        let zdt = ZonedDateTime::try_new(
            nov_30_2023_utc,
            Calendar::from_str("iso8601").unwrap(),
            "Z".into(),
        )
        .unwrap();

        assert_eq!(zdt.year().unwrap(), 2023);
        assert_eq!(zdt.month().unwrap(), 11);
        assert_eq!(zdt.day().unwrap(), 30);
        assert_eq!(zdt.hour().unwrap(), 1);
        assert_eq!(zdt.minute().unwrap(), 49);
        assert_eq!(zdt.second().unwrap(), 12);

        let zdt_minus_five = ZonedDateTime::try_new(
            nov_30_2023_utc,
            Calendar::from_str("iso8601").unwrap(),
            "America/New_York".into(),
        )
        .unwrap();

        assert_eq!(zdt_minus_five.year().unwrap(), 2023);
        assert_eq!(zdt_minus_five.month().unwrap(), 11);
        assert_eq!(zdt_minus_five.day().unwrap(), 29);
        assert_eq!(zdt_minus_five.hour().unwrap(), 20);
        assert_eq!(zdt_minus_five.minute().unwrap(), 49);
        assert_eq!(zdt_minus_five.second().unwrap(), 12);

        let zdt_plus_eleven = ZonedDateTime::try_new(
            nov_30_2023_utc,
            Calendar::from_str("iso8601").unwrap(),
            "Australia/Sydney".into(),
        )
        .unwrap();

        assert_eq!(zdt_plus_eleven.year().unwrap(), 2023);
        assert_eq!(zdt_plus_eleven.month().unwrap(), 11);
        assert_eq!(zdt_plus_eleven.day().unwrap(), 30);
        assert_eq!(zdt_plus_eleven.hour().unwrap(), 12);
        assert_eq!(zdt_plus_eleven.minute().unwrap(), 49);
        assert_eq!(zdt_plus_eleven.second().unwrap(), 12);
    }

    #[cfg(all(feature = "experimental", not(target_os = "windows")))]
    #[test]
    fn basic_zdt_add() {
        let zdt =
            ZonedDateTime::try_new(-560174321098766, Calendar::default(), TimeZone::default())
                .unwrap();
        let d = Duration::new(
            0.into(),
            0.into(),
            0.into(),
            0.into(),
            240.into(),
            0.into(),
            0.into(),
            0.into(),
            0.into(),
            800.into(),
        )
        .unwrap();
        // "1970-01-04T12:23:45.678902034+00:00[UTC]"
        let expected =
            ZonedDateTime::try_new(303825678902034, Calendar::default(), TimeZone::default())
                .unwrap();

        let result = zdt.add(&d, None).unwrap();
        assert_eq!(result, expected);
    }
}
