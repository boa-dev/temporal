//! This module implements `ZonedDateTime` and any directly related algorithms.

use alloc::{borrow::ToOwned, string::String};
use core::{num::NonZeroU128, str::FromStr};
use ixdtf::parsers::records::TimeZoneRecord;
use tinystr::TinyAsciiStr;

use crate::{
    components::{
        calendar::CalendarDateLike,
        duration::normalized::NormalizedTimeDuration,
        tz::{parse_offset, TzProvider},
        EpochNanoseconds,
    },
    iso::{IsoDate, IsoDateTime, IsoTime},
    options::{ArithmeticOverflow, Disambiguation, OffsetDisambiguation, TemporalRoundingMode},
    parsers,
    partial::{PartialDate, PartialTime},
    rounding::{IncrementRounder, Round},
    temporal_assert, Calendar, Duration, Instant, PlainDate, PlainDateTime, Sign, TemporalError,
    TemporalResult, TimeZone,
};

#[cfg(feature = "experimental")]
use crate::components::tz::TZ_PROVIDER;
#[cfg(feature = "experimental")]
use std::ops::Deref;

/// A struct representing a partial `ZonedDateTime`.
pub struct PartialZonedDateTime {
    /// The `PartialDate` portion of a `PartialZonedDateTime`
    pub date: PartialDate,
    /// The `PartialTime` portion of a `PartialZonedDateTime`
    pub time: PartialTime,
    /// An optional offset string
    pub offset: Option<String>,
    /// The time zone value of a partial time zone.
    pub timezone: TimeZone,
}

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
        disambiguation: Option<Disambiguation>,
        offset_option: Option<OffsetDisambiguation>,
        provider: &impl TzProvider,
    ) -> TemporalResult<Self> {
        let calendar = calendar.unwrap_or_default();
        let overflow = overflow.unwrap_or(ArithmeticOverflow::Constrain);
        let disambiguation = disambiguation.unwrap_or(Disambiguation::Compatible);
        let offset_option = offset_option.unwrap_or(OffsetDisambiguation::Reject);

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

        Ok(Self::new_unchecked(
            Instant::from(epoch_nanos),
            calendar,
            partial.timezone,
        ))
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

// ===== Experimental TZ_PROVIDER accessor implementations =====

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
}

// ==== Experimental TZ_PROVIDER calendar method implementations ====

#[cfg(feature = "experimental")]
impl ZonedDateTime {
    pub fn era(&self) -> TemporalResult<Option<TinyAsciiStr<16>>> {
        let provider = TZ_PROVIDER
            .lock()
            .map_err(|_| TemporalError::general("Unable to acquire lock"))?;
        self.era_with_provider(provider.deref())
    }

    pub fn era_year(&self) -> TemporalResult<Option<i32>> {
        let provider = TZ_PROVIDER
            .lock()
            .map_err(|_| TemporalError::general("Unable to acquire lock"))?;
        self.era_year_with_provider(provider.deref())
    }

    /// Returns the calendar day of week value.
    pub fn day_of_week(&self) -> TemporalResult<u16> {
        let provider = TZ_PROVIDER
            .lock()
            .map_err(|_| TemporalError::general("Unable to acquire lock"))?;
        self.day_of_week_with_provider(provider.deref())
    }

    /// Returns the calendar day of year value.
    pub fn day_of_year(&self) -> TemporalResult<u16> {
        let provider = TZ_PROVIDER
            .lock()
            .map_err(|_| TemporalError::general("Unable to acquire lock"))?;
        self.day_of_year_with_provider(provider.deref())
    }

    /// Returns the calendar week of year value.
    pub fn week_of_year(&self) -> TemporalResult<Option<u16>> {
        let provider = TZ_PROVIDER
            .lock()
            .map_err(|_| TemporalError::general("Unable to acquire lock"))?;
        self.week_of_year_with_provider(provider.deref())
    }

    /// Returns the calendar year of week value.
    pub fn year_of_week(&self) -> TemporalResult<Option<i32>> {
        let provider = TZ_PROVIDER
            .lock()
            .map_err(|_| TemporalError::general("Unable to acquire lock"))?;
        self.year_of_week_with_provider(provider.deref())
    }

    /// Returns the calendar days in week value.
    pub fn days_in_week(&self) -> TemporalResult<u16> {
        let provider = TZ_PROVIDER
            .lock()
            .map_err(|_| TemporalError::general("Unable to acquire lock"))?;
        self.days_in_week_with_provider(provider.deref())
    }

    /// Returns the calendar days in month value.
    pub fn days_in_month(&self) -> TemporalResult<u16> {
        let provider = TZ_PROVIDER
            .lock()
            .map_err(|_| TemporalError::general("Unable to acquire lock"))?;
        self.days_in_month_with_provider(provider.deref())
    }

    /// Returns the calendar days in year value.
    pub fn days_in_year(&self) -> TemporalResult<u16> {
        let provider = TZ_PROVIDER
            .lock()
            .map_err(|_| TemporalError::general("Unable to acquire lock"))?;
        self.days_in_year_with_provider(provider.deref())
    }

    /// Returns the calendar months in year value.
    pub fn months_in_year(&self) -> TemporalResult<u16> {
        let provider = TZ_PROVIDER
            .lock()
            .map_err(|_| TemporalError::general("Unable to acquire lock"))?;
        self.months_in_year_with_provider(provider.deref())
    }

    /// Returns returns whether the date in a leap year for the given calendar.
    pub fn in_leap_year(&self) -> TemporalResult<bool> {
        let provider = TZ_PROVIDER
            .lock()
            .map_err(|_| TemporalError::general("Unable to acquire lock"))?;
        self.in_leap_year_with_provider(provider.deref())
    }
}

// ==== Experimental TZ_PROVIDER method implementations ====

#[cfg(feature = "experimental")]
impl ZonedDateTime {
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

    pub fn from_str(
        source: &str,
        disambiguation: Disambiguation,
        offset_option: OffsetDisambiguation,
    ) -> TemporalResult<Self> {
        let provider = TZ_PROVIDER
            .lock()
            .map_err(|_| TemporalError::general("Unable to acquire lock"))?;
        Self::from_str_with_provider(source, disambiguation, offset_option, provider.deref())
    }
}

// ==== HoursInDay accessor method implementation ====

impl ZonedDateTime {
    #[cfg(feature = "experimental")]
    pub fn hours_in_day(&self) -> TemporalResult<u8> {
        let provider = TZ_PROVIDER
            .lock()
            .map_err(|_| TemporalError::general("Unable to acquire lock"))?;
        self.hours_in_day_with_provider(provider.deref())
    }

    pub fn hours_in_day_with_provider(&self, provider: &impl TzProvider) -> TemporalResult<u8> {
        // 1-3. Is engine specific steps
        // 4. Let isoDateTime be GetISODateTimeFor(timeZone, zonedDateTime.[[EpochNanoseconds]]).
        let iso = self.tz.get_iso_datetime_for(&self.instant, provider)?;
        // 5. Let today be isoDateTime.[[ISODate]].
        let today = iso.date;
        // 6. Let tomorrow be BalanceISODate(today.[[Year]], today.[[Month]], today.[[Day]] + 1).
        let tomorrow = IsoDate::balance(today.year, today.month.into(), i32::from(today.day + 1));
        // 7. Let todayNs be ? GetStartOfDay(timeZone, today).
        let today_ns = self.tz.get_start_of_day(&today, provider)?;
        // 8. Let tomorrowNs be ? GetStartOfDay(timeZone, tomorrow).
        let tomorrow_ns = self.tz.get_start_of_day(&tomorrow, provider)?;
        // 9. Let diff be TimeDurationFromEpochNanosecondsDifference(tomorrowNs, todayNs).
        let diff = NormalizedTimeDuration::from_nanosecond_difference(tomorrow_ns.0, today_ns.0)?;
        // NOTE: The below should be safe as today_ns and tomorrow_ns should be at most 25 hours.
        // TODO: Tests for the below cast.
        // 10. Return 𝔽(TotalTimeDuration(diff, hour)).
        Ok(diff.divide(60_000_000_000) as u8)
    }
}

// ==== Core accessor methods ====

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
}

// ==== Core calendar method implementations ====

impl ZonedDateTime {
    pub fn era_with_provider(
        &self,
        provider: &impl TzProvider,
    ) -> TemporalResult<Option<TinyAsciiStr<16>>> {
        let iso = self.tz.get_iso_datetime_for(&self.instant, provider)?;
        let pdt = PlainDateTime::new_unchecked(iso, self.calendar.clone());
        self.calendar.era(&CalendarDateLike::DateTime(&pdt))
    }

    pub fn era_year_with_provider(
        &self,
        provider: &impl TzProvider,
    ) -> TemporalResult<Option<i32>> {
        let iso = self.tz.get_iso_datetime_for(&self.instant, provider)?;
        let pdt = PlainDateTime::new_unchecked(iso, self.calendar.clone());
        self.calendar.era_year(&CalendarDateLike::DateTime(&pdt))
    }

    /// Returns the calendar day of week value.
    pub fn day_of_week_with_provider(&self, provider: &impl TzProvider) -> TemporalResult<u16> {
        let iso = self.tz.get_iso_datetime_for(&self.instant, provider)?;
        let pdt = PlainDateTime::new_unchecked(iso, self.calendar.clone());
        self.calendar.day_of_week(&CalendarDateLike::DateTime(&pdt))
    }

    /// Returns the calendar day of year value.
    pub fn day_of_year_with_provider(&self, provider: &impl TzProvider) -> TemporalResult<u16> {
        let iso = self.tz.get_iso_datetime_for(&self.instant, provider)?;
        let pdt = PlainDateTime::new_unchecked(iso, self.calendar.clone());
        self.calendar.day_of_year(&CalendarDateLike::DateTime(&pdt))
    }

    /// Returns the calendar week of year value.
    pub fn week_of_year_with_provider(
        &self,
        provider: &impl TzProvider,
    ) -> TemporalResult<Option<u16>> {
        let iso = self.tz.get_iso_datetime_for(&self.instant, provider)?;
        let pdt = PlainDateTime::new_unchecked(iso, self.calendar.clone());
        self.calendar
            .week_of_year(&CalendarDateLike::DateTime(&pdt))
    }

    /// Returns the calendar year of week value.
    pub fn year_of_week_with_provider(
        &self,
        provider: &impl TzProvider,
    ) -> TemporalResult<Option<i32>> {
        let iso = self.tz.get_iso_datetime_for(&self.instant, provider)?;
        let pdt = PlainDateTime::new_unchecked(iso, self.calendar.clone());
        self.calendar
            .year_of_week(&CalendarDateLike::DateTime(&pdt))
    }

    /// Returns the calendar days in week value.
    pub fn days_in_week_with_provider(&self, provider: &impl TzProvider) -> TemporalResult<u16> {
        let iso = self.tz.get_iso_datetime_for(&self.instant, provider)?;
        let pdt = PlainDateTime::new_unchecked(iso, self.calendar.clone());
        self.calendar
            .days_in_week(&CalendarDateLike::DateTime(&pdt))
    }

    /// Returns the calendar days in month value.
    pub fn days_in_month_with_provider(&self, provider: &impl TzProvider) -> TemporalResult<u16> {
        let iso = self.tz.get_iso_datetime_for(&self.instant, provider)?;
        let pdt = PlainDateTime::new_unchecked(iso, self.calendar.clone());
        self.calendar
            .days_in_month(&CalendarDateLike::DateTime(&pdt))
    }

    /// Returns the calendar days in year value.
    pub fn days_in_year_with_provider(&self, provider: &impl TzProvider) -> TemporalResult<u16> {
        let iso = self.tz.get_iso_datetime_for(&self.instant, provider)?;
        let pdt = PlainDateTime::new_unchecked(iso, self.calendar.clone());
        self.calendar
            .days_in_year(&CalendarDateLike::DateTime(&pdt))
    }

    /// Returns the calendar months in year value.
    pub fn months_in_year_with_provider(&self, provider: &impl TzProvider) -> TemporalResult<u16> {
        let iso = self.tz.get_iso_datetime_for(&self.instant, provider)?;
        let pdt = PlainDateTime::new_unchecked(iso, self.calendar.clone());
        self.calendar
            .months_in_year(&CalendarDateLike::DateTime(&pdt))
    }

    /// Returns returns whether the date in a leap year for the given calendar.
    pub fn in_leap_year_with_provider(&self, provider: &impl TzProvider) -> TemporalResult<bool> {
        let iso = self.tz.get_iso_datetime_for(&self.instant, provider)?;
        let pdt = PlainDateTime::new_unchecked(iso, self.calendar.clone());
        self.calendar
            .in_leap_year(&CalendarDateLike::DateTime(&pdt))
    }
}

// ==== Core method implementations ====

impl ZonedDateTime {
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
                let candidate_offset = utc_epochs.0 - candidate.0;
                // b. If candidateOffset = offsetNanoseconds, then
                if candidate_offset == offset.into() {
                    // i. Return candidate.
                    return Ok(*candidate);
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
                        return Ok(*candidate);
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
    use crate::{
        options::{Disambiguation, OffsetDisambiguation},
        partial::{PartialDate, PartialTime, PartialZonedDateTime},
        tzdb::FsTzdbProvider,
        Calendar, TimeZone, ZonedDateTime,
    };
    use core::str::FromStr;
    use tinystr::tinystr;

    #[cfg(not(target_os = "windows"))]
    use crate::Duration;

    #[test]
    fn basic_zdt_test() {
        let provider = &FsTzdbProvider::default();
        let nov_30_2023_utc = 1_701_308_952_000_000_000i128;

        let zdt = ZonedDateTime::try_new(
            nov_30_2023_utc,
            Calendar::from_str("iso8601").unwrap(),
            TimeZone::try_from_str_with_provider("Z", provider).unwrap(),
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
            TimeZone::try_from_str_with_provider("America/New_York", provider).unwrap(),
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
            TimeZone::try_from_str_with_provider("Australia/Sydney", provider).unwrap(),
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
            TimeZone::try_from_str("Z").unwrap(),
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
            TimeZone::try_from_str("America/New_York").unwrap(),
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
            TimeZone::try_from_str("Australia/Sydney").unwrap(),
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

    #[test]
    fn zdt_from_partial() {
        let provider = &FsTzdbProvider::default();
        let partial = PartialZonedDateTime {
            date: PartialDate {
                year: Some(1970),
                month_code: Some(tinystr!(4, "M01")),
                day: Some(1),
                ..Default::default()
            },
            time: PartialTime::default(),
            offset: None,
            timezone: TimeZone::default(),
        };

        let result =
            ZonedDateTime::from_partial_with_provider(partial, None, None, None, None, provider);
        assert!(result.is_ok());
    }

    #[test]
    fn zdt_from_str() {
        let provider = &FsTzdbProvider::default();

        let zdt_str = "1970-01-01T00:00[UTC][u-ca=iso8601]";
        let result = ZonedDateTime::from_str_with_provider(
            zdt_str,
            Disambiguation::Compatible,
            OffsetDisambiguation::Reject,
            provider,
        );
        assert!(result.is_ok());
    }
}
