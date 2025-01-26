use crate::error::ffi::TemporalError;

#[diplomat::bridge]
#[diplomat::abi_rename = "temporal_rs_{0}"]
#[diplomat::attr(auto, namespace = "temporal_rs")]
pub mod ffi {
    use crate::calendar::ffi::Calendar;
    use crate::duration::ffi::Duration;
    use crate::error::ffi::TemporalError;

    use crate::options::ffi::{
        ArithmeticOverflow, DifferenceSettings, DisplayCalendar, RoundingOptions,
        ToStringRoundingOptions,
    };
    use crate::plain_date::ffi::PartialDate;
    use crate::plain_time::ffi::{PartialTime, PlainTime};
    use diplomat_runtime::DiplomatWrite;
    use std::fmt::Write;

    #[diplomat::opaque]
    pub struct PlainDateTime(pub(crate) temporal_rs::PlainDateTime);

    pub struct PartialDateTime<'a> {
        pub date: PartialDate<'a>,
        pub time: PartialTime,
    }

    impl PlainDateTime {
        pub fn create(
            year: i32,
            month: u8,
            day: u8,
            hour: u8,
            minute: u8,
            second: u8,
            millisecond: u16,
            microsecond: u16,
            nanosecond: u16,
            calendar: &Calendar,
        ) -> Result<Box<Self>, TemporalError> {
            temporal_rs::PlainDateTime::new(
                year,
                month,
                day,
                hour,
                minute,
                second,
                millisecond,
                microsecond,
                nanosecond,
                calendar.0.clone(),
            )
            .map(|x| Box::new(PlainDateTime(x)))
            .map_err(Into::into)
        }
        pub fn try_create(
            year: i32,
            month: u8,
            day: u8,
            hour: u8,
            minute: u8,
            second: u8,
            millisecond: u16,
            microsecond: u16,
            nanosecond: u16,
            calendar: &Calendar,
        ) -> Result<Box<Self>, TemporalError> {
            temporal_rs::PlainDateTime::try_new(
                year,
                month,
                day,
                hour,
                minute,
                second,
                millisecond,
                microsecond,
                nanosecond,
                calendar.0.clone(),
            )
            .map(|x| Box::new(PlainDateTime(x)))
            .map_err(Into::into)
        }

        pub fn from_partial(
            partial: PartialDateTime,
            overflow: Option<ArithmeticOverflow>,
        ) -> Result<Box<Self>, TemporalError> {
            temporal_rs::PlainDateTime::from_partial(partial.try_into()?, overflow.map(Into::into))
                .map(|x| Box::new(PlainDateTime(x)))
                .map_err(Into::into)
        }
        pub fn with(
            &self,
            partial: PartialDateTime,
            overflow: Option<ArithmeticOverflow>,
        ) -> Result<Box<Self>, TemporalError> {
            self.0
                .with(partial.try_into()?, overflow.map(Into::into))
                .map(|x| Box::new(PlainDateTime(x)))
                .map_err(Into::into)
        }

        pub fn with_time(&self, time: &PlainTime) -> Result<Box<Self>, TemporalError> {
            self.0
                .with_time(time.0)
                .map(|x| Box::new(PlainDateTime(x)))
                .map_err(Into::into)
        }

        pub fn with_calendar(&self, calendar: &Calendar) -> Result<Box<Self>, TemporalError> {
            self.0
                .with_calendar(calendar.0.clone())
                .map(|x| Box::new(PlainDateTime(x)))
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

        pub fn hour(&self) -> u8 {
            self.0.hour()
        }
        pub fn minute(&self) -> u8 {
            self.0.minute()
        }
        pub fn second(&self) -> u8 {
            self.0.second()
        }
        pub fn millisecond(&self) -> u16 {
            self.0.millisecond()
        }
        pub fn microsecond(&self) -> u16 {
            self.0.microsecond()
        }
        pub fn nanosecond(&self) -> u16 {
            self.0.nanosecond()
        }

        pub fn calendar<'a>(&'a self) -> &'a Calendar {
            Calendar::transparent_convert(self.0.calendar())
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

        pub fn round(&self, options: RoundingOptions) -> Result<Box<Self>, TemporalError> {
            self.0
                .round(options.try_into()?)
                .map(|x| Box::new(Self(x)))
                .map_err(Into::into)
        }

        pub fn to_ixdtf_string(
            &self,
            options: ToStringRoundingOptions,

            display_calendar: DisplayCalendar,
            write: &mut DiplomatWrite,
        ) -> Result<(), TemporalError> {
            // TODO this double-allocates, an API returning a Writeable or impl Write would be better
            let string = self
                .0
                .to_ixdtf_string(options.into(), display_calendar.into())?;
            // throw away the error, this should always succeed
            let _ = write.write_str(&string);
            Ok(())
        }
    }
}

impl TryFrom<ffi::PartialDateTime<'_>> for temporal_rs::partial::PartialDateTime {
    type Error = TemporalError;
    fn try_from(other: ffi::PartialDateTime<'_>) -> Result<Self, TemporalError> {
        Ok(Self {
            date: other.date.try_into()?,
            time: other.time.into(),
        })
    }
}
