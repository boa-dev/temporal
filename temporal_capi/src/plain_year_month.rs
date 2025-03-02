#[diplomat::bridge]
#[diplomat::abi_rename = "temporal_rs_{0}"]
#[diplomat::attr(auto, namespace = "temporal_rs")]
pub mod ffi {
    use crate::calendar::ffi::Calendar;
    use crate::duration::ffi::Duration;
    use crate::error::ffi::TemporalError;

    use crate::options::ffi::{ArithmeticOverflow, DifferenceSettings};
    use crate::plain_date::ffi::{PartialDate, PlainDate};
    use diplomat_runtime::DiplomatWrite;
    use std::fmt::Write;

    #[diplomat::opaque]
    pub struct PlainYearMonth(pub(crate) temporal_rs::PlainYearMonth);

    impl PlainYearMonth {
        pub fn create_with_overflow(
            year: i32,
            month: u8,
            reference_day: Option<u8>,
            calendar: &Calendar,
            overflow: ArithmeticOverflow,
        ) -> Result<Box<Self>, TemporalError> {
            temporal_rs::PlainYearMonth::new_with_overflow(
                year,
                month,
                reference_day,
                calendar.0.clone(),
                overflow.into(),
            )
            .map(|x| Box::new(PlainYearMonth(x)))
            .map_err(Into::into)
        }

        pub fn with(
            &self,
            partial: PartialDate,
            overflow: Option<ArithmeticOverflow>,
        ) -> Result<Box<Self>, TemporalError> {
            self.0
                .with(partial.try_into()?, overflow.map(Into::into))
                .map(|x| Box::new(Self(x)))
                .map_err(Into::into)
        }

        pub fn iso_year(&self) -> i32 {
            self.0.iso_year()
        }

        pub fn padded_iso_year_string(&self, write: &mut DiplomatWrite) {
            // TODO this double-allocates, an API returning a Writeable or impl Write would be better
            let string = self.0.padded_iso_year_string();
            // throw away the error, the write itself should always succeed
            let _ = write.write_str(&string);
        }

        pub fn iso_month(&self) -> u8 {
            self.0.iso_month()
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
            let _ = write.write_str(code.as_str());
            Ok(())
        }

        pub fn in_leap_year(&self) -> bool {
            self.0.in_leap_year()
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

        pub fn calendar<'a>(&'a self) -> &'a Calendar {
            Calendar::transparent_convert(self.0.calendar())
        }
        pub fn add(
            &self,
            duration: &Duration,
            overflow: ArithmeticOverflow,
        ) -> Result<Box<Self>, TemporalError> {
            self.0
                .add(&duration.0, overflow.into())
                .map(|x| Box::new(Self(x)))
                .map_err(Into::into)
        }
        pub fn subtract(
            &self,
            duration: &Duration,
            overflow: ArithmeticOverflow,
        ) -> Result<Box<Self>, TemporalError> {
            self.0
                .subtract(&duration.0, overflow.into())
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
        pub fn to_plain_date(&self) -> Result<Box<PlainDate>, TemporalError> {
            self.0
                .to_plain_date()
                .map(|x| Box::new(PlainDate(x)))
                .map_err(Into::into)
        }
    }
}
