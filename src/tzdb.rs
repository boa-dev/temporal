// Relevant operations:
//
//  - Time Zone Identifiers
//  - AvailableNamedTimeZoneIdentifiers
//  - SystemTimeZoneIdentifier
//  - IsTimeZoneOffsetString
//  - GetNamedTimeZoneEpochNanoseconds
//     - fn(id, isoDateTimeRecord) -> [epochNanoseconds]
//  - GetNamedTimeZoneOffsetNanoseconds
//     - fn(id, epochNanoseconds) -> [offset]

// TODO: Potentially implement a IsoDateTimeRecord type to decouple
// public facing APIs from IsoDateTime

// Could return type be something like [Option<i128>; 2]

// NOTE: tzif data is computed in glibc's `__tzfile_compute` in `tzfile.c`.
//
// Handling the logic here may be incredibly important for full tzif support.

// NOTES:
//
// Transitions to DST (in march) + 1. Empty list between 2:00-3:00.
// Transitions to Std (in nov) -1. Two elements 1:00-2:00 is repeated twice.

// Transition Seconds + (offset diff)
// where
// offset diff = is_dst { dst_off - std_off } else { std_off - dst_off }, i.e. to_offset - from_offset

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use combine::Parser;
use tzif::{
    self,
    data::{
        posix::{DstTransitionInfo, PosixTzString, TransitionDay, ZoneVariantInfo},
        time::Seconds,
        tzif::{DataBlock, LocalTimeTypeRecord, TzifData},
    },
};

use crate::{components::tz::TzProvider, iso::IsoDateTime, utils, TemporalError, TemporalResult};

const ZONEINFO_DIR: &str = "/usr/share/zoneinfo/";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LocalTimeRecord {
    /// Whether the local time record is a Daylight Savings Time.
    pub is_dst: bool,
    /// The time zone offset in seconds.
    pub offset: i64,
}

impl LocalTimeRecord {
    fn dst_zi(info: &ZoneVariantInfo) -> Self {
        Self {
            is_dst: true,
            offset: -info.offset.0,
        }
    }

    fn std_zi(info: &ZoneVariantInfo) -> Self {
        Self {
            is_dst: false,
            offset: -info.offset.0,
        }
    }
}

impl From<LocalTimeTypeRecord> for LocalTimeRecord {
    fn from(value: LocalTimeTypeRecord) -> Self {
        Self {
            is_dst: value.is_dst,
            offset: value.utoff.0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum TransitionTimeSearch {
    Index(usize),
    PosixTz,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LocalTimeRecordResult {
    Empty,
    Single(LocalTimeRecord),
    Ambiguous {
        std: LocalTimeRecord,
        dst: LocalTimeRecord,
    },
}

impl From<LocalTimeTypeRecord> for LocalTimeRecordResult {
    fn from(value: LocalTimeTypeRecord) -> Self {
        Self::Single(value.into())
    }
}

impl From<(LocalTimeTypeRecord, LocalTimeTypeRecord)> for LocalTimeRecordResult {
    fn from(value: (LocalTimeTypeRecord, LocalTimeTypeRecord)) -> Self {
        Self::Ambiguous {
            std: value.0.into(),
            dst: value.1.into(),
        }
    }
}

#[derive(Debug)]
pub struct Tzif(TzifData);

impl Tzif {
    pub fn from_fallback(identifier: &str) -> TemporalResult<Self> {
        let Some((_canonical_name, data)) = jiff_tzdb::get(identifier) else {
            return Err(TemporalError::general("Not a valid IANA identifier."));
        };
        let Ok((parse_result, _)) = tzif::parse::tzif::tzif().parse(data) else {
            return Err(TemporalError::general("Illformed Tzif data."));
        };
        Ok(Self(parse_result))
    }

    fn read_tzif(identifier: &str) -> TemporalResult<Self> {
        let mut path = PathBuf::from(ZONEINFO_DIR);
        path.push(identifier);
        Self::from_path(&path)
    }

    pub fn from_path<P: AsRef<Path>>(path: P) -> TemporalResult<Self> {
        tzif::parse_tzif_file(path)
            .map(Self)
            .map_err(|e| TemporalError::general(e.to_string()))
    }

    pub fn posix_tz_string(&self) -> Option<&PosixTzString> {
        self.0.footer.as_ref()
    }

    // There are ultimately
    pub fn get(&self, epoch_seconds: &Seconds) -> TemporalResult<LocalTimeRecord> {
        let Some(result) = self.binary_search(epoch_seconds) else {
            return Err(TemporalError::general("Only Tzif v2+ is supported."));
        };

        let db = self
            .0
            .data_block2
            .as_ref()
            .expect("binary search throws error if datablock doesn't exist.");

        match result {
            Ok(idx) => Ok(get_local_record(db, idx - 1).into()),
            Err(idx) if idx == 0 => Ok(get_local_record(db, idx).into()),
            Err(idx) => {
                if db.transition_times.len() <= idx {
                    return resolve_posix_tz_string_for_epoch_seconds(
                        self.posix_tz_string().ok_or(TemporalError::general(
                            "No POSIX tz string to resolve with.",
                        ))?,
                        epoch_seconds.0,
                    )
                    .into();
                }
                Ok(get_local_record(db, idx - 1).into())
            }
        }
    }

    // There are various other ways to search rather than binary_search. See glibc
    pub fn binary_search(&self, epoch_seconds: &Seconds) -> Option<Result<usize, usize>> {
        self.0
            .data_block2
            .as_ref()
            .map(|b| b.transition_times.binary_search(epoch_seconds))
    }

    pub fn v2_estimate_tz_pair(&self, seconds: &Seconds) -> TemporalResult<LocalTimeRecordResult> {
        // We need to estimate a tz pair.
        // First search the ambiguous seconds.
        // TODO: it would be nice to resolve the Posix str into a local time type record.
        let Some(b_search_result) = self.binary_search(seconds) else {
            return Err(TemporalError::general("Only Tzif v2+ is supported."));
        };

        let data_block = self
            .0
            .data_block2
            .as_ref()
            .expect("binary_search validates that data_block2 exists.");

        let estimated_idx = match b_search_result {
            Ok(idx) => idx,
            Err(idx) if idx == 0 => {
                return Ok(LocalTimeRecordResult::Single(
                    get_local_record(data_block, idx).into(),
                ))
            }
            Err(idx) => {
                if data_block.transition_times.len() <= idx {
                    return resolve_posix_tz_string(
                        self.posix_tz_string()
                            .ok_or(TemporalError::general("Could not resolve time zone."))?,
                        seconds.0,
                    );
                }
                idx
            }
        };

        // The estimated index will be off based on the amount missing
        // from the lack of offset.
        //
        // This means that we may need (idx, idx - 1) or (idx - 1, idx - 2)
        let record = get_local_record(data_block, estimated_idx);
        let record_minus_one = get_local_record(data_block, estimated_idx - 1);

        // Potential shift bugs with odd historical transitions?
        let shift_window = usize::from((record.utoff + record_minus_one.utoff).0.signum() >= 0);

        let new_idx = estimated_idx - shift_window;

        let current_transition = data_block.transition_times[new_idx];
        let current_diff = *seconds - current_transition;

        let initial_record = get_local_record(data_block, new_idx - 1);
        let next_record = get_local_record(data_block, new_idx);

        let offset_range = offset_range(initial_record.utoff.0, next_record.utoff.0);
        match offset_range.contains(&current_diff.0) {
            true if next_record.is_dst => Ok(LocalTimeRecordResult::Empty),
            true => Ok((next_record, initial_record).into()),
            false => Ok(initial_record.into()),
        }
    }
}

#[inline]
fn get_local_record(db: &DataBlock, idx: usize) -> LocalTimeTypeRecord {
    db.local_time_type_records[db.transition_types[idx]]
}

fn resolve_posix_tz_string_for_epoch_seconds(
    posix_tz_string: &PosixTzString,
    seconds: i64,
) -> TemporalResult<LocalTimeRecord> {
    let Some(dst_variant) = &posix_tz_string.dst_info else {
        // Regardless of the time, there is one variant and we can return it.
        return Ok(LocalTimeRecord::std_zi(&posix_tz_string.std_info));
    };

    let start = &dst_variant.start_date;
    let end = &dst_variant.end_date;

    // TODO: Resolve safety issue around utils.
    //   Using f64 is a hold over from early implementation days and should
    //   be moved away from.
    let seconds = seconds as f64;

    let (is_transition_day, transition) =
        cmp_seconds_to_transitions(&start.day, &end.day, seconds)?;

    match compute_tz_for_epoch_seconds(is_transition_day, transition, seconds, dst_variant) {
        TransitionType::Dst => Ok(LocalTimeRecord::dst_zi(&dst_variant.variant_info)),
        TransitionType::Std => Ok(LocalTimeRecord::std_zi(&posix_tz_string.std_info)),
    }
}

// TODO: Validate validity when dealing with epoch nanoseconds vs. ambiguous nanoseconds.
#[inline]
/// Resolve the footer of a tzif file.
///
/// Seconds are epoch seconds in local time.
fn resolve_posix_tz_string(
    posix_tz_string: &PosixTzString,
    seconds: i64,
) -> TemporalResult<LocalTimeRecordResult> {
    let std = &posix_tz_string.std_info;
    let Some(dst) = &posix_tz_string.dst_info else {
        // Regardless of the time, there is one variant and we can return it.
        return Ok(LocalTimeRecordResult::Single(LocalTimeRecord::std_zi(
            &posix_tz_string.std_info,
        )));
    };

    // TODO: Resolve safety issue around utils.
    //   Using f64 is a hold over from early implementation days and should
    //   be moved away from.
    let seconds = seconds as f64;

    // NOTE:
    // STD -> DST == start
    // DST -> STD == end
    let (is_transition_day, is_dst) =
        cmp_seconds_to_transitions(&dst.start_date.day, &dst.end_date.day, seconds)?;
    if is_transition_day {
        let time = utils::epoch_ms_to_ms_in_day(seconds * 1_000.0) as i64 / 1_000;
        let transition_time = if is_dst == TransitionType::Dst {
            dst.start_date.time.0
        } else {
            dst.end_date.time.0
        };
        let transition_diff = if is_dst == TransitionType::Dst {
            std.offset.0 - dst.variant_info.offset.0
        } else {
            dst.variant_info.offset.0 - std.offset.0
        };
        let offset = offset_range(transition_time + transition_diff, transition_time);
        match offset.contains(&time) {
            true if is_dst == TransitionType::Dst => return Ok(LocalTimeRecordResult::Empty),
            true => {
                return Ok(LocalTimeRecordResult::Ambiguous {
                    std: LocalTimeRecord::std_zi(std),
                    dst: LocalTimeRecord::dst_zi(&dst.variant_info),
                })
            }
            _ => {}
        }
    }

    match is_dst {
        TransitionType::Dst => Ok(LocalTimeRecordResult::Single(LocalTimeRecord::dst_zi(
            &dst.variant_info,
        ))),
        TransitionType::Std => Ok(LocalTimeRecordResult::Single(LocalTimeRecord::std_zi(
            &posix_tz_string.std_info,
        ))),
    }
}

fn compute_tz_for_epoch_seconds(
    is_transition_day: bool,
    transition: TransitionType,
    seconds: f64,
    dst_variant: &DstTransitionInfo,
) -> TransitionType {
    if is_transition_day && transition == TransitionType::Dst {
        let time = utils::epoch_ms_to_ms_in_day(seconds * 1_000.0) / 1_000;
        let transition_time = dst_variant.start_date.time.0 - dst_variant.variant_info.offset.0;
        if i64::from(time) < transition_time {
            return TransitionType::Std;
        }
    } else if is_transition_day {
        let time = utils::epoch_ms_to_ms_in_day(seconds * 1_000.0) / 1_000;
        let transition_time = dst_variant.end_date.time.0 - dst_variant.variant_info.offset.0;
        if i64::from(time) < transition_time {
            return TransitionType::Dst;
        }
    }

    transition
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Mwd(u16, u16, u16);

fn cmp_seconds_to_transitions(
    start: &TransitionDay,
    end: &TransitionDay,
    seconds: f64,
) -> TemporalResult<(bool, TransitionType)> {
    let cmp_result = match (start, end) {
        (
            TransitionDay::Mwd(start_month, start_week, start_day),
            TransitionDay::Mwd(end_month, end_week, end_day),
        ) => {
            let month = utils::epoch_time_to_month_in_year(seconds * 1_000.0) as u16 + 1;
            let day_of_month = utils::epoch_seconds_to_day_of_month(seconds);
            let week_of_month = day_of_month / 7 + 1;
            let day_of_week = utils::epoch_seconds_to_day_of_week(seconds);
            let mwd = Mwd(month, week_of_month, day_of_week);
            let start = Mwd(*start_month, *start_week, *start_day);
            let end = Mwd(*end_month, *end_week, *end_day);

            let is_transition = start == mwd || end == mwd;
            let is_dst = if start > end {
                mwd < end || start <= mwd
            } else {
                start <= mwd && mwd < end
            };

            (is_transition, is_dst)
        }
        (TransitionDay::WithLeap(start), TransitionDay::WithLeap(end)) => {
            let day_in_year = utils::epoch_time_to_day_in_year(seconds * 1_000.0) as u16;
            let is_transition = *start == day_in_year || *end == day_in_year;
            let mut is_dst = *start <= day_in_year && day_in_year < *end;
            if start > end {
                is_dst = !is_dst;
            }
            (is_transition, is_dst)
        }
        (TransitionDay::NoLeap(start), TransitionDay::NoLeap(end)) => {
            let day_in_year = utils::epoch_time_to_day_in_year(seconds * 1_000.0) as u16;
            let is_transition = *start == day_in_year || *end == day_in_year;
            let mut is_dst = *start <= day_in_year && day_in_year < *end;
            if start > end {
                is_dst = !is_dst;
            }
            (is_transition, is_dst)
        }
        // NOTE: The assumption here is that mismatched day types on
        // a POSIX string is an illformed string.
        _ => return Err(TemporalError::assert()),
    };

    match cmp_result {
        (true, dst) if dst => Ok((true, TransitionType::Dst)),
        (true, _) => Ok((true, TransitionType::Std)),
        (false, dst) if dst => Ok((false, TransitionType::Dst)),
        (false, _) => Ok((false, TransitionType::Std)),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TransitionType {
    Dst,
    Std,
}

fn offset_range(offset_one: i64, offset_two: i64) -> core::ops::Range<i64> {
    if offset_one < offset_two {
        return offset_one..offset_two;
    }
    offset_two..offset_one
}

#[derive(Debug, Default)]
pub struct FsTzdbProvider {
    cache: HashMap<String, Tzif>,
}

impl FsTzdbProvider {
    pub fn get(&mut self, identifier: &str) -> TemporalResult<&Tzif> {
        if !self.cache.contains_key(identifier) {
            let tzif = Tzif::read_tzif(identifier)?;
            self.cache.insert(identifier.into(), tzif);
            self.cache.get(identifier).ok_or(TemporalError::assert())
        } else {
            self.cache.get(identifier).ok_or(TemporalError::assert())
        }
    }
}

impl TzProvider for FsTzdbProvider {
    fn check_identifier(&mut self, identifier: &str) -> bool {
        self.get(identifier).is_ok()
    }

    fn get_named_tz_epoch_nanoseconds(
        &mut self,
        identifier: &str,
        iso_datetime: IsoDateTime,
    ) -> TemporalResult<Vec<i128>> {
        let seconds = (iso_datetime
            .as_nanoseconds()
            .expect("IsoDateTime to be valid")
            / 1_000_000_000) as i64;
        let tzif = self.get(identifier)?;
        let local_time_record_result = tzif.v2_estimate_tz_pair(&Seconds(seconds))?;
        let result = match local_time_record_result {
            LocalTimeRecordResult::Empty => Vec::default(),
            LocalTimeRecordResult::Single(r) => vec![r.offset as i128 * 1_000_000_000],
            LocalTimeRecordResult::Ambiguous { std, dst } => vec![
                std.offset as i128 * 1_000_000_000,
                dst.offset as i128 * 1_000_000_000,
            ],
        };
        Ok(result)
    }

    fn get_named_tz_offset_nanoseconds(
        &mut self,
        identifier: &str,
        epoch_nanoseconds: i128,
    ) -> TemporalResult<i128> {
        let tzif = self.get(identifier)?;
        let seconds = (epoch_nanoseconds / 1_000_000_000) as i64;
        let local_time_record_result = tzif.get(&Seconds(seconds))?;
        Ok(local_time_record_result.offset as i128 * 1_000_000_000)
    }
}
//

#[cfg(test)]
mod tests {
    use tzif::data::time::Seconds;

    use crate::{
        iso::IsoDateTime,
        tzdb::{LocalTimeRecord, LocalTimeRecordResult, TzProvider},
    };

    use super::{FsTzdbProvider, Tzif};

    #[test]
    fn one_second_after_empty_edge_case() {
        let mut provider = FsTzdbProvider::default();
        let date = crate::iso::IsoDate {
            year: 2017,
            month: 3,
            day: 12,
        };
        let time = crate::iso::IsoTime {
            hour: 3,
            minute: 0,
            second: 0,
            millisecond: 0,
            microsecond: 0,
            nanosecond: 0,
        };
        let today = IsoDateTime::new(date, time).unwrap();

        let local = provider
            .get_named_tz_epoch_nanoseconds("America/New_York", today)
            .unwrap();
        assert_eq!(local.len(), 1);
    }

    #[test]
    fn one_second_before_empty_edge_case() {
        let mut provider = FsTzdbProvider::default();
        let date = crate::iso::IsoDate {
            year: 2017,
            month: 3,
            day: 12,
        };
        let time = crate::iso::IsoTime {
            hour: 2,
            minute: 59,
            second: 59,
            millisecond: 0,
            microsecond: 0,
            nanosecond: 0,
        };
        let today = IsoDateTime::new(date, time).unwrap();

        let local = provider
            .get_named_tz_epoch_nanoseconds("America/New_York", today)
            .unwrap();
        assert!(local.is_empty());
    }

    #[test]
    fn new_york_empty_test_case() {
        let date = crate::iso::IsoDate {
            year: 2017,
            month: 3,
            day: 12,
        };
        let time = crate::iso::IsoTime {
            hour: 2,
            minute: 30,
            second: 0,
            millisecond: 0,
            microsecond: 0,
            nanosecond: 0,
        };
        let edge_case = IsoDateTime::new(date, time).unwrap();
        let edge_case_seconds = edge_case
            .as_nanoseconds()
            .map_or(0, |nanos| (nanos / 1_000_000_000) as i64);

        let new_york = Tzif::read_tzif("America/New_York");
        assert!(new_york.is_ok());
        let new_york = new_york.unwrap();

        let locals = new_york
            .v2_estimate_tz_pair(&Seconds(edge_case_seconds))
            .unwrap();
        assert_eq!(locals, LocalTimeRecordResult::Empty);
    }

    #[test]
    fn sydney_empty_test_case() {
        // Australia Daylight savings day
        let date = crate::iso::IsoDate {
            year: 2017,
            month: 10,
            day: 1,
        };
        let time = crate::iso::IsoTime {
            hour: 2,
            minute: 30,
            second: 0,
            millisecond: 0,
            microsecond: 0,
            nanosecond: 0,
        };
        let today = IsoDateTime::new(date, time).unwrap();
        let seconds = today
            .as_nanoseconds()
            .map_or(0, |nanos| (nanos / 1_000_000_000) as i64);

        let sydney = Tzif::read_tzif("Australia/Sydney");
        assert!(sydney.is_ok());
        let sydney = sydney.unwrap();

        let locals = sydney.v2_estimate_tz_pair(&Seconds(seconds)).unwrap();
        assert_eq!(locals, LocalTimeRecordResult::Empty);
    }

    #[test]
    fn new_york_duplicate_case() {
        let date = crate::iso::IsoDate {
            year: 2017,
            month: 11,
            day: 5,
        };
        let time = crate::iso::IsoTime {
            hour: 1,
            minute: 30,
            second: 0,
            millisecond: 0,
            microsecond: 0,
            nanosecond: 0,
        };
        let edge_case = IsoDateTime::new(date, time).unwrap();
        let edge_case_seconds = edge_case
            .as_nanoseconds()
            .map_or(0, |nanos| (nanos / 1_000_000_000) as i64);

        let new_york = Tzif::read_tzif("America/New_York");
        assert!(new_york.is_ok());
        let new_york = new_york.unwrap();

        let locals = new_york
            .v2_estimate_tz_pair(&Seconds(edge_case_seconds))
            .unwrap();

        assert_eq!(
            locals,
            LocalTimeRecordResult::Ambiguous {
                std: LocalTimeRecord {
                    is_dst: false,
                    offset: -18000
                },
                dst: LocalTimeRecord {
                    is_dst: true,
                    offset: -14400,
                },
            }
        );
    }

    #[test]
    fn sydney_duplicate_case() {
        // Australia Daylight savings day
        let date = crate::iso::IsoDate {
            year: 2017,
            month: 4,
            day: 2,
        };
        let time = crate::iso::IsoTime {
            hour: 2,
            minute: 30,
            second: 0,
            millisecond: 0,
            microsecond: 0,
            nanosecond: 0,
        };
        let today = IsoDateTime::new(date, time).unwrap();
        let seconds = today
            .as_nanoseconds()
            .map_or(0, |nanos| (nanos / 1_000_000_000) as i64);

        let sydney = Tzif::read_tzif("Australia/Sydney");
        assert!(sydney.is_ok());
        let sydney = sydney.unwrap();

        let locals = sydney.v2_estimate_tz_pair(&Seconds(seconds)).unwrap();

        assert_eq!(
            locals,
            LocalTimeRecordResult::Ambiguous {
                std: LocalTimeRecord {
                    is_dst: false,
                    offset: 36000
                },
                dst: LocalTimeRecord {
                    is_dst: true,
                    offset: 39600,
                },
            }
        );
    }

    #[test]
    fn new_york_duplicate_with_slim_format() {
        let date = crate::iso::IsoDate {
            year: 2017,
            month: 11,
            day: 5,
        };
        let time = crate::iso::IsoTime {
            hour: 1,
            minute: 30,
            second: 0,
            millisecond: 0,
            microsecond: 0,
            nanosecond: 0,
        };
        let edge_case = IsoDateTime::new(date, time).unwrap();
        let edge_case_seconds = edge_case
            .as_nanoseconds()
            .map_or(0, |nanos| (nanos / 1_000_000_000) as i64);

        let new_york = Tzif::from_fallback("America/New_York");
        assert!(new_york.is_ok());
        let new_york = new_york.unwrap();

        let locals = new_york
            .v2_estimate_tz_pair(&Seconds(edge_case_seconds))
            .unwrap();

        assert_eq!(
            locals,
            LocalTimeRecordResult::Ambiguous {
                std: LocalTimeRecord {
                    is_dst: false,
                    offset: -18000
                },
                dst: LocalTimeRecord {
                    is_dst: true,
                    offset: -14400,
                },
            }
        );
    }

    #[test]
    fn sydney_duplicate_case_with_slim_format() {
        // Australia Daylight savings day
        let date = crate::iso::IsoDate {
            year: 2017,
            month: 4,
            day: 2,
        };
        let time = crate::iso::IsoTime {
            hour: 2,
            minute: 30,
            second: 0,
            millisecond: 0,
            microsecond: 0,
            nanosecond: 0,
        };
        let today = IsoDateTime::new(date, time).unwrap();
        let seconds = today
            .as_nanoseconds()
            .map_or(0, |nanos| (nanos / 1_000_000_000) as i64);

        let sydney = Tzif::from_fallback("Australia/Sydney");
        assert!(sydney.is_ok());
        let sydney = sydney.unwrap();

        let locals = sydney.v2_estimate_tz_pair(&Seconds(seconds)).unwrap();

        assert_eq!(
            locals,
            LocalTimeRecordResult::Ambiguous {
                std: LocalTimeRecord {
                    is_dst: false,
                    offset: 36000
                },
                dst: LocalTimeRecord {
                    is_dst: true,
                    offset: 39600,
                },
            }
        );
    }

    // TODO: Determine the validity of this test. Primarily, this test
    // goes beyond the regularly historic limit of transition_times, so
    // even when on a DST boundary the first time zone is returned. The
    // question is whether this behavior is consistent with what would
    // be expected.
    #[test]
    fn before_epoch_northern_hemisphere() {
        let date = crate::iso::IsoDate {
            year: 1880,
            month: 11,
            day: 5,
        };
        let time = crate::iso::IsoTime {
            hour: 1,
            minute: 30,
            second: 0,
            millisecond: 0,
            microsecond: 0,
            nanosecond: 0,
        };
        let edge_case = IsoDateTime::new(date, time).unwrap();
        let edge_case_seconds = edge_case
            .as_nanoseconds()
            .map_or(0, |nanos| (nanos / 1_000_000_000) as i64);

        let new_york = Tzif::read_tzif("America/New_York");
        assert!(new_york.is_ok());
        let new_york = new_york.unwrap();

        let locals = new_york
            .v2_estimate_tz_pair(&Seconds(edge_case_seconds))
            .unwrap();

        assert!(matches!(locals, LocalTimeRecordResult::Single(_)));
    }

    // TODO: Determine the validity of this test. Primarily, this test
    // goes beyond the regularly historic limit of transition_times, so
    // even when on a DST boundary the first time zone is returned. The
    // question is whether this behavior is consistent with what would
    // be expected.
    #[test]
    fn before_epoch_southern_hemisphere() {
        // Australia Daylight savings day
        let date = crate::iso::IsoDate {
            year: 1880,
            month: 4,
            day: 2,
        };
        let time = crate::iso::IsoTime {
            hour: 2,
            minute: 30,
            second: 0,
            millisecond: 0,
            microsecond: 0,
            nanosecond: 0,
        };
        let today = IsoDateTime::new(date, time).unwrap();
        let seconds = today
            .as_nanoseconds()
            .map_or(0, |nanos| (nanos / 1_000_000_000) as i64);

        let sydney = Tzif::read_tzif("Australia/Sydney");
        assert!(sydney.is_ok());
        let sydney = sydney.unwrap();

        let locals = sydney.v2_estimate_tz_pair(&Seconds(seconds)).unwrap();
        assert!(matches!(locals, LocalTimeRecordResult::Single(_)));
    }
}
