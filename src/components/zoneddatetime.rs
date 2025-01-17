//! This module implements `ZonedDateTime` and any directly related algorithms.

use alloc::string::String;
use core::num::NonZeroU128;
use ixdtf::parsers::records::{TimeZoneRecord, UtcOffsetRecordOrZ};
use tinystr::TinyAsciiStr;

use crate::{
    components::{
        duration::normalized::{NormalizedDurationRecord, NormalizedTimeDuration},
        timezone::{parse_offset, TimeZoneProvider},
        EpochNanoseconds,
    },
    iso::{IsoDate, IsoDateTime, IsoTime},
    options::{
        ArithmeticOverflow, Disambiguation, DisplayCalendar, DisplayOffset, DisplayTimeZone,
        OffsetDisambiguation, ResolvedRoundingOptions, RoundingIncrement, TemporalRoundingMode,
        TemporalUnit, ToStringRoundingOptions,
    },
    parsers::{self, IxdtfStringBuilder},
    partial::{PartialDate, PartialTime},
    rounding::{IncrementRounder, Round},
    temporal_assert, Calendar, Duration, Instant, PlainDate, PlainDateTime, PlainTime, Sign,
    TemporalError, TemporalResult, TimeZone,
};

#[cfg(feature = "experimental")]
use crate::components::timezone::TZ_PROVIDER;

/// A struct representing a partial `ZonedDateTime`.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct PartialZonedDateTime {
    /// The `PartialDate` portion of a `PartialZonedDateTime`
    pub date: PartialDate,
    /// The `PartialTime` portion of a `PartialZonedDateTime`
    pub time: PartialTime,
    /// An optional offset string
    pub offset: Option<String>,
    /// The time zone value of a partial time zone.
    pub timezone: Option<TimeZone>,
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
        provider: &impl TimeZoneProvider,
    ) -> TemporalResult<Instant> {
        // 1. If DateDurationSign(duration.[[Date]]) = 0, then
        if duration.date().sign() == Sign::Zero {
            // a. Return ? AddInstant(epochNanoseconds, duration.[[Time]]).
            return self.instant.add_to_instant(duration.time());
        }
        // 2. Let isoDateTime be GetISODateTimeFor(timeZone, epochNanoseconds).
        let iso_datetime = self.tz.get_iso_datetime_for(&self.instant, provider)?;
        // 3. Let addedDate be ? CalendarDateAdd(calendar, isoDateTime.[[ISODate]], duration.[[Date]], overflow).
        let added_date = self
            .calendar()
            .date_add(&iso_datetime.date, duration, overflow)?;
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

    /// Adds a duration to the current `ZonedDateTime`, returning the resulting `ZonedDateTime`.
    ///
    /// Aligns with Abstract Operation 6.5.10
    #[inline]
    pub(crate) fn add_internal(
        &self,
        duration: &Duration,
        overflow: ArithmeticOverflow,
        provider: &impl TimeZoneProvider,
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

    /// Internal representation of Abstract Op 6.5.7
    pub(crate) fn diff_with_rounding(
        &self,
        other: &Self,
        resolved_options: ResolvedRoundingOptions,
        provider: &impl TimeZoneProvider,
    ) -> TemporalResult<NormalizedDurationRecord> {
        // 1. If TemporalUnitCategory(largestUnit) is time, then
        if resolved_options.largest_unit.is_time_unit() {
            // a. Return DifferenceInstant(ns1, ns2, roundingIncrement, smallestUnit, roundingMode).
            return self
                .instant
                .diff_instant_internal(&other.instant, resolved_options);
        }
        // 2. let difference be ? differencezoneddatetime(ns1, ns2, timezone, calendar, largestunit).
        let diff = self.diff_zoned_datetime(other, resolved_options.largest_unit, provider)?;
        // 3. if smallestunit is nanosecond and roundingincrement = 1, return difference.
        if resolved_options.smallest_unit == TemporalUnit::Nanosecond
            && resolved_options.increment == RoundingIncrement::ONE
        {
            return Ok(diff);
        }
        // 4. let datetime be getisodatetimefor(timezone, ns1).
        let iso = self
            .timezone()
            .get_iso_datetime_for(&self.instant, provider)?;
        // 5. Return ? RoundRelativeDuration(difference, ns2, dateTime, timeZone, calendar, largestUnit, roundingIncrement, smallestUnit, roundingMode).
        diff.round_relative_duration(
            other.epoch_nanoseconds(),
            &PlainDateTime::new_unchecked(iso, self.calendar().clone()),
            Some((self.timezone(), provider)),
            resolved_options,
        )
    }

    pub(crate) fn diff_zoned_datetime(
        &self,
        other: &Self,
        largest_unit: TemporalUnit,
        provider: &impl TimeZoneProvider,
    ) -> TemporalResult<NormalizedDurationRecord> {
        // 1. If ns1 = ns2, return CombineDateAndTimeDuration(ZeroDateDuration(), 0).
        if self.epoch_nanoseconds() == other.epoch_nanoseconds() {
            return Ok(NormalizedDurationRecord::default());
        }
        // 2. Let startDateTime be GetISODateTimeFor(timeZone, ns1).
        let start = self.tz.get_iso_datetime_for(&self.instant, provider)?;
        // 3. Let endDateTime be GetISODateTimeFor(timeZone, ns2).
        let end = self.tz.get_iso_datetime_for(&other.instant, provider)?;
        // 4. If ns2 - ns1 < 0, let sign be -1; else let sign be 1.
        let sign = if other.epoch_nanoseconds() - self.epoch_nanoseconds() < 0 {
            Sign::Negative
        } else {
            Sign::Positive
        };
        // 5. If sign = 1, let maxDayCorrection be 2; else let maxDayCorrection be 1.
        let max_correction = if sign == Sign::Positive { 2 } else { 1 };
        // 6. Let dayCorrection be 0.
        // 7. Let timeDuration be DifferenceTime(startDateTime.[[Time]], endDateTime.[[Time]]).
        let time = start.time.diff(&end.time);
        // 8. If TimeDurationSign(timeDuration) = -sign, set dayCorrection to dayCorrection + 1.
        let mut day_correction = if time.sign() as i8 == -(sign as i8) {
            1
        } else {
            0
        };

        // 9. Let success be false.
        let mut intermediate_dt = IsoDateTime::default();
        let mut time_duration = NormalizedTimeDuration::default();
        let mut is_success = false;
        // 10. Repeat, while dayCorrection ≤ maxDayCorrection and success is false,
        while day_correction <= max_correction && !is_success {
            // a. Let intermediateDate be BalanceISODate(endDateTime.[[ISODate]].[[Year]], endDateTime.[[ISODate]].[[Month]], endDateTime.[[ISODate]].[[Day]] - dayCorrection × sign).
            let intermediate = IsoDate::balance(
                end.date.year,
                end.date.month.into(),
                i32::from(end.date.day) - i32::from(day_correction * sign as i8),
            );
            // b. Let intermediateDateTime be CombineISODateAndTimeRecord(intermediateDate, startDateTime.[[Time]]).
            intermediate_dt = IsoDateTime::new_unchecked(intermediate, start.time);
            // c. Let intermediateNs be ? GetEpochNanosecondsFor(timeZone, intermediateDateTime, compatible).
            let intermediate_ns = self.tz.get_epoch_nanoseconds_for(
                intermediate_dt,
                Disambiguation::Compatible,
                provider,
            )?;
            // d. Set timeDuration to TimeDurationFromEpochNanosecondsDifference(ns2, intermediateNs).
            time_duration = NormalizedTimeDuration::from_nanosecond_difference(
                other.epoch_nanoseconds(),
                intermediate_ns.0,
            )?;
            // e. Let timeSign be TimeDurationSign(timeDuration).
            let time_sign = time_duration.sign() as i8;
            // f. If sign ≠ -timeSign, then
            if sign as i8 != -time_sign {
                // i. Set success to true.
                is_success = true;
            }
            // g. Set dayCorrection to dayCorrection + 1.
            day_correction += 1;
        }
        // 11. Assert: success is true.
        // 12. Let dateLargestUnit be LargerOfTwoTemporalUnits(largestUnit, day).
        let date_largest = largest_unit.max(TemporalUnit::Day);
        // 13. Let dateDifference be CalendarDateUntil(calendar, startDateTime.[[ISODate]], intermediateDateTime.[[ISODate]], dateLargestUnit).
        // 14. Return CombineDateAndTimeDuration(dateDifference, timeDuration).
        let date_diff =
            self.calendar()
                .date_until(&start.date, &intermediate_dt.date, date_largest)?;
        NormalizedDurationRecord::new(*date_diff.date(), time_duration)
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
        overflow: Option<ArithmeticOverflow>,
        disambiguation: Option<Disambiguation>,
        offset_option: Option<OffsetDisambiguation>,
        provider: &impl TimeZoneProvider,
    ) -> TemporalResult<Self> {
        let overflow = overflow.unwrap_or(ArithmeticOverflow::Constrain);
        let disambiguation = disambiguation.unwrap_or(Disambiguation::Compatible);
        let offset_option = offset_option.unwrap_or(OffsetDisambiguation::Reject);

        let date = partial
            .date
            .calendar
            .date_from_partial(&partial.date, overflow)?
            .iso;
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

        let timezone = partial.timezone.unwrap_or_default();

        let epoch_nanos = interpret_isodatetime_offset(
            date,
            time,
            false,
            offset_nanos,
            &timezone,
            disambiguation,
            offset_option,
            true,
            provider,
        )?;

        Ok(Self::new_unchecked(
            Instant::from(epoch_nanos),
            partial.date.calendar.clone(),
            timezone,
        ))
    }

    /// Returns the `epochMilliseconds` value of this `ZonedDateTime`.
    #[must_use]
    pub fn epoch_milliseconds(&self) -> i64 {
        self.instant.epoch_milliseconds()
    }

    /// Returns the `epochNanoseconds` value of this `ZonedDateTime`.
    #[must_use]
    pub fn epoch_nanoseconds(&self) -> i128 {
        self.instant.epoch_nanoseconds()
    }

    /// Returns the current `ZonedDateTime` as an [`Instant`].
    #[must_use]
    pub fn to_instant(&self) -> Instant {
        self.instant
    }

    /// Creates a new `ZonedDateTime` from the current `ZonedDateTime`
    /// combined with the provided `TimeZone`.
    pub fn with_timezone(&self, timezone: TimeZone) -> TemporalResult<Self> {
        Self::try_new(self.epoch_nanoseconds(), self.calendar.clone(), timezone)
    }

    /// Creates a new `ZonedDateTime` from the current `ZonedDateTime`
    /// combined with the provided `Calendar`.
    pub fn with_calendar(&self, calendar: Calendar) -> TemporalResult<Self> {
        Self::try_new(self.epoch_nanoseconds(), calendar, self.tz.clone())
    }
}

// ===== Experimental TZ_PROVIDER accessor implementations =====

#[cfg(feature = "experimental")]
impl ZonedDateTime {
    pub fn year(&self) -> TemporalResult<i32> {
        let provider = TZ_PROVIDER
            .lock()
            .map_err(|_| TemporalError::general("Unable to acquire lock"))?;
        self.year_with_provider(&*provider)
    }

    pub fn month(&self) -> TemporalResult<u8> {
        let provider = TZ_PROVIDER
            .lock()
            .map_err(|_| TemporalError::general("Unable to acquire lock"))?;
        self.month_with_provider(&*provider)
    }

    pub fn month_code(&self) -> TemporalResult<TinyAsciiStr<4>> {
        let provider = TZ_PROVIDER
            .lock()
            .map_err(|_| TemporalError::general("Unable to acquire lock"))?;
        self.month_code_with_provider(&*provider)
    }

    pub fn day(&self) -> TemporalResult<u8> {
        let provider = TZ_PROVIDER
            .lock()
            .map_err(|_| TemporalError::general("Unable to acquire lock"))?;
        self.day_with_provider(&*provider)
    }

    pub fn hour(&self) -> TemporalResult<u8> {
        let provider = TZ_PROVIDER
            .lock()
            .map_err(|_| TemporalError::general("Unable to acquire lock"))?;
        self.hour_with_provider(&*provider)
    }

    pub fn minute(&self) -> TemporalResult<u8> {
        let provider = TZ_PROVIDER
            .lock()
            .map_err(|_| TemporalError::general("Unable to acquire lock"))?;
        self.minute_with_provider(&*provider)
    }

    pub fn second(&self) -> TemporalResult<u8> {
        let provider = TZ_PROVIDER
            .lock()
            .map_err(|_| TemporalError::general("Unable to acquire lock"))?;
        self.second_with_provider(&*provider)
    }

    pub fn millisecond(&self) -> TemporalResult<u16> {
        let provider = TZ_PROVIDER
            .lock()
            .map_err(|_| TemporalError::general("Unable to acquire lock"))?;
        self.millisecond_with_provider(&*provider)
    }

    pub fn microsecond(&self) -> TemporalResult<u16> {
        let provider = TZ_PROVIDER
            .lock()
            .map_err(|_| TemporalError::general("Unable to acquire lock"))?;
        self.millisecond_with_provider(&*provider)
    }

    pub fn nanosecond(&self) -> TemporalResult<u16> {
        let provider = TZ_PROVIDER
            .lock()
            .map_err(|_| TemporalError::general("Unable to acquire lock"))?;

        self.millisecond_with_provider(&*provider)
    }
}

// ==== Experimental TZ_PROVIDER calendar method implementations ====

#[cfg(feature = "experimental")]
impl ZonedDateTime {
    pub fn era(&self) -> TemporalResult<Option<TinyAsciiStr<16>>> {
        let provider = TZ_PROVIDER
            .lock()
            .map_err(|_| TemporalError::general("Unable to acquire lock"))?;
        self.era_with_provider(&*provider)
    }

    pub fn era_year(&self) -> TemporalResult<Option<i32>> {
        let provider = TZ_PROVIDER
            .lock()
            .map_err(|_| TemporalError::general("Unable to acquire lock"))?;
        self.era_year_with_provider(&*provider)
    }

    /// Returns the calendar day of week value.
    pub fn day_of_week(&self) -> TemporalResult<u16> {
        let provider = TZ_PROVIDER
            .lock()
            .map_err(|_| TemporalError::general("Unable to acquire lock"))?;
        self.day_of_week_with_provider(&*provider)
    }

    /// Returns the calendar day of year value.
    pub fn day_of_year(&self) -> TemporalResult<u16> {
        let provider = TZ_PROVIDER
            .lock()
            .map_err(|_| TemporalError::general("Unable to acquire lock"))?;
        self.day_of_year_with_provider(&*provider)
    }

    /// Returns the calendar week of year value.
    pub fn week_of_year(&self) -> TemporalResult<Option<u16>> {
        let provider = TZ_PROVIDER
            .lock()
            .map_err(|_| TemporalError::general("Unable to acquire lock"))?;
        self.week_of_year_with_provider(&*provider)
    }

    /// Returns the calendar year of week value.
    pub fn year_of_week(&self) -> TemporalResult<Option<i32>> {
        let provider = TZ_PROVIDER
            .lock()
            .map_err(|_| TemporalError::general("Unable to acquire lock"))?;
        self.year_of_week_with_provider(&*provider)
    }

    /// Returns the calendar days in week value.
    pub fn days_in_week(&self) -> TemporalResult<u16> {
        let provider = TZ_PROVIDER
            .lock()
            .map_err(|_| TemporalError::general("Unable to acquire lock"))?;
        self.days_in_week_with_provider(&*provider)
    }

    /// Returns the calendar days in month value.
    pub fn days_in_month(&self) -> TemporalResult<u16> {
        let provider = TZ_PROVIDER
            .lock()
            .map_err(|_| TemporalError::general("Unable to acquire lock"))?;
        self.days_in_month_with_provider(&*provider)
    }

    /// Returns the calendar days in year value.
    pub fn days_in_year(&self) -> TemporalResult<u16> {
        let provider = TZ_PROVIDER
            .lock()
            .map_err(|_| TemporalError::general("Unable to acquire lock"))?;
        self.days_in_year_with_provider(&*provider)
    }

    /// Returns the calendar months in year value.
    pub fn months_in_year(&self) -> TemporalResult<u16> {
        let provider = TZ_PROVIDER
            .lock()
            .map_err(|_| TemporalError::general("Unable to acquire lock"))?;
        self.months_in_year_with_provider(&*provider)
    }

    /// Returns returns whether the date in a leap year for the given calendar.
    pub fn in_leap_year(&self) -> TemporalResult<bool> {
        let provider = TZ_PROVIDER
            .lock()
            .map_err(|_| TemporalError::general("Unable to acquire lock"))?;
        self.in_leap_year_with_provider(&*provider)
    }
}

// ==== Experimental TZ_PROVIDER method implementations ====

#[cfg(feature = "experimental")]
impl ZonedDateTime {
    /// Creates a new `ZonedDateTime` from the current `ZonedDateTime`
    /// combined with the provided `TimeZone`.
    pub fn with_plain_time(&self, time: PlainTime) -> TemporalResult<Self> {
        let provider = TZ_PROVIDER
            .lock()
            .map_err(|_| TemporalError::general("Unable to acquire lock"))?;
        self.with_plain_time_and_provider(time, &*provider)
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
            &*provider,
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
            &*provider,
        )
    }

    pub fn start_of_day(&self) -> TemporalResult<Self> {
        let provider = TZ_PROVIDER
            .lock()
            .map_err(|_| TemporalError::general("Unable to acquire lock"))?;
        self.start_of_day_with_provider(&*provider)
    }

    pub fn to_plain_date(&self) -> TemporalResult<PlainDate> {
        let provider = TZ_PROVIDER
            .lock()
            .map_err(|_| TemporalError::general("Unable to acquire lock"))?;
        self.to_plain_date_with_provider(&*provider)
    }

    pub fn to_plain_time(&self) -> TemporalResult<PlainTime> {
        let provider = TZ_PROVIDER
            .lock()
            .map_err(|_| TemporalError::general("Unable to acquire lock"))?;
        self.to_plain_time_with_provider(&*provider)
    }

    pub fn to_plain_datetime(&self) -> TemporalResult<PlainDateTime> {
        let provider = TZ_PROVIDER
            .lock()
            .map_err(|_| TemporalError::general("Unable to acquire lock"))?;
        self.to_plain_datetime_with_provider(&*provider)
    }

    pub fn from_str(
        source: &str,
        disambiguation: Disambiguation,
        offset_option: OffsetDisambiguation,
    ) -> TemporalResult<Self> {
        let provider = TZ_PROVIDER
            .lock()
            .map_err(|_| TemporalError::general("Unable to acquire lock"))?;
        Self::from_str_with_provider(source, disambiguation, offset_option, &*provider)
    }
}

// ==== HoursInDay accessor method implementation ====

impl ZonedDateTime {
    #[cfg(feature = "experimental")]
    pub fn hours_in_day(&self) -> TemporalResult<u8> {
        let provider = TZ_PROVIDER
            .lock()
            .map_err(|_| TemporalError::general("Unable to acquire lock"))?;
        self.hours_in_day_with_provider(&*provider)
    }

    pub fn hours_in_day_with_provider(
        &self,
        provider: &impl TimeZoneProvider,
    ) -> TemporalResult<u8> {
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
    pub fn year_with_provider(&self, provider: &impl TimeZoneProvider) -> TemporalResult<i32> {
        let iso = self.tz.get_iso_datetime_for(&self.instant, provider)?;
        let dt = PlainDateTime::new_unchecked(iso, self.calendar.clone());
        self.calendar.year(&dt.iso.date)
    }

    /// Returns the `month` value for this `ZonedDateTime`.
    pub fn month_with_provider(&self, provider: &impl TimeZoneProvider) -> TemporalResult<u8> {
        let iso = self.tz.get_iso_datetime_for(&self.instant, provider)?;
        let dt = PlainDateTime::new_unchecked(iso, self.calendar.clone());
        self.calendar.month(&dt.iso.date)
    }

    /// Returns the `monthCode` value for this `ZonedDateTime`.
    pub fn month_code_with_provider(
        &self,
        provider: &impl TimeZoneProvider,
    ) -> TemporalResult<TinyAsciiStr<4>> {
        let iso = self.tz.get_iso_datetime_for(&self.instant, provider)?;
        let dt = PlainDateTime::new_unchecked(iso, self.calendar.clone());
        self.calendar.month_code(&dt.iso.date)
    }

    /// Returns the `day` value for this `ZonedDateTime`.
    pub fn day_with_provider(&self, provider: &impl TimeZoneProvider) -> TemporalResult<u8> {
        let iso = self.tz.get_iso_datetime_for(&self.instant, provider)?;
        let dt = PlainDateTime::new_unchecked(iso, self.calendar.clone());
        self.calendar.day(&dt.iso.date)
    }

    /// Returns the `hour` value for this `ZonedDateTime`.
    pub fn hour_with_provider(&self, provider: &impl TimeZoneProvider) -> TemporalResult<u8> {
        let iso = self.tz.get_iso_datetime_for(&self.instant, provider)?;
        Ok(iso.time.hour)
    }

    /// Returns the `minute` value for this `ZonedDateTime`.
    pub fn minute_with_provider(&self, provider: &impl TimeZoneProvider) -> TemporalResult<u8> {
        let iso = self.tz.get_iso_datetime_for(&self.instant, provider)?;
        Ok(iso.time.minute)
    }

    /// Returns the `second` value for this `ZonedDateTime`.
    pub fn second_with_provider(&self, provider: &impl TimeZoneProvider) -> TemporalResult<u8> {
        let iso = self.tz.get_iso_datetime_for(&self.instant, provider)?;
        Ok(iso.time.second)
    }

    /// Returns the `millisecond` value for this `ZonedDateTime`.
    pub fn millisecond_with_provider(
        &self,
        provider: &impl TimeZoneProvider,
    ) -> TemporalResult<u16> {
        let iso = self.tz.get_iso_datetime_for(&self.instant, provider)?;
        Ok(iso.time.millisecond)
    }

    /// Returns the `microsecond` value for this `ZonedDateTime`.
    pub fn microsecond_with_provider(
        &self,
        provider: &impl TimeZoneProvider,
    ) -> TemporalResult<u16> {
        let iso = self.tz.get_iso_datetime_for(&self.instant, provider)?;
        Ok(iso.time.millisecond)
    }

    /// Returns the `nanosecond` value for this `ZonedDateTime`.
    pub fn nanosecond_with_provider(
        &self,
        provider: &impl TimeZoneProvider,
    ) -> TemporalResult<u16> {
        let iso = self.tz.get_iso_datetime_for(&self.instant, provider)?;
        Ok(iso.time.nanosecond)
    }
}

// ==== Core calendar method implementations ====

impl ZonedDateTime {
    pub fn era_with_provider(
        &self,
        provider: &impl TimeZoneProvider,
    ) -> TemporalResult<Option<TinyAsciiStr<16>>> {
        let iso = self.tz.get_iso_datetime_for(&self.instant, provider)?;
        let pdt = PlainDateTime::new_unchecked(iso, self.calendar.clone());
        self.calendar.era(&pdt.iso.date)
    }

    pub fn era_year_with_provider(
        &self,
        provider: &impl TimeZoneProvider,
    ) -> TemporalResult<Option<i32>> {
        let iso = self.tz.get_iso_datetime_for(&self.instant, provider)?;
        let pdt = PlainDateTime::new_unchecked(iso, self.calendar.clone());
        self.calendar.era_year(&pdt.iso.date)
    }

    /// Returns the calendar day of week value.
    pub fn day_of_week_with_provider(
        &self,
        provider: &impl TimeZoneProvider,
    ) -> TemporalResult<u16> {
        let iso = self.tz.get_iso_datetime_for(&self.instant, provider)?;
        let pdt = PlainDateTime::new_unchecked(iso, self.calendar.clone());
        self.calendar.day_of_week(&pdt.iso.date)
    }

    /// Returns the calendar day of year value.
    pub fn day_of_year_with_provider(
        &self,
        provider: &impl TimeZoneProvider,
    ) -> TemporalResult<u16> {
        let iso = self.tz.get_iso_datetime_for(&self.instant, provider)?;
        let pdt = PlainDateTime::new_unchecked(iso, self.calendar.clone());
        self.calendar.day_of_year(&pdt.iso.date)
    }

    /// Returns the calendar week of year value.
    pub fn week_of_year_with_provider(
        &self,
        provider: &impl TimeZoneProvider,
    ) -> TemporalResult<Option<u16>> {
        let iso = self.tz.get_iso_datetime_for(&self.instant, provider)?;
        let pdt = PlainDateTime::new_unchecked(iso, self.calendar.clone());
        self.calendar.week_of_year(&pdt.iso.date)
    }

    /// Returns the calendar year of week value.
    pub fn year_of_week_with_provider(
        &self,
        provider: &impl TimeZoneProvider,
    ) -> TemporalResult<Option<i32>> {
        let iso = self.tz.get_iso_datetime_for(&self.instant, provider)?;
        let pdt = PlainDateTime::new_unchecked(iso, self.calendar.clone());
        self.calendar.year_of_week(&pdt.iso.date)
    }

    /// Returns the calendar days in week value.
    pub fn days_in_week_with_provider(
        &self,
        provider: &impl TimeZoneProvider,
    ) -> TemporalResult<u16> {
        let iso = self.tz.get_iso_datetime_for(&self.instant, provider)?;
        let pdt = PlainDateTime::new_unchecked(iso, self.calendar.clone());
        self.calendar.days_in_week(&pdt.iso.date)
    }

    /// Returns the calendar days in month value.
    pub fn days_in_month_with_provider(
        &self,
        provider: &impl TimeZoneProvider,
    ) -> TemporalResult<u16> {
        let iso = self.tz.get_iso_datetime_for(&self.instant, provider)?;
        let pdt = PlainDateTime::new_unchecked(iso, self.calendar.clone());
        self.calendar.days_in_month(&pdt.iso.date)
    }

    /// Returns the calendar days in year value.
    pub fn days_in_year_with_provider(
        &self,
        provider: &impl TimeZoneProvider,
    ) -> TemporalResult<u16> {
        let iso = self.tz.get_iso_datetime_for(&self.instant, provider)?;
        let pdt = PlainDateTime::new_unchecked(iso, self.calendar.clone());
        self.calendar.days_in_year(&pdt.iso.date)
    }

    /// Returns the calendar months in year value.
    pub fn months_in_year_with_provider(
        &self,
        provider: &impl TimeZoneProvider,
    ) -> TemporalResult<u16> {
        let iso = self.tz.get_iso_datetime_for(&self.instant, provider)?;
        let pdt = PlainDateTime::new_unchecked(iso, self.calendar.clone());
        self.calendar.months_in_year(&pdt.iso.date)
    }

    /// Returns returns whether the date in a leap year for the given calendar.
    pub fn in_leap_year_with_provider(
        &self,
        provider: &impl TimeZoneProvider,
    ) -> TemporalResult<bool> {
        let iso = self.tz.get_iso_datetime_for(&self.instant, provider)?;
        let pdt = PlainDateTime::new_unchecked(iso, self.calendar.clone());
        self.calendar.in_leap_year(&pdt.iso.date)
    }
}

// ==== Core method implementations ====

impl ZonedDateTime {
    /// Creates a new `ZonedDateTime` from the current `ZonedDateTime`
    /// combined with the provided `TimeZone`.
    pub fn with_plain_time_and_provider(
        &self,
        time: PlainTime,
        provider: &impl TimeZoneProvider,
    ) -> TemporalResult<Self> {
        let iso = self.tz.get_iso_datetime_for(&self.instant, provider)?;
        let result_iso = IsoDateTime::new_unchecked(iso.date, time.iso);
        let epoch_ns =
            self.tz
                .get_epoch_nanoseconds_for(result_iso, Disambiguation::Compatible, provider)?;
        Self::try_new(epoch_ns.0, self.calendar.clone(), self.tz.clone())
    }

    /// Add a duration to the current `ZonedDateTime`
    pub fn add_with_provider(
        &self,
        duration: &Duration,
        overflow: Option<ArithmeticOverflow>,
        provider: &impl TimeZoneProvider,
    ) -> TemporalResult<Self> {
        self.add_internal(
            duration,
            overflow.unwrap_or(ArithmeticOverflow::Constrain),
            provider,
        )
    }

    /// Subtract a duration to the current `ZonedDateTime`
    pub fn subtract_with_provider(
        &self,
        duration: &Duration,
        overflow: Option<ArithmeticOverflow>,
        provider: &impl TimeZoneProvider,
    ) -> TemporalResult<Self> {
        self.add_internal(
            &duration.negated(),
            overflow.unwrap_or(ArithmeticOverflow::Constrain),
            provider,
        )
    }

    /// Return a `ZonedDateTime` representing the start of the day
    /// for the current `ZonedDateTime`.
    pub fn start_of_day_with_provider(
        &self,
        provider: &impl TimeZoneProvider,
    ) -> TemporalResult<Self> {
        let iso = self.tz.get_iso_datetime_for(&self.instant, provider)?;
        let epoch_nanos = self.tz.get_start_of_day(&iso.date, provider)?;
        Self::try_new(epoch_nanos.0, self.calendar.clone(), self.tz.clone())
    }

    /// Convert the current `ZonedDateTime` to a [`PlainDate`] with
    /// a user defined time zone provider.
    pub fn to_plain_date_with_provider(
        &self,
        provider: &impl TimeZoneProvider,
    ) -> TemporalResult<PlainDate> {
        let iso = self.tz.get_iso_datetime_for(&self.instant, provider)?;
        Ok(PlainDate::new_unchecked(iso.date, self.calendar.clone()))
    }

    /// Convert the current `ZonedDateTime` to a [`PlainTime`] with
    /// a user defined time zone provider.
    pub fn to_plain_time_with_provider(
        &self,
        provider: &impl TimeZoneProvider,
    ) -> TemporalResult<PlainTime> {
        let iso = self.tz.get_iso_datetime_for(&self.instant, provider)?;
        Ok(PlainTime::new_unchecked(iso.time))
    }

    /// Convert the current `ZonedDateTime` to a [`PlainDateTime`] with
    /// a user defined time zone provider.
    pub fn to_plain_datetime_with_provider(
        &self,
        provider: &impl TimeZoneProvider,
    ) -> TemporalResult<PlainDateTime> {
        let iso = self.tz.get_iso_datetime_for(&self.instant, provider)?;
        Ok(PlainDateTime::new_unchecked(iso, self.calendar.clone()))
    }

    /// Creates a default formatted IXDTF (RFC 9557) date/time string for the provided `ZonedDateTime`.
    pub fn to_string_with_provider(
        &self,
        provider: &impl TimeZoneProvider,
    ) -> TemporalResult<String> {
        self.to_ixdtf_string_with_provider(
            DisplayOffset::Auto,
            DisplayTimeZone::Auto,
            DisplayCalendar::Auto,
            ToStringRoundingOptions::default(),
            provider,
        )
    }

    /// Creates an IXDTF (RFC 9557) date/time string for the provided `ZonedDateTime` according
    /// to the provided display options.
    pub fn to_ixdtf_string_with_provider(
        &self,
        display_offset: DisplayOffset,
        display_timezone: DisplayTimeZone,
        display_calendar: DisplayCalendar,
        options: ToStringRoundingOptions,
        provider: &impl TimeZoneProvider,
    ) -> TemporalResult<String> {
        let resolved_options = options.resolve()?;
        let result =
            self.instant
                .round_instant(ResolvedRoundingOptions::from_to_string_options(
                    &resolved_options,
                ))?;

        let offset = self.tz.get_offset_nanos_for(result, provider)?;
        let datetime = self.tz.get_iso_datetime_for(&self.instant, provider)?;
        let (sign, hour, minute) = nanoseconds_to_formattable_offset_minutes(offset)?;
        let timezone_id = self.timezone().identifier()?;

        let ixdtf_string = IxdtfStringBuilder::default()
            .with_date(datetime.date)
            .with_time(datetime.time, resolved_options.precision)
            .with_minute_offset(sign, hour, minute, display_offset)
            .with_timezone(&timezone_id, display_timezone)
            .with_calendar(self.calendar.identifier(), display_calendar)
            .build();

        Ok(ixdtf_string)
    }

    // TODO: Should IANA Identifier be prechecked or allow potentially invalid IANA Identifer values here?
    pub fn from_str_with_provider(
        source: &str,
        disambiguation: Disambiguation,
        offset_option: OffsetDisambiguation,
        provider: &impl TimeZoneProvider,
    ) -> TemporalResult<Self> {
        let parse_result = parsers::parse_date_time(source)?;

        let Some(annotation) = parse_result.tz else {
            return Err(TemporalError::r#type()
                .with_message("Time zone annotation is required for ZonedDateTime string."));
        };

        let timezone = match annotation.tz {
            TimeZoneRecord::Name(s) => {
                TimeZone::IanaIdentifier(String::from_utf8_lossy(s).into_owned())
            }
            TimeZoneRecord::Offset(offset_record) => {
                // NOTE: ixdtf parser restricts minute/second to 0..=60
                let minutes = i16::from(offset_record.hour) * 60 + offset_record.minute as i16;
                TimeZone::OffsetMinutes(minutes * i16::from(offset_record.sign as i8))
            }
            // TimeZoneRecord is non_exhaustive, but all current branches are matching.
            _ => return Err(TemporalError::assert()),
        };

        let (offset_nanos, is_exact) = parse_result
            .offset
            .map(|record| {
                let UtcOffsetRecordOrZ::Offset(offset) = record else {
                    return (None, true);
                };
                let hours_in_ns = i64::from(offset.hour) * 3_600_000_000_000_i64;
                let minutes_in_ns = i64::from(offset.minute) * 60_000_000_000_i64;
                let seconds_in_ns = i64::from(offset.minute) * 1_000_000_000_i64;
                (
                    Some(
                        (hours_in_ns
                            + minutes_in_ns
                            + seconds_in_ns
                            + i64::from(offset.nanosecond))
                            * i64::from(offset.sign as i8),
                    ),
                    false,
                )
            })
            .unwrap_or((None, false));

        let calendar = parse_result
            .calendar
            .map(Calendar::from_utf8)
            .transpose()?
            .unwrap_or_default();

        let time = parse_result
            .time
            .map(|time| {
                IsoTime::from_components(time.hour, time.minute, time.second, time.nanosecond)
            })
            .transpose()?;

        let Some(parsed_date) = parse_result.date else {
            return Err(
                TemporalError::range().with_message("No valid DateRecord Parse Node was found.")
            );
        };

        let date = IsoDate::new_with_overflow(
            parsed_date.year,
            parsed_date.month,
            parsed_date.day,
            ArithmeticOverflow::Reject,
        )?;

        let epoch_nanos = interpret_isodatetime_offset(
            date,
            time,
            is_exact,
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
pub(crate) fn interpret_isodatetime_offset(
    date: IsoDate,
    time: Option<IsoTime>,
    is_exact: bool,
    offset_nanos: Option<i64>,
    timezone: &TimeZone,
    disambiguation: Disambiguation,
    offset_option: OffsetDisambiguation,
    match_minutes: bool,
    provider: &impl TimeZoneProvider,
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
    match (is_exact, offset_nanos) {
        // 4. If offsetBehaviour is exact, or offsetBehaviour is option and offsetOption is use, then
        (true, Some(offset)) if offset_option == OffsetDisambiguation::Use => {
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
        (_, Some(offset))
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
                    let rounded_candidate = IncrementRounder::from_signed_num(
                        candidate_offset,
                        NonZeroU128::new(60_000_000_000).expect("cannot be zero"), // TODO: Add back const { } after MSRV can be bumped
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

// Formatting utils
const NS_PER_MINUTE: i128 = 60_000_000_000;

pub(crate) fn nanoseconds_to_formattable_offset_minutes(
    nanoseconds: i128,
) -> TemporalResult<(Sign, u8, u8)> {
    // Per 11.1.7 this should be rounding
    let nanoseconds = IncrementRounder::from_signed_num(nanoseconds, unsafe {
        NonZeroU128::new_unchecked(NS_PER_MINUTE as u128)
    })?
    .round(TemporalRoundingMode::HalfExpand);
    let offset_minutes = (nanoseconds / NS_PER_MINUTE) as i32;
    let sign = if offset_minutes < 0 {
        Sign::Negative
    } else {
        Sign::Positive
    };
    let hour = offset_minutes.abs() / 60;
    let minute = offset_minutes.abs() % 60;
    Ok((sign, hour as u8, minute as u8))
}

#[cfg(all(test, feature = "tzdb"))]
mod tests {
    use crate::{
        options::{DifferenceSettings, Disambiguation, OffsetDisambiguation, TemporalUnit},
        partial::{PartialDate, PartialTime, PartialZonedDateTime},
        primitive::FiniteF64,
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
            timezone: Some(TimeZone::default()),
        };

        let result = ZonedDateTime::from_partial_with_provider(partial, None, None, None, provider);
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

    #[test]
    // https://github.com/tc39/test262/blob/d9b10790bc4bb5b3e1aa895f11cbd2d31a5ec743/test/intl402/Temporal/ZonedDateTime/from/dst-skipped-cross-midnight.js
    fn dst_skipped_cross_midnight() {
        let provider = &FsTzdbProvider::default();
        let start_of_day = ZonedDateTime::from_str_with_provider(
            "1919-03-31[America/Toronto]",
            Disambiguation::Compatible,
            OffsetDisambiguation::Reject,
            provider,
        )
        .unwrap();
        let midnight_disambiguated = ZonedDateTime::from_str_with_provider(
            "1919-03-31T00[America/Toronto]",
            Disambiguation::Compatible,
            OffsetDisambiguation::Reject,
            provider,
        )
        .unwrap();

        assert_eq!(start_of_day.epoch_nanoseconds(), -1601753400000000000);
        assert_eq!(
            midnight_disambiguated.epoch_nanoseconds(),
            -1601751600000000000
        );
        let diff = start_of_day
            .instant
            .until(
                &midnight_disambiguated.instant,
                DifferenceSettings {
                    largest_unit: Some(TemporalUnit::Year),
                    smallest_unit: Some(TemporalUnit::Nanosecond),
                    ..Default::default()
                },
            )
            .unwrap();
        let zero = FiniteF64::from(0);
        assert_eq!(diff.years(), zero);
        assert_eq!(diff.months(), zero);
        assert_eq!(diff.weeks(), zero);
        assert_eq!(diff.days(), zero);
        assert_eq!(diff.hours(), zero);
        assert_eq!(diff.minutes(), FiniteF64::from(30));
        assert_eq!(diff.seconds(), zero);
        assert_eq!(diff.milliseconds(), zero);
        assert_eq!(diff.microseconds(), zero);
        assert_eq!(diff.nanoseconds(), zero);
    }
}
