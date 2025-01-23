//! RelativeTo rounding option

use alloc::string::String;

use crate::builtins::core::zoneddatetime::interpret_isodatetime_offset;
use crate::builtins::core::{calendar::Calendar, timezone::TimeZone, PlainDate, ZonedDateTime};
use crate::iso::{IsoDate, IsoTime};
use crate::options::{ArithmeticOverflow, Disambiguation, OffsetDisambiguation};
use crate::parsers::parse_date_time;
use crate::provider::TimeZoneProvider;
use crate::{TemporalError, TemporalResult, TemporalUnwrap};

use ixdtf::parsers::records::{TimeZoneRecord, UtcOffsetRecordOrZ};

// ==== RelativeTo Object ====

#[derive(Debug, Clone)]
pub enum RelativeTo {
    PlainDate(PlainDate),
    ZonedDateTime(ZonedDateTime),
}

impl From<PlainDate> for RelativeTo {
    fn from(value: PlainDate) -> Self {
        Self::PlainDate(value)
    }
}

impl From<ZonedDateTime> for RelativeTo {
    fn from(value: ZonedDateTime) -> Self {
        Self::ZonedDateTime(value)
    }
}

impl RelativeTo {
    /// Attempts to parse a `ZonedDateTime` string falling back to a `PlainDate`
    /// if possible.
    ///
    /// If the fallback fails or either the `ZonedDateTime` or `PlainDate`
    /// is invalid, then an error is returned.
    pub fn try_from_str_with_provider(
        source: &str,
        provider: &impl TimeZoneProvider,
    ) -> TemporalResult<Self> {
        let result = parse_date_time(source)?;

        let Some(annotation) = result.tz else {
            let date_record = result.date.temporal_unwrap()?;

            let calendar = result
                .calendar
                .map(Calendar::from_utf8)
                .transpose()?
                .unwrap_or_default();

            return Ok(PlainDate::try_new(
                date_record.year,
                date_record.month,
                date_record.day,
                calendar,
            )?
            .into());
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

        let (offset_nanos, is_exact) = result
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

        let calendar = result
            .calendar
            .map(Calendar::from_utf8)
            .transpose()?
            .unwrap_or_default();

        let time = result
            .time
            .map(|time| {
                IsoTime::from_components(time.hour, time.minute, time.second, time.nanosecond)
            })
            .transpose()?;

        let date = result.date.temporal_unwrap()?;
        let iso = IsoDate::new_with_overflow(
            date.year,
            date.month,
            date.day,
            ArithmeticOverflow::Constrain,
        )?;

        let epoch_ns = interpret_isodatetime_offset(
            iso,
            time,
            is_exact,
            offset_nanos,
            &timezone,
            Disambiguation::Compatible,
            OffsetDisambiguation::Reject,
            true,
            provider,
        )?;

        Ok(ZonedDateTime::try_new(epoch_ns.0, calendar, timezone)?.into())
    }
}
