//! Implementation of `ResolvedCalendarFields`

use tinystr::tinystr;
use tinystr::TinyAsciiStr;

use alloc::format;

use crate::iso::{constrain_iso_day, is_valid_iso_day};
use crate::options::ArithmeticOverflow;
use crate::{TemporalError, TemporalResult};

use crate::components::{calendar::Calendar, PartialDate};

/// `ResolvedCalendarFields` represents the resolved field values necessary for
/// creating a Date from potentially partial values.
#[derive(Debug)]
pub struct ResolvedCalendarFields {
    pub(crate) era_year: EraYear,
    pub(crate) month_code: MonthCode,
    pub(crate) day: u8,
}

impl ResolvedCalendarFields {
    #[inline]
    pub fn try_from_partial_and_calendar(
        calendar: &Calendar,
        partial_date: &PartialDate,
        overflow: ArithmeticOverflow,
    ) -> TemporalResult<Self> {
        let era_year = EraYear::try_from_partial_values_and_calendar(
            partial_date.year,
            partial_date.era,
            partial_date.era_year,
            calendar,
        )?;
        if calendar.is_iso() {
            let month_code =
                resolve_iso_month(partial_date.month_code, partial_date.month, overflow)?;
            let day = partial_date
                .day
                .ok_or(TemporalError::r#type().with_message("Required day field is empty."))?;

            let day = if overflow == ArithmeticOverflow::Constrain {
                constrain_iso_day(era_year.year, ascii_four_to_integer(month_code)?, day)
            } else {
                if !is_valid_iso_day(era_year.year, ascii_four_to_integer(month_code)?, day) {
                    return Err(
                        TemporalError::range().with_message("day value is not in a valid range.")
                    );
                }
                day
            };
            return Ok(Self {
                era_year,
                month_code: MonthCode(month_code),
                day,
            });
        }

        let month_code = MonthCode::try_from_partial_date(partial_date, calendar)?;
        let day = partial_date
            .day
            .ok_or(TemporalError::r#type().with_message("Required day field is empty."))?;

        Ok(Self {
            era_year,
            month_code,
            day,
        })
    }
}

#[derive(Debug)]
pub struct Era(pub(crate) TinyAsciiStr<16>);

#[derive(Debug)]
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
                    return Err(TemporalError::r#type()
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
            _ => Err(TemporalError::r#type()
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

// TODO: Handle instances where month values may be outside of valid
// bounds. In other words, it is totally possible for a value to be
// passed in that is { month: 300 } with overflow::constrain.
/// MonthCode struct v2
#[derive(Debug)]
pub struct MonthCode(pub(crate) TinyAsciiStr<4>);

impl MonthCode {
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
            return Ok(MonthCode(*month_code));
        }

        match calendar.identifier() {
            "chinese" | "dangi" if LUNAR_LEAP_MONTHS.contains(month_code) => {
                Ok(MonthCode(*month_code))
            }
            "coptic" | "ethiopic" | "ethiopicaa" if MONTH_THIRTEEN == *month_code => {
                Ok(MonthCode(*month_code))
            }
            "hebrew" if MONTH_FIVE_LEAP == *month_code => Ok(MonthCode(*month_code)),
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
            _ => Err(TemporalError::r#type()
                .with_message("Month or monthCode is required to determine date.")),
        }
    }

    pub fn as_iso_month_integer(&self) -> TemporalResult<u8> {
        ascii_four_to_integer(self.0)
    }
}

// NOTE: This is a greedy function, should handle differently for all calendars.
pub(crate) fn month_to_month_code(month: u8) -> TemporalResult<TinyAsciiStr<4>> {
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

fn are_month_and_month_code_resolvable(month: u8, mc: &TinyAsciiStr<4>) -> TemporalResult<()> {
    if month != ascii_four_to_integer(*mc)? {
        return Err(TemporalError::range()
            .with_message("Month and monthCode values could not be resolved."));
    }
    Ok(())
}

// NOTE: This is a greedy function, should handle differently for all calendars.
pub(crate) fn ascii_four_to_integer(mc: TinyAsciiStr<4>) -> TemporalResult<u8> {
    match mc {
        MONTH_ONE => Ok(1),
        MONTH_TWO => Ok(2),
        MONTH_THREE => Ok(3),
        MONTH_FOUR => Ok(4),
        MONTH_FIVE => Ok(5),
        MONTH_SIX => Ok(6),
        MONTH_SEVEN => Ok(7),
        MONTH_EIGHT => Ok(8),
        MONTH_NINE => Ok(9),
        MONTH_TEN => Ok(10),
        MONTH_ELEVEN => Ok(11),
        MONTH_TWELVE => Ok(12),
        _ => Err(TemporalError::range()
            .with_message(format!("MonthCode is not supported: {}", mc.as_str()))),
    }
}

fn resolve_iso_month(
    mc: Option<TinyAsciiStr<4>>,
    month: Option<u8>,
    overflow: ArithmeticOverflow,
) -> TemporalResult<TinyAsciiStr<4>> {
    match (mc, month) {
        (None, None) => {
            Err(TemporalError::r#type().with_message("Month or monthCode must be provided."))
        }
        (None, Some(month)) => {
            if overflow == ArithmeticOverflow::Constrain {
                return month_to_month_code(month.clamp(1, 12));
            }
            if !(1..=12).contains(&month) {
                return Err(
                    TemporalError::range().with_message("month value is not in a valid range.")
                );
            }
            month_to_month_code(month)
        }
        (Some(mc), None) => {
            // Check that monthCode is parsable.
            let _ = ascii_four_to_integer(mc)?;
            Ok(mc)
        }
        (Some(mc), Some(month)) => {
            let month_code_int = ascii_four_to_integer(mc)?;
            if month != month_code_int {
                return Err(TemporalError::range()
                    .with_message("month and monthCode could not be resolved."));
            }
            Ok(mc)
        }
    }
}

#[cfg(test)]
mod tests {
    use tinystr::tinystr;

    use crate::{
        components::{calendar::Calendar, PartialDate},
        options::ArithmeticOverflow,
    };

    use super::ResolvedCalendarFields;

    #[test]
    fn day_overflow_test() {
        let bad_fields = PartialDate {
            year: Some(2019),
            month: Some(1),
            day: Some(32),
            ..Default::default()
        };

        let cal = Calendar::default();

        let err = cal.date_from_partial(&bad_fields, ArithmeticOverflow::Reject);
        assert!(err.is_err());
        let result = cal.date_from_partial(&bad_fields, ArithmeticOverflow::Constrain);
        assert!(result.is_ok());
    }

    #[test]
    fn unresolved_month_and_month_code() {
        let bad_fields = PartialDate {
            year: Some(1976),
            month: Some(11),
            month_code: Some(tinystr!(4, "M12")),
            day: Some(18),
            ..Default::default()
        };

        let cal = Calendar::default();
        let err = ResolvedCalendarFields::try_from_partial_and_calendar(
            &cal,
            &bad_fields,
            ArithmeticOverflow::Reject,
        );
        assert!(err.is_err());
    }

    #[test]
    fn missing_partial_fields() {
        let bad_fields = PartialDate {
            year: Some(2019),
            day: Some(19),
            ..Default::default()
        };

        let cal = Calendar::default();
        let err = ResolvedCalendarFields::try_from_partial_and_calendar(
            &cal,
            &bad_fields,
            ArithmeticOverflow::Reject,
        );
        assert!(err.is_err());

        let bad_fields = PartialDate::default();
        let err = ResolvedCalendarFields::try_from_partial_and_calendar(
            &cal,
            &bad_fields,
            ArithmeticOverflow::Reject,
        );
        assert!(err.is_err());
    }
}
