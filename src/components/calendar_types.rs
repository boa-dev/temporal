use tinystr::{tinystr, TinyAsciiStr};

use crate::{TemporalError, TemporalResult};

use super::{calendar::Calendar, PartialDate};

// TODO: Deal with "ethiopic-amete-alem" in relation to Era.
struct CalendarFields {
    calendar: &'static str,
    year: EraYear,
    month: u8,
    month_code: MonthCodeV2,
    day: u8,
}

impl CalendarFields {
    pub fn try_from_partial_and_calendar(
        partial_date: &PartialDate,
        calendar: &Calendar,
    ) -> TemporalResult<Self> {
        todo!()
    }
}

pub struct EraYear {
    era: Era,
    year: i32,
}

impl EraYear {
    pub(crate) fn try_from_partial_and_calendar(
        partial_date: &PartialDate,
        calendar: &Calendar,
    ) -> TemporalResult<Self> {
        match partial_date {
            PartialDate { year: Some(year), era: None, era_year: None, ..} => {
                if matches!(calendar.identifier(), "coptic" | "ethiopic" | "gregory" | "japanese" | "roc") {
                    return Err(TemporalError::range()
                        .with_message(format!("Era and eraYear are required for the {} calendar.", calendar.identifier())))
                }

            },
            PartialDate { era: Some(era), era_year: Some(era_year), ..} => {
            },
            _=> {},
        }


        todo!()
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
    pub fn try_new(month_code: TinyAsciiStr<4>, calendar: &Calendar) -> TemporalResult<Self> {
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

        if COMMON_MONTH_CODES.contains(&month_code) {
            return Ok(MonthCodeV2(month_code));
        }

        match calendar.identifier() {
            "chinese" | "dangi" if LUNAR_LEAP_MONTHS.contains(&month_code) => {
                Ok(MonthCodeV2(month_code))
            }
            "coptic" | "ethiopic" | "ethiopicaa" if MONTH_THIRTEEN == month_code => {
                Ok(MonthCodeV2(month_code))
            }
            "hebrew" if MONTH_FIVE_LEAP == month_code => Ok(MonthCodeV2(month_code)),
            _ => Err(TemporalError::range()
                .with_message("MonthCode was not valid for the current calendar.")),
        }
    }

    pub(crate) fn as_month_integer(&self) -> u8 {
        match self.0 {
            MONTH_ONE | MONTH_ONE_LEAP => 1,
            MONTH_TWO | MONTH_TWO_LEAP => 2,
            MONTH_THREE | MONTH_THREE_LEAP => 3,
            MONTH_FOUR | MONTH_FOUR_LEAP => 4,
            MONTH_FIVE | MONTH_FIVE_LEAP => 5,
            MONTH_SIX | MONTH_SIX_LEAP => 6,
            MONTH_SEVEN | MONTH_SEVEN_LEAP => 7,
            MONTH_EIGHT | MONTH_EIGHT_LEAP => 8,
            MONTH_NINE | MONTH_NINE_LEAP => 9,
            MONTH_TEN | MONTH_TEN_LEAP => 10,
            MONTH_ELEVEN | MONTH_ELEVEN_LEAP => 11,
            MONTH_TWELVE | MONTH_TWELVE_LEAP => 12,
            MONTH_THIRTEEN => 13,
            _ => unreachable!(),
        }
    }
}



pub struct Era(pub(crate) TinyAsciiStr<16>);

impl Era {
    pub(crate) fn try_new(era: TinyAsciiStr::<16>, calendar: &'static str) -> TemporalResult<Self> {
        todo!()
    }
}
