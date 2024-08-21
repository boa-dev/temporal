use std::ops::RangeInclusive;

use tinystr::{tinystr, TinyAsciiStr};

use crate::{TemporalError, TemporalResult};

use super::{
    calendar::{Calendar, CalendarMethods},
    Date, PartialDate, YearMonth,
};

// TODO: Potentially store calendar identifier in `CalendarFields` so that it is self contained.
/// `CalendarFields` represents the static values
pub struct CalendarFields {
    pub(crate) era_year: EraYear,
    pub(crate) month_code: MonthCodeV2,
    pub(crate) day: u8,
}

impl CalendarFields {
    pub fn try_from_partial_and_calendar(
        calendar: &Calendar,
        partial_date: &PartialDate,
    ) -> TemporalResult<Self> {
        let era_year = EraYear::try_from_partial_values_and_calendar(
            partial_date.year,
            partial_date.era,
            partial_date.era_year,
            calendar,
        )?;
        let month_code = MonthCodeV2::try_from_partial_date(partial_date, calendar)?;
        let day = Day::try_from_partial_field(
            partial_date
                .day
                .ok_or(TemporalError::range().with_message("Required day field is empty."))?,
        )?;

        Ok(Self {
            era_year,
            month_code,
            day: day.0,
        })
    }

    /// Create a `CalendarFields1 from a `PartialDate`, falling back to the values provided by date if present.
    pub fn try_from_partial_with_fallback_date(
        calendar: &Calendar,
        partial_date: &PartialDate,
        fallback: &impl CalendarMethods,
    ) -> TemporalResult<Self> {
        let year = partial_date.year.unwrap_or(fallback.year()?);
        let month_code =
            MonthCodeV2::try_from_partial_date_with_fallback(partial_date, calendar, fallback)?;
        let day = Day::try_from_partial_field(partial_date.day.unwrap_or(fallback.day()?.into()))?;
        // TODO: Determine best way to handle era/eraYear.
        let (era, era_year) =
            if let (Some(era), Some(era_year)) = (partial_date.era, partial_date.era_year) {
                (Some(era), Some(era_year))
            } else {
                let era = fallback
                    .era()?
                    .map(|t| TinyAsciiStr::<19>::from_bytes(t.as_bytes()))
                    .transpose()
                    .map_err(|_| TemporalError::general("Invalid era parsing."))?;
                (era, fallback.era_year()?)
            };

        let era_year =
            EraYear::try_from_partial_values_and_calendar(Some(year), era, era_year, calendar)?;

        Ok(Self {
            era_year,
            month_code,
            day: day.0,
        })
    }

    pub(crate) fn try_from_year_month(
        calendar: &Calendar,
        year_month: &YearMonth,
    ) -> TemporalResult<Self> {
        let year = year_month.year()?;
        let month_code = MonthCodeV2(year_month.month_code()?);
        let era = year_month
            .era()?
            .map(|t| TinyAsciiStr::<19>::from_bytes(t.as_bytes()))
            .transpose()
            .map_err(|_| TemporalError::general("Invalid era parsing."))?;
        let era_year = year_month.era_year()?;

        let era_year =
            EraYear::try_from_partial_values_and_calendar(Some(year), era, era_year, calendar)?;

        Ok(Self {
            era_year,
            month_code,
            day: 1,
        })
    }
}

impl TryFrom<&Date> for CalendarFields {
    type Error = TemporalError;

    fn try_from(value: &Date) -> Result<Self, Self::Error> {
        Self::try_from_partial_with_fallback_date(value.calendar(), &PartialDate::default(), value)
    }
}

pub struct Day(u8);

impl Day {
    fn try_from_partial_field(value: i32) -> TemporalResult<Self> {
        if !(1..=31).contains(&value) {
            return Err(
                TemporalError::range().with_message("day value was not within a valid day range.")
            );
        };
        Ok(Self(value as u8))
    }
}

pub struct Era(pub(crate) TinyAsciiStr<16>);

pub struct EraYear {
    pub(crate) era: Era,
    pub(crate) year: i32,
}

impl EraYear {
    pub(crate) fn try_from_partial_values_and_calendar(
        year: Option<i32>,
        era: Option<TinyAsciiStr<19>>,
        era_year: Option<i32>,
        calendar: &Calendar,
    ) -> TemporalResult<Self> {
        match (year, era, era_year) {
            (Some(year), None, None) => {
                let Some(era) = calendar.get_calendar_default_era() else {
                    return Err(TemporalError::range()
                        .with_message("Era is required for the provided calendar."));
                };
                Ok(Self {
                    era: Era(era.name),
                    year,
                })
            }
            (None, Some(era), Some(era_year)) => {
                let Some(era_info) = calendar.get_era_info(&era) else {
                    return Err(TemporalError::range().with_message("Invalid era provided."));
                };
                if !era_info.range.contains(&era_year) {
                    return Err(TemporalError::range().with_message(format!(
                        "Year is not valid for the era {}",
                        era_info.name.as_str()
                    )));
                }
                Ok(Self {
                    year: era_year,
                    era: Era(era_info.name),
                })
            }
            _ => Err(TemporalError::range()
                .with_message("Required fields missing to determine an era and year.")),
        }
    }
}

// MonthCode constants.
const MONTH_ONE: TinyAsciiStr<4> = tinystr!(4, "M01");
const MONTH_ONE_LEAP: TinyAsciiStr<4> = tinystr!(4, "M01L");
const MONTH_TWO: TinyAsciiStr<4> = tinystr!(4, "M02");
const MONTH_TWO_LEAP: TinyAsciiStr<4> = tinystr!(4, "M02L");
const MONTH_THREE: TinyAsciiStr<4> = tinystr!(4, "M03");
const MONTH_THREE_LEAP: TinyAsciiStr<4> = tinystr!(4, "M03L");
const MONTH_FOUR: TinyAsciiStr<4> = tinystr!(4, "M04");
const MONTH_FOUR_LEAP: TinyAsciiStr<4> = tinystr!(4, "M04L");
const MONTH_FIVE: TinyAsciiStr<4> = tinystr!(4, "M05");
const MONTH_FIVE_LEAP: TinyAsciiStr<4> = tinystr!(4, "M05L");
const MONTH_SIX: TinyAsciiStr<4> = tinystr!(4, "M06");
const MONTH_SIX_LEAP: TinyAsciiStr<4> = tinystr!(4, "M06L");
const MONTH_SEVEN: TinyAsciiStr<4> = tinystr!(4, "M07");
const MONTH_SEVEN_LEAP: TinyAsciiStr<4> = tinystr!(4, "M07L");
const MONTH_EIGHT: TinyAsciiStr<4> = tinystr!(4, "M08");
const MONTH_EIGHT_LEAP: TinyAsciiStr<4> = tinystr!(4, "M08L");
const MONTH_NINE: TinyAsciiStr<4> = tinystr!(4, "M09");
const MONTH_NINE_LEAP: TinyAsciiStr<4> = tinystr!(4, "M09L");
const MONTH_TEN: TinyAsciiStr<4> = tinystr!(4, "M10");
const MONTH_TEN_LEAP: TinyAsciiStr<4> = tinystr!(4, "M10L");
const MONTH_ELEVEN: TinyAsciiStr<4> = tinystr!(4, "M11");
const MONTH_ELEVEN_LEAP: TinyAsciiStr<4> = tinystr!(4, "M11L");
const MONTH_TWELVE: TinyAsciiStr<4> = tinystr!(4, "M12");
const MONTH_TWELVE_LEAP: TinyAsciiStr<4> = tinystr!(4, "M12L");
const MONTH_THIRTEEN: TinyAsciiStr<4> = tinystr!(4, "M13");

/// MonthCode struct v2
pub struct MonthCodeV2(pub(crate) TinyAsciiStr<4>);

impl MonthCodeV2 {
    pub fn try_new(month_code: &TinyAsciiStr<4>, calendar: &Calendar) -> TemporalResult<Self> {
        const COMMON_MONTH_CODES: [TinyAsciiStr<4>; 12] = [
            MONTH_ONE,
            MONTH_TWO,
            MONTH_THREE,
            MONTH_FOUR,
            MONTH_FIVE,
            MONTH_SIX,
            MONTH_SEVEN,
            MONTH_EIGHT,
            MONTH_NINE,
            MONTH_TEN,
            MONTH_ELEVEN,
            MONTH_TWELVE,
        ];

        const LUNAR_LEAP_MONTHS: [TinyAsciiStr<4>; 12] = [
            MONTH_ONE_LEAP,
            MONTH_TWO_LEAP,
            MONTH_THREE_LEAP,
            MONTH_FOUR_LEAP,
            MONTH_FIVE_LEAP,
            MONTH_SIX_LEAP,
            MONTH_SEVEN_LEAP,
            MONTH_EIGHT_LEAP,
            MONTH_NINE_LEAP,
            MONTH_TEN_LEAP,
            MONTH_ELEVEN_LEAP,
            MONTH_TWELVE_LEAP,
        ];

        if COMMON_MONTH_CODES.contains(month_code) {
            return Ok(MonthCodeV2(*month_code));
        }

        match calendar.identifier() {
            "chinese" | "dangi" if LUNAR_LEAP_MONTHS.contains(month_code) => {
                Ok(MonthCodeV2(*month_code))
            }
            "coptic" | "ethiopic" | "ethiopicaa" if MONTH_THIRTEEN == *month_code => {
                Ok(MonthCodeV2(*month_code))
            }
            "hebrew" if MONTH_FIVE_LEAP == *month_code => Ok(MonthCodeV2(*month_code)),
            _ => Err(TemporalError::range()
                .with_message("MonthCode was not valid for the current calendar.")),
        }
    }

    pub(crate) fn try_from_partial_date(
        partial_date: &PartialDate,
        calendar: &Calendar,
    ) -> TemporalResult<Self> {
        match partial_date {
            PartialDate {
                month: Some(month),
                month_code: None,
                ..
            } => Self::try_new(&month_to_month_code(*month)?, calendar),
            PartialDate {
                month_code: Some(month_code),
                month: None,
                ..
            } => Self::try_new(month_code, calendar),
            PartialDate {
                month: Some(month),
                month_code: Some(month_code),
                ..
            } => {
                are_month_and_month_code_resolvable(*month, month_code)?;
                Self::try_new(month_code, calendar)
            }
            _ => Err(TemporalError::range()
                .with_message("Month code needed is required to determine date.")),
        }
    }

    pub(crate) fn try_from_partial_date_with_fallback(
        partial: &PartialDate,
        calendar: &Calendar,
        fallback: &impl CalendarMethods,
    ) -> TemporalResult<Self> {
        match partial {
            PartialDate {
                month: Some(month),
                month_code: None,
                ..
            } => Self::try_new(&month_to_month_code(*month)?, calendar),
            PartialDate {
                month_code: Some(month_code),
                month: None,
                ..
            } => Self::try_new(month_code, calendar),
            PartialDate {
                month: Some(month),
                month_code: Some(month_code),
                ..
            } => {
                are_month_and_month_code_resolvable(*month, month_code)?;
                Self::try_new(month_code, calendar)
            }
            PartialDate {
                month: None,
                month_code: None,
                ..
            } => Ok(Self(fallback.month_code()?)),
        }
    }

    pub fn as_month_integer(&self) -> TemporalResult<u8> {
        ascii_four_to_integer(self.0)
    }
}

fn month_to_month_code(month: i32) -> TemporalResult<TinyAsciiStr<4>> {
    match month {
        1 => Ok(MONTH_ONE),
        2 => Ok(MONTH_TWO),
        3 => Ok(MONTH_THREE),
        4 => Ok(MONTH_FOUR),
        5 => Ok(MONTH_FIVE),
        6 => Ok(MONTH_SIX),
        7 => Ok(MONTH_SEVEN),
        8 => Ok(MONTH_EIGHT),
        9 => Ok(MONTH_NINE),
        10 => Ok(MONTH_TEN),
        11 => Ok(MONTH_ELEVEN),
        12 => Ok(MONTH_TWELVE),
        13 => Ok(MONTH_THIRTEEN),
        _ => Err(TemporalError::range().with_message("Month not in a valid range.")),
    }
}

fn are_month_and_month_code_resolvable(month: i32, mc: &TinyAsciiStr<4>) -> TemporalResult<()> {
    if month != ascii_four_to_integer(*mc)?.into() {
        return Err(TemporalError::range()
            .with_message("Month and monthCode values could not be resolved."));
    }
    Ok(())
}

fn ascii_four_to_integer(mc: TinyAsciiStr<4>) -> TemporalResult<u8> {
    match mc {
        MONTH_ONE | MONTH_ONE_LEAP => Ok(1),
        MONTH_TWO | MONTH_TWO_LEAP => Ok(2),
        MONTH_THREE | MONTH_THREE_LEAP => Ok(3),
        MONTH_FOUR | MONTH_FOUR_LEAP => Ok(4),
        MONTH_FIVE | MONTH_FIVE_LEAP => Ok(5),
        MONTH_SIX | MONTH_SIX_LEAP => Ok(6),
        MONTH_SEVEN | MONTH_SEVEN_LEAP => Ok(7),
        MONTH_EIGHT | MONTH_EIGHT_LEAP => Ok(8),
        MONTH_NINE | MONTH_NINE_LEAP => Ok(9),
        MONTH_TEN | MONTH_TEN_LEAP => Ok(10),
        MONTH_ELEVEN | MONTH_ELEVEN_LEAP => Ok(11),
        MONTH_TWELVE | MONTH_TWELVE_LEAP => Ok(12),
        MONTH_THIRTEEN => Ok(13),
        _ => Err(TemporalError::range()
            .with_message(format!("Invalid monthCode value: {}", mc.as_str()))),
    }
}

// TODO: Update era size
pub(crate) struct EraInfo {
    name: TinyAsciiStr<16>,
    range: RangeInclusive<i32>,
}

macro_rules! era_identifier {
    ($name:literal) => {
        tinystr!(19, $name)
    };
}

macro_rules! valid_era {
    ($name:literal, $range:expr ) => {
        EraInfo {
            name: tinystr!(16, $name),
            range: $range,
        }
    };
}

pub(crate) const BUDDHIST_ERA_IDENTIFIERS: [TinyAsciiStr<19>; 2] =
    [era_identifier!("buddhist"), era_identifier!("be")];

pub(crate) const ETHIOPIC_ERA_IDENTIFIERS: [TinyAsciiStr<19>; 2] =
    [era_identifier!("ethiopic"), era_identifier!("incar")];

pub(crate) const ETHIOPIC_ETHOPICAA_ERA_IDENTIFIERS: [TinyAsciiStr<19>; 3] = [
    era_identifier!("ethiopicaa"),
    era_identifier!("ethiopic-amete-alem"), // TODO: probably will break?
    era_identifier!("mundi"),
];

pub(crate) const ETHOPICAA_ERA_IDENTIFIERS: [TinyAsciiStr<19>; 3] = [
    era_identifier!("ethiopicaa"),
    era_identifier!("ethiopic-amete-alem"), // TODO: probably will break?
    era_identifier!("mundi"),
];

pub(crate) const GREGORY_ERA_IDENTIFIERS: [TinyAsciiStr<19>; 3] = [
    era_identifier!("gregory"),
    era_identifier!("ce"),
    era_identifier!("ad"),
];

pub(crate) const GREGORY_INVERSE_ERA_IDENTIFIERS: [TinyAsciiStr<19>; 3] = [
    era_identifier!("gregory-inverse"),
    era_identifier!("bc"),
    era_identifier!("bce"),
];

pub(crate) const HEBREW_ERA_IDENTIFIERS: [TinyAsciiStr<19>; 2] =
    [era_identifier!("hebrew"), era_identifier!("am")];

pub(crate) const INDIAN_ERA_IDENTIFIERS: [TinyAsciiStr<19>; 2] =
    [era_identifier!("indian"), era_identifier!("saka")];

pub(crate) const ISLAMIC_ERA_IDENTIFIERS: [TinyAsciiStr<19>; 2] =
    [era_identifier!("islamic"), era_identifier!("ah")];

pub(crate) const ISLAMIC_CIVIL_ERA_IDENTIFIERS: [TinyAsciiStr<19>; 3] = [
    era_identifier!("islamic-civil"),
    era_identifier!("islamicc"),
    era_identifier!("ah"),
];

// TODO: Support islamic-rgsa
pub(crate) const _ISLAMIC_RGSA_ERA_IDENTIFIERS: [TinyAsciiStr<19>; 2] =
    [era_identifier!("islamic-rgsa"), era_identifier!("ah")];

pub(crate) const ISLAMIC_TBLA_ERA_IDENTIFIERS: [TinyAsciiStr<19>; 2] =
    [era_identifier!("islamic-tbla"), era_identifier!("ah")];

pub(crate) const ISLAMIC_UMALQURA_ERA_IDENTIFIERS: [TinyAsciiStr<19>; 2] =
    [era_identifier!("islamic-umalqura"), era_identifier!("ah")];

pub(crate) const JAPANESE_ERA_IDENTIFIERS: [TinyAsciiStr<19>; 4] = [
    era_identifier!("japanese"),
    era_identifier!("gregory"),
    era_identifier!("ce"),
    era_identifier!("ad"),
];

pub(crate) const JAPANESE_INVERSE_ERA_IDENTIFIERS: [TinyAsciiStr<19>; 4] = [
    era_identifier!("japanese-inverse"),
    era_identifier!("gregory-inverse"),
    era_identifier!("bc"),
    era_identifier!("bce"),
];

pub(crate) const PERSIAN_ERA_IDENTIFIERS: [TinyAsciiStr<19>; 2] =
    [era_identifier!("persian"), era_identifier!("ap")];

pub(crate) const ROC_ERA_IDENTIFIERS: [TinyAsciiStr<19>; 2] =
    [era_identifier!("roc"), era_identifier!("minguo")];

pub(crate) const ROC_INVERSE_ERA_IDENTIFIERS: [TinyAsciiStr<19>; 2] = [
    era_identifier!("roc-inverse"),
    era_identifier!("before-roc"),
];

pub(crate) const ISO_ERA: EraInfo = valid_era!("default", i32::MIN..=i32::MAX);
pub(crate) const BUDDHIST_ERA: EraInfo = valid_era!("buddhist", i32::MIN..=i32::MAX);
pub(crate) const CHINESE_ERA: EraInfo = valid_era!("chinese", i32::MIN..=i32::MAX);
pub(crate) const COPTIC_ERA: EraInfo = valid_era!("coptic", 1..=i32::MAX);
pub(crate) const COPTIC_INVERSE_ERA: EraInfo = valid_era!("coptic-inverse", 1..=i32::MAX);
pub(crate) const DANGI_ERA: EraInfo = valid_era!("dangi", i32::MIN..=i32::MAX);
pub(crate) const ETHIOPIC_ERA: EraInfo = valid_era!("ethiopic", 1..=i32::MAX);
pub(crate) const ETHIOPIC_ETHIOPICAA_ERA: EraInfo = valid_era!("ethiopicaa", i32::MIN..=5500);
pub(crate) const ETHIOPICAA_ERA: EraInfo = valid_era!("ethiopicaa", i32::MIN..=i32::MAX);
pub(crate) const GREGORY_ERA: EraInfo = valid_era!("gregory", 1..=i32::MAX);
pub(crate) const GREGORY_INVERSE_ERA: EraInfo = valid_era!("gregory-inverse", 1..=i32::MAX);
pub(crate) const HEBREW_ERA: EraInfo = valid_era!("hebrew", i32::MIN..=i32::MAX);
pub(crate) const INDIAN_ERA: EraInfo = valid_era!("indian", i32::MIN..=i32::MAX);
pub(crate) const ISLAMIC_ERA: EraInfo = valid_era!("islamic", i32::MIN..=i32::MAX);
pub(crate) const ISLAMIC_CIVIL_ERA: EraInfo = valid_era!("islamic-civil", i32::MIN..=i32::MAX);
// TODO: Support islamic-rgsa
pub(crate) const _ISLAMIC_RGSA_ERA: EraInfo = valid_era!("islamic-rgsa", i32::MIN..=i32::MAX);
pub(crate) const ISLAMIC_TBLA_ERA: EraInfo = valid_era!("islamic-tbla", i32::MIN..=i32::MAX);
pub(crate) const ISLAMIC_UMALQURA_ERA: EraInfo =
    valid_era!("islamic-umalqura", i32::MIN..=i32::MAX);
pub(crate) const HEISEI_ERA: EraInfo = valid_era!("heisei", 1..=31);
pub(crate) const JAPANESE_ERA: EraInfo = valid_era!("japanese", 1..=1868);
pub(crate) const JAPANESE_INVERSE_ERA: EraInfo = valid_era!("japanese-inverse", 1..=i32::MAX);
pub(crate) const MEJEI_ERA: EraInfo = valid_era!("mejei", 1..=45);
pub(crate) const REIWA_ERA: EraInfo = valid_era!("reiwa", 1..=i32::MAX);
pub(crate) const SHOWA_ERA: EraInfo = valid_era!("showa", 1..=64);
pub(crate) const TAISHO_ERA: EraInfo = valid_era!("showa", 1..=45);
pub(crate) const PERSIAN_ERA: EraInfo = valid_era!("persian", i32::MIN..=i32::MAX);
pub(crate) const ROC_ERA: EraInfo = valid_era!("roc", 1..=i32::MAX);
pub(crate) const ROC_INVERSE_ERA: EraInfo = valid_era!("roc-inverse", 1..=i32::MAX);
