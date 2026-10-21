use ixdtf::records::{UtcOffsetRecord, UtcOffsetRecordOrZ};

use crate::provider::TimeZoneProvider;
use crate::{builtins::time_zone::UtcOffset, TimeZone};

use super::{parse_ixdtf, ParseVariant, TemporalEncoding};

#[inline]
pub(crate) fn parse_allowed_timezone_formats<T: TemporalEncoding>(
    s: &[T::CodeUnit],
    provider: &(impl TimeZoneProvider + ?Sized),
) -> Option<TimeZone> {
    let (offset, annotation) = if let Ok(r) = parse_ixdtf::<T>(s, ParseVariant::DateTime) {
        (r.offset, r.tz)
    } else if let Ok(r) = parse_ixdtf::<T>(s, ParseVariant::Time) {
        (r.offset, r.tz)
    } else if let Ok(r) = parse_ixdtf::<T>(s, ParseVariant::YearMonth) {
        (r.offset, r.tz)
    } else if let Ok(r) = parse_ixdtf::<T>(s, ParseVariant::MonthDay) {
        (r.offset, r.tz)
    } else {
        return None;
    };

    if let Some(annotation) = annotation {
        return TimeZone::from_time_zone_record::<T>(annotation.tz, provider).ok();
    };

    if let Some(offset) = offset {
        match offset {
            UtcOffsetRecordOrZ::Z => return Some(TimeZone::utc_with_provider(provider)),
            UtcOffsetRecordOrZ::Offset(offset) => {
                let offset = match offset {
                    UtcOffsetRecord::MinutePrecision(offset) => offset,
                    _ => return None,
                };
                return Some(TimeZone::UtcOffset(UtcOffset::from_ixdtf_minute_record(
                    offset,
                )));
            }
        }
    }

    None
}
