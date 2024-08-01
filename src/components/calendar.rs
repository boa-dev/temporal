//! This module implements the calendar traits and related components.
//!
//! The goal of the calendar module of `boa_temporal` is to provide
//! Temporal compatible calendar implementations.

use std::str::FromStr;

use crate::{
    components::{
        duration::{DateDuration, TimeDuration},
        Date, DateTime, Duration, MonthDay, YearMonth,
    },
    fields::{FieldMap, TemporalFields},
    iso::{IsoDate, IsoDateSlots},
    options::{ArithmeticOverflow, TemporalUnit},
    TemporalError, TemporalResult,
};

use icu_calendar::{
    any_calendar::AnyDateInner,
    buddhist::Buddhist,
    chinese::Chinese,
    coptic::Coptic,
    dangi::Dangi,
    ethiopian::{Ethiopian, EthiopianEraStyle},
    hebrew::Hebrew,
    indian::Indian,
    islamic::{IslamicCivil, IslamicObservational, IslamicTabular, IslamicUmmAlQura},
    japanese::{Japanese, JapaneseExtended},
    persian::Persian,
    roc::Roc,
    types::{DayOfMonth, DayOfYearInfo, Era, FormattableMonth, FormattableYear, MonthCode},
    week::{RelativeUnit, WeekCalculator},
    AnyCalendar, AnyCalendarKind, Calendar as IcuCalendar, DateDuration as IcuDateDuration,
    DateDurationUnit as IcuDateDurationUnit, Gregorian, Iso, Ref,
};
use tinystr::TinyAsciiStr;

use super::ZonedDateTime;

/// The ECMAScript defined protocol methods
pub const CALENDAR_PROTOCOL_METHODS: [&str; 21] = [
    "dateAdd",
    "dateFromFields",
    "dateUntil",
    "day",
    "dayOfWeek",
    "dayOfYear",
    "daysInMonth",
    "daysInWeek",
    "daysInYear",
    "fields",
    "id",
    "inLeapYear",
    "mergeFields",
    "month",
    "monthCode",
    "monthDayFromFields",
    "monthsInYear",
    "weekOfYear",
    "year",
    "yearMonthFromFields",
    "yearOfWeek",
];

#[derive(Debug, Clone)]
pub struct Calendar(Ref<'static, AnyCalendar>);

impl Default for Calendar {
    fn default() -> Self {
        Calendar::new(AnyCalendarKind::Iso)
    }
}

impl PartialEq for Calendar {
    fn eq(&self, other: &Self) -> bool {
        self.identifier() == other.identifier()
    }
}

impl Eq for Calendar {}

impl IcuCalendar for Calendar {
    type DateInner = AnyDateInner;

    fn date_from_codes(
        &self,
        era: icu_calendar::types::Era,
        year: i32,
        month_code: icu_calendar::types::MonthCode,
        day: u8,
    ) -> Result<Self::DateInner, icu_calendar::Error> {
        self.0.date_from_codes(era, year, month_code, day)
    }

    fn date_from_iso(&self, iso: icu_calendar::Date<Iso>) -> Self::DateInner {
        self.0.date_from_iso(iso)
    }

    fn date_to_iso(&self, date: &Self::DateInner) -> icu_calendar::Date<Iso> {
        self.0.date_to_iso(date)
    }

    fn months_in_year(&self, date: &Self::DateInner) -> u8 {
        self.0.months_in_year(date)
    }

    fn days_in_year(&self, date: &Self::DateInner) -> u16 {
        self.0.days_in_year(date)
    }

    fn days_in_month(&self, date: &Self::DateInner) -> u8 {
        self.0.days_in_month(date)
    }

    fn offset_date(&self, date: &mut Self::DateInner, offset: IcuDateDuration<Self>) {
        self.0.offset_date(date, offset.cast_unit())
    }

    fn until(
        &self,
        date1: &Self::DateInner,
        date2: &Self::DateInner,
        calendar2: &Self,
        largest_unit: IcuDateDurationUnit,
        smallest_unit: IcuDateDurationUnit,
    ) -> IcuDateDuration<Self> {
        self.0
            .until(date1, date2, &calendar2.0, largest_unit, smallest_unit)
            .cast_unit()
    }

    fn debug_name(&self) -> &'static str {
        self.0.debug_name()
    }

    fn year(&self, date: &Self::DateInner) -> FormattableYear {
        self.0.year(date)
    }

    fn is_in_leap_year(&self, date: &Self::DateInner) -> bool {
        self.0.is_in_leap_year(date)
    }

    fn month(&self, date: &Self::DateInner) -> FormattableMonth {
        self.0.month(date)
    }

    fn day_of_month(&self, date: &Self::DateInner) -> DayOfMonth {
        self.0.day_of_month(date)
    }

    fn day_of_year_info(&self, date: &Self::DateInner) -> DayOfYearInfo {
        self.0.day_of_year_info(date)
    }
}

impl Calendar {
    #[warn(clippy::wildcard_enum_match_arm)] // Warns if the calendar kind gets out of sync.
    pub fn new(kind: AnyCalendarKind) -> Self {
        let cal = match kind {
            AnyCalendarKind::Buddhist => &AnyCalendar::Buddhist(Buddhist),
            AnyCalendarKind::Chinese => const { &AnyCalendar::Chinese(Chinese::new()) },
            AnyCalendarKind::Coptic => &AnyCalendar::Coptic(Coptic),
            AnyCalendarKind::Dangi => const { &AnyCalendar::Dangi(Dangi::new()) },
            AnyCalendarKind::Ethiopian => {
                const {
                    &AnyCalendar::Ethiopian(Ethiopian::new_with_era_style(
                        EthiopianEraStyle::AmeteMihret,
                    ))
                }
            }
            AnyCalendarKind::EthiopianAmeteAlem => {
                const {
                    &AnyCalendar::Ethiopian(Ethiopian::new_with_era_style(
                        EthiopianEraStyle::AmeteAlem,
                    ))
                }
            }
            AnyCalendarKind::Gregorian => &AnyCalendar::Gregorian(Gregorian),
            AnyCalendarKind::Hebrew => &AnyCalendar::Hebrew(Hebrew),
            AnyCalendarKind::Indian => &AnyCalendar::Indian(Indian),
            AnyCalendarKind::IslamicCivil => &AnyCalendar::IslamicCivil(IslamicCivil),
            AnyCalendarKind::IslamicObservational => {
                const { &AnyCalendar::IslamicObservational(IslamicObservational::new()) }
            }
            AnyCalendarKind::IslamicTabular => &AnyCalendar::IslamicTabular(IslamicTabular),
            AnyCalendarKind::IslamicUmmAlQura => {
                const { &AnyCalendar::IslamicUmmAlQura(IslamicUmmAlQura::new()) }
            }
            AnyCalendarKind::Iso => &AnyCalendar::Iso(Iso),
            AnyCalendarKind::Japanese => const { &AnyCalendar::Japanese(Japanese::new()) },
            AnyCalendarKind::JapaneseExtended => {
                const { &AnyCalendar::JapaneseExtended(JapaneseExtended::new()) }
            }
            AnyCalendarKind::Persian => &AnyCalendar::Persian(Persian),
            AnyCalendarKind::Roc => &AnyCalendar::Roc(Roc),
            _ => unreachable!("match must handle all variants of `AnyCalendarKind`"),
        };

        Self(Ref(cal))
    }
}

impl FromStr for Calendar {
    type Err = TemporalError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // NOTE(nekesss): Catch the iso identifier here, as `iso8601` is not a valid ID below.
        if s == "iso8601" {
            return Ok(Self::default());
        }

        let Some(cal) = AnyCalendarKind::get_for_bcp47_string(s) else {
            return Err(TemporalError::range().with_message("Not a builtin calendar."));
        };

        Ok(Calendar::new(cal))
    }
}

/// Designate the type of `CalendarFields` needed
#[derive(Debug, Clone, Copy)]
pub enum CalendarFieldsType {
    /// Whether the Fields should return for a Date.
    Date,
    /// Whether the Fields should return for a YearMonth.
    YearMonth,
    /// Whether the Fields should return for a MonthDay.
    MonthDay,
}

// TODO: Optimize to TinyStr or &str.
impl From<&[String]> for CalendarFieldsType {
    fn from(value: &[String]) -> Self {
        let year_present = value.contains(&"year".to_owned());
        let day_present = value.contains(&"day".to_owned());

        if year_present && day_present {
            CalendarFieldsType::Date
        } else if year_present {
            CalendarFieldsType::YearMonth
        } else {
            CalendarFieldsType::MonthDay
        }
    }
}

/// The `DateLike` objects that can be provided to the `CalendarProtocol`.
#[derive(Debug)]
pub enum CalendarDateLike {
    /// Represents a `DateTime`.
    DateTime(DateTime),
    /// Represents a `Date`.
    Date(Date),
    /// Represents a `YearMonth`.
    YearMonth(YearMonth),
    /// Represents a `MonthDay`.
    MonthDay(MonthDay),
}

impl CalendarDateLike {
    /// Retrieves the internal `IsoDate` field.
    #[inline]
    #[must_use]
    pub fn as_iso_date(&self) -> IsoDate {
        match self {
            CalendarDateLike::DateTime(dt) => dt.iso_date(),
            CalendarDateLike::Date(d) => d.iso_date(),
            CalendarDateLike::YearMonth(ym) => ym.iso_date(),
            CalendarDateLike::MonthDay(md) => md.iso_date(),
        }
    }
}

/// A trait for retrieving an internal calendar slice.
pub trait GetTemporalCalendar {
    /// Returns the `TemporalCalendar` value of the implementor.
    fn get_calendar(&self) -> Calendar;
}

// ==== Public `CalendarSlot` methods ====

impl Calendar {
    /// Returns whether the current calendar is `ISO`
    pub fn is_iso(&self) -> bool {
        matches!(self.0 .0, AnyCalendar::Iso(_))
    }

    /// `CalendarDateFromFields`
    pub fn date_from_fields(
        &self,
        fields: &mut TemporalFields,
        overflow: ArithmeticOverflow,
    ) -> TemporalResult<Date> {
        if self.is_iso() {
            // Resolve month and monthCode;
            fields.iso_resolve_month()?;
            return Date::new(
                fields.year.unwrap_or(0),
                fields.month.unwrap_or(0),
                fields.day.unwrap_or(0),
                self.clone(),
                overflow,
            );
        }

        let era = Era::from_str(&fields.era.map_or(String::default(), |s| s.to_string()))
            .map_err(|e| TemporalError::general(format!("{e:?}")))?;
        let month_code = MonthCode(
            fields
                .month_code
                .map(|mc| {
                    TinyAsciiStr::from_bytes(mc.as_str().as_bytes())
                        .expect("MonthCode as_str is always valid.")
                })
                .ok_or(TemporalError::range().with_message("No MonthCode provided."))?,
        );
        // NOTE: This might preemptively throw as `ICU4X` does not support constraining.
        // Resolve month and monthCode;
        let calendar_date = self.0.date_from_codes(
            era,
            fields.year.unwrap_or(0),
            month_code,
            fields.day.unwrap_or(0) as u8,
        )?;
        let iso = self.0.date_to_iso(&calendar_date);
        Date::new(
            iso.year().number,
            iso.month().ordinal as i32,
            iso.day_of_month().0 as i32,
            self.clone(),
            overflow,
        )
    }

    /// `CalendarMonthDayFromFields`
    pub fn month_day_from_fields(
        &self,
        fields: &mut TemporalFields,
        overflow: ArithmeticOverflow,
    ) -> TemporalResult<MonthDay> {
        if self.is_iso() {
            fields.iso_resolve_month()?;
            return MonthDay::new(
                fields.month.unwrap_or(0),
                fields.day.unwrap_or(0),
                self.clone(),
                overflow,
            );
        }

        // TODO: This may get complicated...
        // For reference: https://github.com/tc39/proposal-temporal/blob/main/polyfill/lib/calendar.mjs#L1275.
        Err(TemporalError::range().with_message("Not yet implemented/supported."))
    }

    /// `CalendarYearMonthFromFields`
    pub fn year_month_from_fields(
        &self,
        fields: &mut TemporalFields,
        overflow: ArithmeticOverflow,
    ) -> TemporalResult<YearMonth> {
        if self.is_iso() {
            fields.iso_resolve_month()?;
            return YearMonth::new(
                fields.year.unwrap_or(0),
                fields.month.unwrap_or(0),
                fields.day,
                self.clone(),
                overflow,
            );
        }

        let era = Era::from_str(&fields.era.map_or(String::default(), |s| s.to_string()))
            .map_err(|e| TemporalError::general(format!("{e:?}")))?;
        let month_code = MonthCode(
            fields
                .month_code
                .map(|mc| {
                    TinyAsciiStr::from_bytes(mc.as_str().as_bytes())
                        .expect("MonthCode as_str is always valid.")
                })
                .ok_or(TemporalError::range().with_message("No MonthCode provided."))?,
        );

        // NOTE: This might preemptively throw as `ICU4X` does not support regulating.
        let calendar_date = self.0.date_from_codes(
            era,
            fields.year.unwrap_or(0),
            month_code,
            fields.day.unwrap_or(1) as u8,
        )?;
        let iso = self.0.date_to_iso(&calendar_date);
        YearMonth::new(
            iso.year().number,
            iso.month().ordinal as i32,
            Some(iso.day_of_month().0 as i32),
            self.clone(),
            overflow,
        )
    }

    /// `CalendarDateAdd`
    pub fn date_add(
        &self,
        date: &Date,
        duration: &Duration,
        overflow: ArithmeticOverflow,
    ) -> TemporalResult<Date> {
        if self.is_iso() {
            // 8. Let norm be NormalizeTimeDuration(duration.[[Hours]], duration.[[Minutes]], duration.[[Seconds]],
            // duration.[[Milliseconds]], duration.[[Microseconds]], duration.[[Nanoseconds]]).
            // 9. Let balanceResult be BalanceTimeDuration(norm, "day").
            let (balance_days, _) =
                TimeDuration::from_normalized(duration.time().to_normalized(), TemporalUnit::Day)?;

            // 10. Let result be ? AddISODate(date.[[ISOYear]], date.[[ISOMonth]], date.[[ISODay]], duration.[[Years]],
            // duration.[[Months]], duration.[[Weeks]], duration.[[Days]] + balanceResult.[[Days]], overflow).
            let result = date.iso.add_date_duration(
                &DateDuration::new_unchecked(
                    duration.years(),
                    duration.months(),
                    duration.weeks(),
                    duration.days().checked_add(&balance_days)?,
                ),
                overflow,
            )?;
            // 11. Return ? CreateTemporalDate(result.[[Year]], result.[[Month]], result.[[Day]], "iso8601").
            return Date::new(
                result.year,
                result.month.into(),
                result.day.into(),
                date.calendar().clone(),
                ArithmeticOverflow::Reject,
            );
        }

        Err(TemporalError::range().with_message("Not yet implemented."))
    }

    /// `CalendarDateUntil`
    pub fn date_until(
        &self,
        one: &Date,
        two: &Date,
        largest_unit: TemporalUnit,
    ) -> TemporalResult<Duration> {
        if self.is_iso() {
            let date_duration = one.iso.diff_iso_date(&two.iso, largest_unit)?;
            return Ok(Duration::from(date_duration));
        }

        Err(TemporalError::range().with_message("Not yet implemented."))
    }

    /// `CalendarEra`
    pub fn era(&self, date_like: &CalendarDateLike) -> TemporalResult<Option<TinyAsciiStr<16>>> {
        if self.is_iso() {
            return Ok(None);
        }
        let calendar_date = self.0.date_from_iso(date_like.as_iso_date().as_icu4x()?);
        Ok(Some(self.0.year(&calendar_date).era.0))
    }

    /// `CalendarEraYear`
    pub fn era_year(&self, date_like: &CalendarDateLike) -> TemporalResult<Option<i32>> {
        if self.is_iso() {
            return Ok(None);
        }
        let calendar_date = self.0.date_from_iso(date_like.as_iso_date().as_icu4x()?);
        Ok(Some(self.0.year(&calendar_date).number))
    }

    /// `CalendarYear`
    pub fn year(&self, date_like: &CalendarDateLike) -> TemporalResult<i32> {
        if self.is_iso() {
            return Ok(date_like.as_iso_date().year);
        }
        let calendar_date = self.0.date_from_iso(date_like.as_iso_date().as_icu4x()?);
        Ok(self.0.year(&calendar_date).number)
    }

    /// `CalendarMonth`
    pub fn month(&self, date_like: &CalendarDateLike) -> TemporalResult<u8> {
        if self.is_iso() {
            return Ok(date_like.as_iso_date().month);
        }

        Err(TemporalError::range().with_message("Not yet implemented."))
    }

    /// `CalendarMonthCode`
    pub fn month_code(&self, date_like: &CalendarDateLike) -> TemporalResult<TinyAsciiStr<4>> {
        if self.is_iso() {
            return Ok(date_like.as_iso_date().as_icu4x()?.month().code.0);
        }

        Err(TemporalError::range().with_message("Not yet implemented."))
    }

    /// `CalendarDay`
    pub fn day(&self, date_like: &CalendarDateLike) -> TemporalResult<u8> {
        if self.is_iso() {
            return Ok(date_like.as_iso_date().day);
        }

        Err(TemporalError::range().with_message("Not yet implemented."))
    }

    /// `CalendarDayOfWeek`
    pub fn day_of_week(&self, date_like: &CalendarDateLike) -> TemporalResult<u16> {
        if self.is_iso() {
            return Ok(date_like.as_iso_date().as_icu4x()?.day_of_week() as u16);
        }

        Err(TemporalError::range().with_message("Not yet implemented."))
    }

    /// `CalendarDayOfYear`
    pub fn day_of_year(&self, date_like: &CalendarDateLike) -> TemporalResult<u16> {
        if self.is_iso() {
            return Ok(date_like
                .as_iso_date()
                .as_icu4x()?
                .day_of_year_info()
                .day_of_year);
        }
        Err(TemporalError::range().with_message("Not yet implemented."))?
    }

    /// `CalendarWeekOfYear`
    pub fn week_of_year(&self, date_like: &CalendarDateLike) -> TemporalResult<u16> {
        if self.is_iso() {
            let date = date_like.as_iso_date().as_icu4x()?;

            let week_calculator = WeekCalculator::default();

            let week_of = date
                .week_of_year(&week_calculator)
                .map_err(|err| TemporalError::range().with_message(err.to_string()))?;

            return Ok(week_of.week);
        }
        Err(TemporalError::range().with_message("Not yet implemented."))
    }

    /// `CalendarYearOfWeek`
    pub fn year_of_week(&self, date_like: &CalendarDateLike) -> TemporalResult<i32> {
        if self.is_iso() {
            let date = date_like.as_iso_date().as_icu4x()?;

            let week_calculator = WeekCalculator::default();

            let week_of = date
                .week_of_year(&week_calculator)
                .map_err(|err| TemporalError::range().with_message(err.to_string()))?;

            return match week_of.unit {
                RelativeUnit::Previous => Ok(date.year().number - 1),
                RelativeUnit::Current => Ok(date.year().number),
                RelativeUnit::Next => Ok(date.year().number + 1),
            };
        }
        Err(TemporalError::range().with_message("Not yet implemented."))
    }

    /// `CalendarDaysInWeek`
    pub fn days_in_week(&self, _date_like: &CalendarDateLike) -> TemporalResult<u16> {
        if self.is_iso() {
            return Ok(7);
        }
        Err(TemporalError::range().with_message("Not yet implemented."))
    }

    /// `CalendarDaysInMonth`
    pub fn days_in_month(&self, date_like: &CalendarDateLike) -> TemporalResult<u16> {
        if self.is_iso() {
            return Ok(date_like.as_iso_date().as_icu4x()?.days_in_month() as u16);
        }
        Err(TemporalError::range().with_message("Not yet implemented."))
    }

    /// `CalendarDaysInYear`
    pub fn days_in_year(&self, date_like: &CalendarDateLike) -> TemporalResult<u16> {
        if self.is_iso() {
            return Ok(date_like.as_iso_date().as_icu4x()?.days_in_year());
        }

        Err(TemporalError::range().with_message("Not yet implemented."))
    }

    /// `CalendarMonthsInYear`
    pub fn months_in_year(&self, _date_like: &CalendarDateLike) -> TemporalResult<u16> {
        if self.is_iso() {
            return Ok(12);
        }
        Err(TemporalError::range().with_message("Not yet implemented."))
    }

    /// `CalendarInLeapYear`
    pub fn in_leap_year(&self, date_like: &CalendarDateLike) -> TemporalResult<bool> {
        if self.is_iso() {
            return Ok(date_like.as_iso_date().as_icu4x()?.is_in_leap_year());
        }
        Err(TemporalError::range().with_message("Not yet implemented."))
    }

    /// `CalendarFields`
    pub fn fields(&self, fields: Vec<String>) -> TemporalResult<Vec<String>> {
        if self.is_iso() {
            return Ok(fields);
        }
        Err(TemporalError::range().with_message("Not yet implemented."))
    }

    /// Returns the identifier of this calendar slot.
    pub fn identifier(&self) -> String {
        if self.is_iso() {
            return String::from("iso8601");
        }
        String::from(self.0 .0.kind().as_bcp47_string())
    }
}

impl Calendar {
    /// Returns the designated field descriptors for builtin calendars.
    pub fn field_descriptors(
        &self,
        _fields_type: CalendarFieldsType,
    ) -> TemporalResult<Vec<(String, bool)>> {
        // TODO: Research and implement the appropriate descriptors for all `BuiltinCalendars.`
        Err(TemporalError::range().with_message("FieldDescriptors is not yet implemented."))
    }

    /// Provides field keys to be ignored depending on the calendar.
    pub fn field_keys_to_ignore(&self, keys: FieldMap) -> TemporalResult<FieldMap> {
        let mut ignored_keys = FieldMap::empty();
        if self.is_iso() {
            // NOTE: It is okay for ignored keys to have duplicates?
            for key in keys.iter() {
                ignored_keys.set(key, true);
                if key == FieldMap::MONTH {
                    ignored_keys.set(FieldMap::MONTH_CODE, true);
                } else if key == FieldMap::MONTH_CODE {
                    ignored_keys.set(FieldMap::MONTH, true);
                }
            }

            return Ok(ignored_keys);
        }

        // TODO: Research and implement the appropriate KeysToIgnore for all `BuiltinCalendars.`
        Err(TemporalError::range().with_message("FieldKeysToIgnore is not yet implemented."))
    }

    /// `CalendarResolveFields`
    pub fn resolve_fields(
        &self,
        _fields: &mut TemporalFields,
        _typ: CalendarFieldsType,
    ) -> TemporalResult<()> {
        // TODO: Research and implement the appropriate ResolveFields for all `BuiltinCalendars.`
        Err(TemporalError::range().with_message("CalendarResolveFields is not yet implemented."))
    }
}

impl From<Date> for Calendar {
    fn from(value: Date) -> Self {
        value.calendar().clone()
    }
}

impl From<DateTime> for Calendar {
    fn from(value: DateTime) -> Self {
        value.calendar().clone()
    }
}

impl From<ZonedDateTime> for Calendar {
    fn from(value: ZonedDateTime) -> Self {
        value.calendar().clone()
    }
}

impl From<MonthDay> for Calendar {
    fn from(value: MonthDay) -> Self {
        value.calendar().clone()
    }
}

impl From<YearMonth> for Calendar {
    fn from(value: YearMonth) -> Self {
        value.calendar().clone()
    }
}

#[cfg(test)]
mod tests {
    use crate::{components::Date, iso::IsoDate, options::TemporalUnit};

    use super::Calendar;

    #[test]
    fn date_until_largest_year() {
        // tests format: (Date one, Date two, Duration result)
        let tests = [
            ((2021, 7, 16), (2021, 7, 16), (0, 0, 0, 0, 0, 0, 0, 0, 0, 0)),
            ((2021, 7, 16), (2021, 7, 17), (0, 0, 0, 1, 0, 0, 0, 0, 0, 0)),
            ((2021, 7, 16), (2021, 7, 23), (0, 0, 0, 7, 0, 0, 0, 0, 0, 0)),
            ((2021, 7, 16), (2021, 8, 16), (0, 1, 0, 0, 0, 0, 0, 0, 0, 0)),
            (
                (2020, 12, 16),
                (2021, 1, 16),
                (0, 1, 0, 0, 0, 0, 0, 0, 0, 0),
            ),
            ((2021, 1, 5), (2021, 2, 5), (0, 1, 0, 0, 0, 0, 0, 0, 0, 0)),
            ((2021, 1, 7), (2021, 3, 7), (0, 2, 0, 0, 0, 0, 0, 0, 0, 0)),
            ((2021, 7, 16), (2021, 8, 17), (0, 1, 0, 1, 0, 0, 0, 0, 0, 0)),
            (
                (2021, 7, 16),
                (2021, 8, 13),
                (0, 0, 0, 28, 0, 0, 0, 0, 0, 0),
            ),
            ((2021, 7, 16), (2021, 9, 16), (0, 2, 0, 0, 0, 0, 0, 0, 0, 0)),
            ((2021, 7, 16), (2022, 7, 16), (1, 0, 0, 0, 0, 0, 0, 0, 0, 0)),
            (
                (2021, 7, 16),
                (2031, 7, 16),
                (10, 0, 0, 0, 0, 0, 0, 0, 0, 0),
            ),
            ((2021, 7, 16), (2022, 7, 19), (1, 0, 0, 3, 0, 0, 0, 0, 0, 0)),
            ((2021, 7, 16), (2022, 9, 19), (1, 2, 0, 3, 0, 0, 0, 0, 0, 0)),
            (
                (2021, 7, 16),
                (2031, 12, 16),
                (10, 5, 0, 0, 0, 0, 0, 0, 0, 0),
            ),
            (
                (1997, 12, 16),
                (2021, 7, 16),
                (23, 7, 0, 0, 0, 0, 0, 0, 0, 0),
            ),
            (
                (1997, 7, 16),
                (2021, 7, 16),
                (24, 0, 0, 0, 0, 0, 0, 0, 0, 0),
            ),
            (
                (1997, 7, 16),
                (2021, 7, 15),
                (23, 11, 0, 29, 0, 0, 0, 0, 0, 0),
            ),
            (
                (1997, 6, 16),
                (2021, 6, 15),
                (23, 11, 0, 30, 0, 0, 0, 0, 0, 0),
            ),
            (
                (1960, 2, 16),
                (2020, 3, 16),
                (60, 1, 0, 0, 0, 0, 0, 0, 0, 0),
            ),
            (
                (1960, 2, 16),
                (2021, 3, 15),
                (61, 0, 0, 27, 0, 0, 0, 0, 0, 0),
            ),
            (
                (1960, 2, 16),
                (2020, 3, 15),
                (60, 0, 0, 28, 0, 0, 0, 0, 0, 0),
            ),
            (
                (2021, 3, 30),
                (2021, 7, 16),
                (0, 3, 0, 16, 0, 0, 0, 0, 0, 0),
            ),
            (
                (2020, 3, 30),
                (2021, 7, 16),
                (1, 3, 0, 16, 0, 0, 0, 0, 0, 0),
            ),
            (
                (1960, 3, 30),
                (2021, 7, 16),
                (61, 3, 0, 16, 0, 0, 0, 0, 0, 0),
            ),
            (
                (2019, 12, 30),
                (2021, 7, 16),
                (1, 6, 0, 16, 0, 0, 0, 0, 0, 0),
            ),
            (
                (2020, 12, 30),
                (2021, 7, 16),
                (0, 6, 0, 16, 0, 0, 0, 0, 0, 0),
            ),
            (
                (1997, 12, 30),
                (2021, 7, 16),
                (23, 6, 0, 16, 0, 0, 0, 0, 0, 0),
            ),
            (
                (1, 12, 25),
                (2021, 7, 16),
                (2019, 6, 0, 21, 0, 0, 0, 0, 0, 0),
            ),
            ((2019, 12, 30), (2021, 3, 5), (1, 2, 0, 5, 0, 0, 0, 0, 0, 0)),
            (
                (2021, 7, 17),
                (2021, 7, 16),
                (0, 0, 0, -1, 0, 0, 0, 0, 0, 0),
            ),
            (
                (2021, 7, 23),
                (2021, 7, 16),
                (0, 0, 0, -7, 0, 0, 0, 0, 0, 0),
            ),
            (
                (2021, 8, 16),
                (2021, 7, 16),
                (0, -1, 0, 0, 0, 0, 0, 0, 0, 0),
            ),
            (
                (2021, 1, 16),
                (2020, 12, 16),
                (0, -1, 0, 0, 0, 0, 0, 0, 0, 0),
            ),
            ((2021, 2, 5), (2021, 1, 5), (0, -1, 0, 0, 0, 0, 0, 0, 0, 0)),
            ((2021, 3, 7), (2021, 1, 7), (0, -2, 0, 0, 0, 0, 0, 0, 0, 0)),
            (
                (2021, 8, 17),
                (2021, 7, 16),
                (0, -1, 0, -1, 0, 0, 0, 0, 0, 0),
            ),
            (
                (2021, 8, 13),
                (2021, 7, 16),
                (0, 0, 0, -28, 0, 0, 0, 0, 0, 0),
            ),
            (
                (2021, 9, 16),
                (2021, 7, 16),
                (0, -2, 0, 0, 0, 0, 0, 0, 0, 0),
            ),
            (
                (2022, 7, 16),
                (2021, 7, 16),
                (-1, 0, 0, 0, 0, 0, 0, 0, 0, 0),
            ),
            (
                (2031, 7, 16),
                (2021, 7, 16),
                (-10, 0, 0, 0, 0, 0, 0, 0, 0, 0),
            ),
            (
                (2022, 7, 19),
                (2021, 7, 16),
                (-1, 0, 0, -3, 0, 0, 0, 0, 0, 0),
            ),
            (
                (2022, 9, 19),
                (2021, 7, 16),
                (-1, -2, 0, -3, 0, 0, 0, 0, 0, 0),
            ),
            (
                (2031, 12, 16),
                (2021, 7, 16),
                (-10, -5, 0, 0, 0, 0, 0, 0, 0, 0),
            ),
            (
                (2021, 7, 16),
                (1997, 12, 16),
                (-23, -7, 0, 0, 0, 0, 0, 0, 0, 0),
            ),
            (
                (2021, 7, 16),
                (1997, 7, 16),
                (-24, 0, 0, 0, 0, 0, 0, 0, 0, 0),
            ),
            (
                (2021, 7, 15),
                (1997, 7, 16),
                (-23, -11, 0, -30, 0, 0, 0, 0, 0, 0),
            ),
            (
                (2021, 6, 15),
                (1997, 6, 16),
                (-23, -11, 0, -29, 0, 0, 0, 0, 0, 0),
            ),
            (
                (2020, 3, 16),
                (1960, 2, 16),
                (-60, -1, 0, 0, 0, 0, 0, 0, 0, 0),
            ),
            (
                (2021, 3, 15),
                (1960, 2, 16),
                (-61, 0, 0, -28, 0, 0, 0, 0, 0, 0),
            ),
            (
                (2020, 3, 15),
                (1960, 2, 16),
                (-60, 0, 0, -28, 0, 0, 0, 0, 0, 0),
            ),
            (
                (2021, 7, 16),
                (2021, 3, 30),
                (0, -3, 0, -17, 0, 0, 0, 0, 0, 0),
            ),
            (
                (2021, 7, 16),
                (2020, 3, 30),
                (-1, -3, 0, -17, 0, 0, 0, 0, 0, 0),
            ),
            (
                (2021, 7, 16),
                (1960, 3, 30),
                (-61, -3, 0, -17, 0, 0, 0, 0, 0, 0),
            ),
            (
                (2021, 7, 16),
                (2019, 12, 30),
                (-1, -6, 0, -17, 0, 0, 0, 0, 0, 0),
            ),
            (
                (2021, 7, 16),
                (2020, 12, 30),
                (0, -6, 0, -17, 0, 0, 0, 0, 0, 0),
            ),
            (
                (2021, 7, 16),
                (1997, 12, 30),
                (-23, -6, 0, -17, 0, 0, 0, 0, 0, 0),
            ),
            (
                (2021, 7, 16),
                (1, 12, 25),
                (-2019, -6, 0, -22, 0, 0, 0, 0, 0, 0),
            ),
            (
                (2021, 3, 5),
                (2019, 12, 30),
                (-1, -2, 0, -6, 0, 0, 0, 0, 0, 0),
            ),
        ];

        let calendar = Calendar::default();

        for test in tests {
            let first = Date::new_unchecked(
                IsoDate::new_unchecked(test.0 .0, test.0 .1, test.0 .2),
                calendar.clone(),
            );
            let second = Date::new_unchecked(
                IsoDate::new_unchecked(test.1 .0, test.1 .1, test.1 .2),
                calendar.clone(),
            );
            let result = calendar
                .date_until(&first, &second, TemporalUnit::Year)
                .unwrap();
            assert_eq!(
                result.years().0 as i32,
                test.2 .0,
                "year failed for test \"{test:?}\""
            );
            assert_eq!(
                result.months().0 as i32,
                test.2 .1,
                "months failed for test \"{test:?}\""
            );
            assert_eq!(
                result.weeks().0 as i32,
                test.2 .2,
                "weeks failed for test \"{test:?}\""
            );
            assert_eq!(
                result.days().0 as i32,
                test.2 .3,
                "days failed for test \"{test:?}\""
            );
        }
    }
}
