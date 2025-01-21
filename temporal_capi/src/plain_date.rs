use crate::error::ffi::TemporalError;

#[diplomat::bridge]
#[diplomat::abi_rename = "temporal_rs_{0}"]
#[diplomat::attr(auto, namespace = "temporal_rs")]
pub mod ffi {
    use crate::calendar::ffi::Calendar;
    use crate::duration::ffi::Duration;
    use crate::error::ffi::TemporalError;
    use crate::options::ffi::{ArithmeticOverflow, DifferenceSettings, DisplayCalendar};
    use diplomat_runtime::{DiplomatOption, DiplomatStrSlice, DiplomatWrite};
    use std::fmt::Write;

    #[diplomat::opaque]
    pub struct PlainDate(pub(crate) temporal_rs::PlainDate);

    pub struct PartialDate<'a> {
        pub year: DiplomatOption<i32>,
        pub month: DiplomatOption<u8>,
        // None if empty
        pub month_code: DiplomatStrSlice<'a>,
        pub day: DiplomatOption<u8>,
        // None if empty
        pub era: DiplomatStrSlice<'a>,
        pub era_year: DiplomatOption<i32>,
        pub calendar: &'a Calendar,
    }

    impl PlainDate {
        pub fn create(
            year: i32,
            month: u8,
            day: u8,
            calendar: &Calendar,
        ) -> Result<Box<Self>, TemporalError> {
            temporal_rs::PlainDate::new(year, month, day, calendar.0.clone())
                .map(|x| Box::new(PlainDate(x)))
                .map_err(Into::into)
        }
        pub fn try_create(
            year: i32,
            month: u8,
            day: u8,
            calendar: &Calendar,
        ) -> Result<Box<Self>, TemporalError> {
            temporal_rs::PlainDate::try_new(year, month, day, calendar.0.clone())
                .map(|x| Box::new(PlainDate(x)))
                .map_err(Into::into)
        }
        pub fn create_with_overflow(
            year: i32,
            month: u8,
            day: u8,
            calendar: &Calendar,
            overflow: ArithmeticOverflow,
        ) -> Result<Box<Self>, TemporalError> {
            temporal_rs::PlainDate::new_with_overflow(
                year,
                month,
                day,
                calendar.0.clone(),
                overflow.into(),
            )
            .map(|x| Box::new(PlainDate(x)))
            .map_err(Into::into)
        }
        pub fn from_partial(
            partial: PartialDate,
            overflow: Option<ArithmeticOverflow>,
        ) -> Result<Box<Self>, TemporalError> {
            temporal_rs::PlainDate::from_partial(partial.try_into()?, overflow.map(Into::into))
                .map(|x| Box::new(PlainDate(x)))
                .map_err(Into::into)
        }
        pub fn with(
            &self,
            partial: PartialDate,
            overflow: Option<ArithmeticOverflow>,
        ) -> Result<Box<Self>, TemporalError> {
            self.0
                .with(partial.try_into()?, overflow.map(Into::into))
                .map(|x| Box::new(PlainDate(x)))
                .map_err(Into::into)
        }

        pub fn with_calendar(&self, calendar: &Calendar) -> Result<Box<Self>, TemporalError> {
            self.0
                .with_calendar(calendar.0.clone())
                .map(|x| Box::new(PlainDate(x)))
                .map_err(Into::into)
        }

        pub fn iso_year(&self) -> i32 {
            self.0.iso_year()
        }
        pub fn iso_month(&self) -> u8 {
            self.0.iso_month()
        }
        pub fn iso_day(&self) -> u8 {
            self.0.iso_day()
        }

        pub fn calendar<'a>(&'a self) -> &'a Calendar {
            Calendar::transparent_convert(self.0.calendar())
        }

        pub fn is_valid(&self) -> bool {
            self.0.is_valid()
        }

        pub fn days_until(&self, other: &Self) -> i32 {
            self.0.days_until(&other.0)
        }

        pub fn add(
            &self,
            duration: &Duration,
            overflow: Option<ArithmeticOverflow>,
        ) -> Result<Box<Self>, TemporalError> {
            self.0
                .add(&duration.0, overflow.map(Into::into))
                .map(|x| Box::new(Self(x)))
                .map_err(Into::into)
        }
        pub fn subtract(
            &self,
            duration: &Duration,
            overflow: Option<ArithmeticOverflow>,
        ) -> Result<Box<Self>, TemporalError> {
            self.0
                .subtract(&duration.0, overflow.map(Into::into))
                .map(|x| Box::new(Self(x)))
                .map_err(Into::into)
        }
        pub fn until(
            &self,
            other: &Self,
            settings: DifferenceSettings,
        ) -> Result<Box<Duration>, TemporalError> {
            self.0
                .until(&other.0, settings.try_into()?)
                .map(|x| Box::new(Duration(x)))
                .map_err(Into::into)
        }
        pub fn since(
            &self,
            other: &Self,
            settings: DifferenceSettings,
        ) -> Result<Box<Duration>, TemporalError> {
            self.0
                .since(&other.0, settings.try_into()?)
                .map(|x| Box::new(Duration(x)))
                .map_err(Into::into)
        }

        pub fn year(&self) -> Result<i32, TemporalError> {
            self.0.year().map_err(Into::into)
        }
        pub fn month(&self) -> Result<u8, TemporalError> {
            self.0.month().map_err(Into::into)
        }
        pub fn month_code(&self, write: &mut DiplomatWrite) -> Result<(), TemporalError> {
            let code = self.0.month_code().map_err(Into::<TemporalError>::into)?;
            // throw away the error, this should always succeed
            let _ = write.write_str(&code);
            Ok(())
        }
        pub fn day(&self) -> Result<u8, TemporalError> {
            self.0.day().map_err(Into::into)
        }
        pub fn day_of_week(&self) -> Result<u16, TemporalError> {
            self.0.day_of_week().map_err(Into::into)
        }
        pub fn day_of_year(&self) -> Result<u16, TemporalError> {
            self.0.day_of_year().map_err(Into::into)
        }
        pub fn week_of_year(&self) -> Result<Option<u16>, TemporalError> {
            self.0.week_of_year().map_err(Into::into)
        }
        pub fn year_of_week(&self) -> Result<Option<i32>, TemporalError> {
            self.0.year_of_week().map_err(Into::into)
        }
        pub fn days_in_week(&self) -> Result<u16, TemporalError> {
            self.0.days_in_week().map_err(Into::into)
        }
        pub fn days_in_month(&self) -> Result<u16, TemporalError> {
            self.0.days_in_month().map_err(Into::into)
        }
        pub fn days_in_year(&self) -> Result<u16, TemporalError> {
            self.0.days_in_year().map_err(Into::into)
        }
        pub fn months_in_year(&self) -> Result<u16, TemporalError> {
            self.0.months_in_year().map_err(Into::into)
        }
        pub fn in_leap_year(&self) -> Result<bool, TemporalError> {
            self.0.in_leap_year().map_err(Into::into)
        }
        // Writes an empty string for no era
        pub fn era(&self, write: &mut DiplomatWrite) -> Result<(), TemporalError> {
            let era = self.0.era().map_err(Into::<TemporalError>::into)?;
            if let Some(era) = era {
                // throw away the error, this should always succeed
                let _ = write.write_str(&era);
            }
            Ok(())
        }

        pub fn era_year(&self) -> Result<Option<i32>, TemporalError> {
            self.0.era_year().map_err(Into::into)
        }

        // TODO conversions (needs other date/time types)

        pub fn to_ixdtf_string(
            &self,
            display_calendar: DisplayCalendar,
            write: &mut DiplomatWrite,
        ) {
            // TODO this double-allocates, an API returning a Writeable or impl Write would be better
            let string = self.0.to_ixdtf_string(display_calendar.into());
            // throw away the error, this should always succeed
            let _ = write.write_str(&string);
        }
    }
}

impl TryFrom<ffi::PartialDate<'_>> for temporal_rs::partial::PartialDate {
    type Error = TemporalError;
    fn try_from(other: ffi::PartialDate<'_>) -> Result<Self, TemporalError> {
        use temporal_rs::TinyAsciiStr;

        let month_code = if other.month_code.is_empty() {
            None
        } else {
            Some(
                TinyAsciiStr::try_from_utf8(other.month_code.into())
                    .map_err(|_| TemporalError::syntax())?,
            )
        };

        let era = if other.era.is_empty() {
            None
        } else {
            Some(
                TinyAsciiStr::try_from_utf8(other.era.into())
                    .map_err(|_| TemporalError::syntax())?,
            )
        };
        Ok(Self {
            year: other.year.into(),
            month: other.month.into(),
            month_code,
            day: other.day.into(),
            era_year: other.era_year.into(),
            era,
            calendar: other.calendar.0.clone(),
        })
    }
}
