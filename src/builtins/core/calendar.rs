//! This module implements the calendar traits and related components.
//!
//! The goal of the calendar module of `boa_temporal` is to provide
//! Temporal compatible calendar implementations.

use crate::{
    builtins::core::{
        duration::DateDuration, Duration, PlainDate, PlainDateTime, PlainMonthDay, PlainYearMonth,
    },
    iso::IsoDate,
    options::{Overflow, Unit},
    parsers::parse_allowed_calendar_formats,
    TemporalError, TemporalResult,
};
use core::str::FromStr;

use core::num::NonZero;
use icu_calendar::{
    cal::{
        Buddhist, Coptic, Ethiopian, EthiopianEraStyle, Hebrew, Hijri, Indian, Japanese,
        JapaneseExtended, LunarChinese, Persian, Roc,
    },
    AnyCalendar, AnyCalendarKind, Calendar as IcuCalendar, Iso, Ref,
};
use icu_calendar::{
    cal::{HijriTabularEpoch, HijriTabularLeapYears},
    options::{DateFromFieldsOptions, MissingFieldsStrategy, Overflow as IcuOverflow},
    preferences::CalendarAlgorithm,
    types::DateFields,
    types::MonthCode as IcuMonthCode,
    Gregorian,
};
use icu_locale::extensions::unicode::Value;
use tinystr::TinyAsciiStr;

use super::ZonedDateTime;

mod fields;
mod types;

pub use fields::{CalendarFields, YearMonthCalendarFields};
#[cfg(test)]
pub(crate) use types::month_to_month_code;
pub(crate) use types::ResolutionType;
pub use types::{MonthCode, ResolvedIsoFields};

/// The core `Calendar` type for `temporal_rs`
///
/// A `Calendar` in `temporal_rs` can be any calendar that is currently
/// supported by [`icu_calendar`].
#[derive(Debug, Clone)]
pub struct Calendar(Ref<'static, AnyCalendar>);

impl Default for Calendar {
    fn default() -> Self {
        Self::ISO
    }
}

impl PartialEq for Calendar {
    fn eq(&self, other: &Self) -> bool {
        self.identifier() == other.identifier()
    }
}

impl Eq for Calendar {}

impl Calendar {
    /// The Buddhist calendar
    pub const BUDDHIST: Self = Self::new(AnyCalendarKind::Buddhist);
    /// The Chinese calendar
    pub const CHINESE: Self = Self::new(AnyCalendarKind::Chinese);
    /// The Coptic calendar
    pub const COPTIC: Self = Self::new(AnyCalendarKind::Coptic);
    /// The Dangi calendar
    pub const DANGI: Self = Self::new(AnyCalendarKind::Dangi);
    /// The Ethiopian calendar
    pub const ETHIOPIAN: Self = Self::new(AnyCalendarKind::Ethiopian);
    /// The Ethiopian Amete Alem calendar
    pub const ETHIOPIAN_AMETE_ALEM: Self = Self::new(AnyCalendarKind::EthiopianAmeteAlem);
    /// The Gregorian calendar
    pub const GREGORIAN: Self = Self::new(AnyCalendarKind::Gregorian);
    /// The Hebrew calendar
    pub const HEBREW: Self = Self::new(AnyCalendarKind::Hebrew);
    /// The Indian calendar
    pub const INDIAN: Self = Self::new(AnyCalendarKind::Indian);
    /// The Hijri Tabular calendar with a Friday epoch
    pub const HIJRI_TABULAR_FRIDAY: Self = Self::new(AnyCalendarKind::HijriTabularTypeIIFriday);
    /// The Hijri Tabular calendar with a Thursday epoch
    pub const HIJRI_TABULAR_THURSDAY: Self = Self::new(AnyCalendarKind::HijriTabularTypeIIThursday);
    /// The Hijri Umm al-Qura calendar
    pub const HIJRI_UMM_AL_QURA: Self = Self::new(AnyCalendarKind::HijriUmmAlQura);
    /// The ISO 8601 calendar
    pub const ISO: Self = Self::new(AnyCalendarKind::Iso);
    /// The Japanese calendar
    pub const JAPANESE: Self = Self::new(AnyCalendarKind::Japanese);
    /// The Persian calendar
    pub const PERSIAN: Self = Self::new(AnyCalendarKind::Persian);
    /// The ROC calendar
    pub const ROC: Self = Self::new(AnyCalendarKind::Roc);

    /// Create a `Calendar` from an ICU [`AnyCalendarKind`].
    #[warn(clippy::wildcard_enum_match_arm)] // Warns if the calendar kind gets out of sync.
    pub const fn new(kind: AnyCalendarKind) -> Self {
        let cal = match kind {
            AnyCalendarKind::Buddhist => &AnyCalendar::Buddhist(Buddhist),
            AnyCalendarKind::Chinese => const { &AnyCalendar::Chinese(LunarChinese::new_china()) },
            AnyCalendarKind::Coptic => &AnyCalendar::Coptic(Coptic),
            AnyCalendarKind::Dangi => const { &AnyCalendar::Dangi(LunarChinese::new_dangi()) },
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
            AnyCalendarKind::HijriTabularTypeIIFriday => {
                const {
                    &AnyCalendar::HijriTabular(Hijri::new_tabular(
                        HijriTabularLeapYears::TypeII,
                        HijriTabularEpoch::Friday,
                    ))
                }
            }
            AnyCalendarKind::HijriSimulatedMecca => {
                // This calendar is currently unsupported by Temporal
                &AnyCalendar::Iso(Iso)
            }
            AnyCalendarKind::HijriTabularTypeIIThursday => {
                const {
                    &AnyCalendar::HijriTabular(Hijri::new_tabular(
                        HijriTabularLeapYears::TypeII,
                        HijriTabularEpoch::Thursday,
                    ))
                }
            }
            AnyCalendarKind::HijriUmmAlQura => {
                const { &AnyCalendar::HijriUmmAlQura(Hijri::new_umm_al_qura()) }
            }
            AnyCalendarKind::Iso => &AnyCalendar::Iso(Iso),
            AnyCalendarKind::Japanese => const { &AnyCalendar::Japanese(Japanese::new()) },
            AnyCalendarKind::JapaneseExtended => {
                const { &AnyCalendar::JapaneseExtended(JapaneseExtended::new()) }
            }
            AnyCalendarKind::Persian => &AnyCalendar::Persian(Persian),
            AnyCalendarKind::Roc => &AnyCalendar::Roc(Roc),
            _ => {
                debug_assert!(
                    false,
                    "Unreachable: match must handle all variants of `AnyCalendarKind`"
                );
                &AnyCalendar::Iso(Iso)
            }
        };

        Self(Ref(cal))
    }

    /// Returns a `Calendar` from the a slice of UTF-8 encoded bytes.
    pub fn try_from_utf8(bytes: &[u8]) -> TemporalResult<Self> {
        let kind = Self::try_kind_from_utf8(bytes)?;
        Ok(Self::new(kind))
    }

    /// Returns a `Calendar` from the a slice of UTF-8 encoded bytes.
    pub(crate) fn try_kind_from_utf8(bytes: &[u8]) -> TemporalResult<AnyCalendarKind> {
        // TODO: Determine the best way to handle "julian" here.
        // Not supported by `CalendarAlgorithm`
        let icu_locale_value = Value::try_from_utf8(&bytes.to_ascii_lowercase())
            .map_err(|_| TemporalError::range().with_message("unknown calendar"))?;
        let algorithm = CalendarAlgorithm::try_from(&icu_locale_value)
            .map_err(|_| TemporalError::range().with_message("unknown calendar"))?;
        let calendar_kind = match AnyCalendarKind::try_from(algorithm) {
            Ok(c) => c,
            // Handle `islamic` calendar idenitifier.
            //
            // This should be updated depending on `icu_calendar` support and
            // intl-era-monthcode.
            Err(()) if algorithm == CalendarAlgorithm::Hijri(None) => {
                AnyCalendarKind::HijriTabularTypeIIFriday
            }
            Err(()) => return Err(TemporalError::range().with_message("unknown calendar")),
        };
        Ok(calendar_kind)
    }
}

impl FromStr for Calendar {
    type Err = TemporalError;

    // 13.34 ParseTemporalCalendarString ( string )
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match parse_allowed_calendar_formats(s.as_bytes()) {
            Some([]) => Ok(Calendar::ISO),
            Some(result) => Calendar::try_from_utf8(result),
            None => Calendar::try_from_utf8(s.as_bytes()),
        }
    }
}

impl From<Overflow> for IcuOverflow {
    fn from(other: Overflow) -> Self {
        match other {
            Overflow::Reject => Self::Reject,
            Overflow::Constrain => Self::Constrain,
        }
    }
}

impl<'a> TryFrom<&'a CalendarFields> for DateFields<'a> {
    type Error = TemporalError;
    fn try_from(other: &'a CalendarFields) -> TemporalResult<Self> {
        let ordinal_month = other
            .month
            .map(|o| {
                NonZero::new(o).ok_or(
                    TemporalError::range().with_message("Ordinal month must be positive integer"),
                )
            })
            .transpose()?;
        let day = other
            .day
            .map(|o| {
                NonZero::new(o)
                    .ok_or(TemporalError::range().with_message("Day must be positive integer"))
            })
            .transpose()?;
        let mut this = DateFields::default();
        this.era = other.era.as_deref();
        this.era_year = other.era_year;
        this.extended_year = other.year;
        this.month_code = other.month_code.map(|c| IcuMonthCode(c.0));
        this.ordinal_month = ordinal_month;
        this.day = day;
        Ok(this)
    }
}
// ==== Public `CalendarSlot` methods ====

impl Calendar {
    /// Returns whether the current calendar is `ISO`
    #[inline]
    pub fn is_iso(&self) -> bool {
        matches!(self.0 .0, AnyCalendar::Iso(_))
    }

    /// Returns the kind of this calendar
    #[inline]
    pub fn kind(&self) -> AnyCalendarKind {
        self.0 .0.kind()
    }

    /// `CalendarDateFromFields`
    pub fn date_from_fields(
        &self,
        fields: CalendarFields,
        overflow: Overflow,
    ) -> TemporalResult<PlainDate> {
        if self.is_iso() {
            let resolved_fields =
                ResolvedIsoFields::try_from_fields(&fields, overflow, ResolutionType::Date)?;

            // Resolve month and monthCode;
            return PlainDate::new_with_overflow(
                resolved_fields.arithmetic_year,
                resolved_fields.month,
                resolved_fields.day,
                self.clone(),
                overflow,
            );
        }

        let fields = DateFields::try_from(&fields)?;
        let mut options = DateFromFieldsOptions::default();
        options.overflow = Some(overflow.into());
        options.missing_fields_strategy = Some(MissingFieldsStrategy::Reject);

        let calendar_date = self.0.from_fields(fields, options)?;
        let iso = self.0.to_iso(&calendar_date);
        PlainDate::new_with_overflow(
            Iso.extended_year(&iso),
            Iso.month(&iso).ordinal,
            Iso.day_of_month(&iso).0,
            self.clone(),
            overflow,
        )
    }

    /// `CalendarPlainMonthDayFromFields`
    pub fn month_day_from_fields(
        &self,
        mut fields: CalendarFields,
        overflow: Overflow,
    ) -> TemporalResult<PlainMonthDay> {
        // You are allowed to specify year information, however
        // it is *only* used for resolving the given month/day data.
        //
        // For example, constructing a PlainMonthDay for {year: 2025, month: 2, day: 29}
        // with overflow: constrain will produce 02-28 since it will constrain
        // the date to 2025-02-28 first, and only *then* will it construct an MD.
        //
        // This is specced partially in https://tc39.es/proposal-temporal/#sec-temporal-calendarmonthdaytoisoreferencedate
        // notice that RegulateISODate is called with the passed-in year, but the reference year is used regardless
        // of the passed in year in the final result.
        //
        // There may be more efficient ways to do this, but this works pretty well and doesn't require
        // calendrical knowledge.
        if fields.year.is_some() || (fields.era.is_some() && fields.era_year.is_some()) {
            let date = self.date_from_fields(fields, overflow)?;
            fields = CalendarFields::from_date(&date);
        }
        if self.is_iso() {
            let resolved_fields =
                ResolvedIsoFields::try_from_fields(&fields, overflow, ResolutionType::MonthDay)?;
            return PlainMonthDay::new_with_overflow(
                resolved_fields.month,
                resolved_fields.day,
                self.clone(),
                overflow,
                None,
            );
        }

        let fields = DateFields::try_from(&fields)?;
        let mut options = DateFromFieldsOptions::default();
        options.overflow = Some(overflow.into());
        if fields.day.is_none() {
            // Otherwise we're liable to hit the YearMonth resolution
            return Err(TemporalError::r#type().with_message("Must specify day for MonthDay"));
        }
        options.missing_fields_strategy = Some(MissingFieldsStrategy::Ecma);

        let mut calendar_date = self.0.from_fields(fields, options)?;

        // The MonthDay algorithm wants us to resolve a date *with* the provided year,
        // if one was provided, but then use a reference year afterwards.
        // So if a year was provided, we reresolve.
        if fields.era_year.is_some() || fields.extended_year.is_some() {
            let mut fields2 = DateFields::default();
            fields2.day = NonZero::new(self.0.day_of_month(&calendar_date).0);
            fields2.month_code = Some(self.0.month(&calendar_date).standard_code);

            calendar_date = self.0.from_fields(fields2, options)?;
        }

        let iso = self.0.to_iso(&calendar_date);
        PlainMonthDay::new_with_overflow(
            Iso.month(&iso).ordinal,
            Iso.day_of_month(&iso).0,
            self.clone(),
            overflow,
            Some(Iso.extended_year(&iso)),
        )
    }

    /// `CalendarPlainYearMonthFromFields`
    pub fn year_month_from_fields(
        &self,
        fields: YearMonthCalendarFields,
        overflow: Overflow,
    ) -> TemporalResult<PlainYearMonth> {
        if self.is_iso() {
            // TODO: add a from_partial_year_month method on ResolvedCalendarFields
            let resolved_fields = ResolvedIsoFields::try_from_fields(
                &CalendarFields::from(fields),
                overflow,
                ResolutionType::YearMonth,
            )?;
            return PlainYearMonth::new_with_overflow(
                resolved_fields.arithmetic_year,
                resolved_fields.month,
                Some(resolved_fields.day),
                self.clone(),
                overflow,
            );
        }

        let fields = CalendarFields::from(fields);
        let fields = DateFields::try_from(&fields)?;
        let mut options = DateFromFieldsOptions::default();
        options.overflow = Some(overflow.into());
        if fields.extended_year.is_none() && fields.era_year.is_none() {
            // Otherwise we're liable to hit the MonthDay resolution
            return Err(TemporalError::r#type().with_message("Must specify year for YearMonth"));
        }
        options.missing_fields_strategy = Some(MissingFieldsStrategy::Ecma);
        let calendar_date = self.0.from_fields(fields, options)?;
        let iso = self.0.to_iso(&calendar_date);
        PlainYearMonth::new_with_overflow(
            Iso.year_info(&iso).year,
            Iso.month(&iso).ordinal,
            Some(Iso.day_of_month(&iso).0),
            self.clone(),
            overflow,
        )
    }

    /// `CalendarDateAdd`
    pub fn date_add(
        &self,
        date: &IsoDate,
        duration: &DateDuration,
        overflow: Overflow,
    ) -> TemporalResult<PlainDate> {
        // 1. If calendar is "iso8601", then
        if self.is_iso() {
            let result = date.add_date_duration(duration, overflow)?;
            // 11. Return ? CreateTemporalDate(result.[[Year]], result.[[Month]], result.[[Day]], "iso8601").
            return PlainDate::try_new(result.year, result.month, result.day, self.clone());
        }

        Err(TemporalError::range().with_message("Not yet implemented."))
    }

    /// `CalendarDateUntil`
    pub fn date_until(
        &self,
        one: &IsoDate,
        two: &IsoDate,
        largest_unit: Unit,
    ) -> TemporalResult<Duration> {
        if self.is_iso() {
            let date_duration = one.diff_iso_date(two, largest_unit)?;
            return Ok(Duration::from(date_duration));
        }
        Err(TemporalError::range().with_message("Not yet implemented."))
    }

    /// `CalendarEra`
    pub fn era(&self, iso_date: &IsoDate) -> Option<TinyAsciiStr<16>> {
        if self.is_iso() {
            return None;
        }
        let calendar_date = self.0.from_iso(*iso_date.to_icu4x().inner());
        self.0
            .year_info(&calendar_date)
            .era()
            .map(|era_info| era_info.era)
    }

    /// `CalendarEraYear`
    pub fn era_year(&self, iso_date: &IsoDate) -> Option<i32> {
        if self.is_iso() {
            return None;
        }
        let calendar_date = self.0.from_iso(*iso_date.to_icu4x().inner());
        self.0
            .year_info(&calendar_date)
            .era()
            .map(|era_info| era_info.year)
    }

    /// `CalendarArithmeticYear`
    pub fn year(&self, iso_date: &IsoDate) -> i32 {
        if self.is_iso() {
            return iso_date.year;
        }
        let calendar_date = self.0.from_iso(*iso_date.to_icu4x().inner());
        self.0.extended_year(&calendar_date)
    }

    /// `CalendarMonth`
    pub fn month(&self, iso_date: &IsoDate) -> u8 {
        if self.is_iso() {
            return iso_date.month;
        }
        let calendar_date = self.0.from_iso(*iso_date.to_icu4x().inner());
        self.0.month(&calendar_date).ordinal
    }

    /// `CalendarMonthCode`
    pub fn month_code(&self, iso_date: &IsoDate) -> MonthCode {
        if self.is_iso() {
            let mc = iso_date.to_icu4x().month().standard_code.0;
            return MonthCode(mc);
        }
        let calendar_date = self.0.from_iso(*iso_date.to_icu4x().inner());
        MonthCode(self.0.month(&calendar_date).standard_code.0)
    }

    /// `CalendarDay`
    pub fn day(&self, iso_date: &IsoDate) -> u8 {
        if self.is_iso() {
            return iso_date.day;
        }
        let calendar_date = self.0.from_iso(*iso_date.to_icu4x().inner());
        self.0.day_of_month(&calendar_date).0
    }

    /// `CalendarDayOfWeek`
    pub fn day_of_week(&self, iso_date: &IsoDate) -> u16 {
        iso_date.to_icu4x().day_of_week() as u16
    }

    /// `CalendarDayOfYear`
    pub fn day_of_year(&self, iso_date: &IsoDate) -> u16 {
        if self.is_iso() {
            return iso_date.to_icu4x().day_of_year().0;
        }
        let calendar_date = self.0.from_iso(*iso_date.to_icu4x().inner());
        self.0.day_of_year(&calendar_date).0
    }

    /// `CalendarWeekOfYear`
    pub fn week_of_year(&self, iso_date: &IsoDate) -> Option<u8> {
        if self.is_iso() {
            return Some(iso_date.to_icu4x().week_of_year().week_number);
        }
        // TODO: Research in ICU4X and determine best approach.
        None
    }

    /// `CalendarYearOfWeek`
    pub fn year_of_week(&self, iso_date: &IsoDate) -> Option<i32> {
        if self.is_iso() {
            return Some(iso_date.to_icu4x().week_of_year().iso_year);
        }
        // TODO: Research in ICU4X and determine best approach.
        None
    }

    /// `CalendarDaysInWeek`
    pub fn days_in_week(&self, _iso_date: &IsoDate) -> u16 {
        7
    }

    /// `CalendarDaysInMonth`
    pub fn days_in_month(&self, iso_date: &IsoDate) -> u16 {
        if self.is_iso() {
            return iso_date.to_icu4x().days_in_month() as u16;
        }
        let calendar_date = self.0.from_iso(*iso_date.to_icu4x().inner());
        self.0.days_in_month(&calendar_date) as u16
    }

    /// `CalendarDaysInYear`
    pub fn days_in_year(&self, iso_date: &IsoDate) -> u16 {
        if self.is_iso() {
            return iso_date.to_icu4x().days_in_year();
        }
        let calendar_date = self.0.from_iso(*iso_date.to_icu4x().inner());
        self.0.days_in_year(&calendar_date)
    }

    /// `CalendarMonthsInYear`
    pub fn months_in_year(&self, iso_date: &IsoDate) -> u16 {
        if self.is_iso() {
            return 12;
        }
        let calendar_date = self.0.from_iso(*iso_date.to_icu4x().inner());
        self.0.months_in_year(&calendar_date) as u16
    }

    /// `CalendarInLeapYear`
    pub fn in_leap_year(&self, iso_date: &IsoDate) -> bool {
        if self.is_iso() {
            return iso_date.to_icu4x().is_in_leap_year();
        }
        let calendar_date = self.0.from_iso(*iso_date.to_icu4x().inner());
        self.0.is_in_leap_year(&calendar_date)
    }

    /// Returns the identifier of this calendar slot.
    pub fn identifier(&self) -> &'static str {
        // icu_calendar lists iso8601 and julian as None
        match self.0.calendar_algorithm() {
            Some(c) => c.as_str(),
            None if self.is_iso() => "iso8601",
            None => "julian",
        }
    }
}

impl Calendar {
    pub(crate) fn calendar_has_eras(kind: AnyCalendarKind) -> bool {
        match kind {
            AnyCalendarKind::Buddhist
            | AnyCalendarKind::Coptic
            | AnyCalendarKind::Ethiopian
            | AnyCalendarKind::EthiopianAmeteAlem
            | AnyCalendarKind::Gregorian
            | AnyCalendarKind::Hebrew
            | AnyCalendarKind::Indian
            | AnyCalendarKind::HijriSimulatedMecca
            | AnyCalendarKind::HijriTabularTypeIIFriday
            | AnyCalendarKind::HijriTabularTypeIIThursday
            | AnyCalendarKind::HijriUmmAlQura
            | AnyCalendarKind::Japanese
            | AnyCalendarKind::Persian
            | AnyCalendarKind::Roc => true,
            AnyCalendarKind::Chinese | AnyCalendarKind::Dangi | AnyCalendarKind::Iso => false,
            _ => false,
        }
    }
}

impl From<PlainDate> for Calendar {
    fn from(value: PlainDate) -> Self {
        value.calendar().clone()
    }
}

impl From<PlainDateTime> for Calendar {
    fn from(value: PlainDateTime) -> Self {
        value.calendar().clone()
    }
}

impl From<ZonedDateTime> for Calendar {
    fn from(value: ZonedDateTime) -> Self {
        value.calendar().clone()
    }
}

impl From<PlainMonthDay> for Calendar {
    fn from(value: PlainMonthDay) -> Self {
        value.calendar().clone()
    }
}

impl From<PlainYearMonth> for Calendar {
    fn from(value: PlainYearMonth) -> Self {
        value.calendar().clone()
    }
}

#[cfg(test)]
mod tests {
    use crate::{iso::IsoDate, options::Unit};
    use core::str::FromStr;

    use super::Calendar;

    #[test]
    fn calendar_from_str_is_case_insensitive() {
        let cal_str = "iSo8601";
        let calendar = Calendar::try_from_utf8(cal_str.as_bytes()).unwrap();
        assert_eq!(calendar, Calendar::default());

        let cal_str = "iSO8601";
        let calendar = Calendar::try_from_utf8(cal_str.as_bytes()).unwrap();
        assert_eq!(calendar, Calendar::default());
    }

    #[test]
    fn calendar_invalid_ascii_value() {
        let cal_str = "İSO8601";
        let _err = Calendar::from_str(cal_str).unwrap_err();

        let cal_str = "\u{0130}SO8601";
        let _err = Calendar::from_str(cal_str).unwrap_err();

        // Verify that an empty calendar is an error.
        let cal_str = "2025-02-07T01:24:00-06:00[u-ca=]";
        let _err = Calendar::from_str(cal_str).unwrap_err();
    }

    #[test]
    fn date_until_largest_year() {
        // tests format: (Date one, PlainDate two, Duration result)
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
            let first = IsoDate::new_unchecked(test.0 .0, test.0 .1, test.0 .2);
            let second = IsoDate::new_unchecked(test.1 .0, test.1 .1, test.1 .2);
            let result = calendar.date_until(&first, &second, Unit::Year).unwrap();
            assert_eq!(
                result.years() as i32,
                test.2 .0,
                "year failed for test \"{test:?}\""
            );
            assert_eq!(
                result.months() as i32,
                test.2 .1,
                "months failed for test \"{test:?}\""
            );
            assert_eq!(
                result.weeks() as i32,
                test.2 .2,
                "weeks failed for test \"{test:?}\""
            );
            assert_eq!(
                result.days(),
                test.2 .3,
                "days failed for test \"{test:?}\""
            );
        }
    }
}
