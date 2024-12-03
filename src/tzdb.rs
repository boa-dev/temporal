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

use std::path::Path;
#[cfg(not(target_os = "windows"))]
use std::path::PathBuf;

use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::{vec, vec::Vec};
use core::cell::RefCell;

use combine::Parser;

use tzif::{
    self,
    data::{
        posix::{DstTransitionInfo, PosixTzString, TransitionDay, ZoneVariantInfo},
        time::Seconds,
        tzif::{DataBlock, LocalTimeTypeRecord, TzifData, TzifHeader},
    },
};

use crate::{components::tz::TzProvider, iso::IsoDateTime, utils, TemporalError, TemporalResult};

#[cfg(not(target_os = "windows"))]
const ZONEINFO_DIR: &str = "/usr/share/zoneinfo/";

/// `LocalTimeRecord` represents an local time offset record.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LocalTimeRecord {
    /// Whether the local time record is a Daylight Savings Time.
    pub is_dst: bool,
    /// The time zone offset in seconds.
    pub offset: i64,
}

impl LocalTimeRecord {
    fn from_daylight_savings_time(info: &ZoneVariantInfo) -> Self {
        Self {
            is_dst: true,
            offset: -info.offset.0,
        }
    }

    fn from_standard_time(info: &ZoneVariantInfo) -> Self {
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

// TODO: Workshop record name?
/// The `LocalTimeRecord` result represents the result of searching for a
/// a for a time zone transition without the offset seconds applied to the
/// epoch seconds.
///
/// As a result of the search, it is possible for the resulting search to be either
/// Empty (due to an invalid time being provided that would be in the +1 tz shift)
/// or two time zones (when a time exists in the ambiguous range of a -1 shift).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LocalTimeRecordResult {
    Empty,
    Single(LocalTimeRecord),
    // Note(nekevss): it may be best to switch this to initial, need to double check
    // disambiguation ops with inverse DST-STD relationship
    Ambiguous {
        std: LocalTimeRecord,
        dst: LocalTimeRecord,
    },
}

impl From<LocalTimeRecord> for LocalTimeRecordResult {
    fn from(value: LocalTimeRecord) -> Self {
        Self::Single(value)
    }
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

/// `TZif` stands for Time zone information format is laid out by [RFC 8536][rfc8536] and
/// laid out by the [tzdata manual][tzif-manual]
///
/// To be specific, this representation of `TZif` is solely to extend functionality
/// fo the parsed type from the `tzif` [rust crate][tzif-crate], which has further detail on the
/// layout in Rust.
///
/// `TZif` files are compiled via [`zic`][zic-manual], which offers a variety of options for changing the layout
/// and range of a `TZif`.
///
/// [rfc8536]: https://datatracker.ietf.org/doc/html/rfc8536
/// [tzif-manual]: https://man7.org/linux/man-pages/man5/tzfile.5.html
/// [tzif-crate]: https://docs.rs/tzif/latest/tzif/
/// [zic-manual]: https://man7.org/linux/man-pages/man8/zic.8.html
#[derive(Debug, Clone)]
pub struct Tzif {
    pub header1: TzifHeader,
    pub data_block1: DataBlock,
    pub header2: Option<TzifHeader>,
    pub data_block2: Option<DataBlock>,
    pub footer: Option<PosixTzString>,
}

impl From<TzifData> for Tzif {
    fn from(value: TzifData) -> Self {
        let TzifData {
            header1,
            data_block1,
            header2,
            data_block2,
            footer,
        } = value;

        Self {
            header1,
            data_block1,
            header2,
            data_block2,
            footer,
        }
    }
}

impl Tzif {
    pub fn from_bytes(data: &[u8]) -> TemporalResult<Self> {
        let Ok((parse_result, _)) = tzif::parse::tzif::tzif().parse(data) else {
            return Err(TemporalError::general("Illformed Tzif data."));
        };
        Ok(Self::from(parse_result))
    }

    #[cfg(not(target_os = "windows"))]
    pub fn read_tzif(identifier: &str) -> TemporalResult<Self> {
        let mut path = PathBuf::from(ZONEINFO_DIR);
        path.push(identifier);
        Self::from_path(&path)
    }

    pub fn from_path<P: AsRef<Path>>(path: P) -> TemporalResult<Self> {
        tzif::parse_tzif_file(path)
            .map(Into::into)
            .map_err(|e| TemporalError::general(e.to_string()))
    }

    pub fn posix_tz_string(&self) -> Option<&PosixTzString> {
        self.footer.as_ref()
    }

    pub fn get_data_block2(&self) -> TemporalResult<&DataBlock> {
        self.data_block2
            .as_ref()
            .ok_or(TemporalError::general("Only Tzif V2+ is supported."))
    }

    pub fn get(&self, epoch_seconds: &Seconds) -> TemporalResult<LocalTimeRecord> {
        let db = self.get_data_block2()?;
        let result = db.transition_times.binary_search(epoch_seconds);

        match result {
            Ok(idx) => Ok(get_local_record(db, idx - 1).into()),
            Err(idx) if idx == 0 => Ok(get_local_record(db, idx).into()),
            Err(idx) => {
                if db.transition_times.len() <= idx {
                    // The transition time provided is beyond the length of
                    // the available transition time, so the time zone is
                    // resolved with the POSIX tz string.
                    return resolve_posix_tz_string_for_epoch_seconds(
                        self.posix_tz_string().ok_or(TemporalError::general(
                            "No POSIX tz string to resolve with.",
                        ))?,
                        epoch_seconds.0,
                    );
                }
                Ok(get_local_record(db, idx - 1).into())
            }
        }
    }

    // For more information, see /docs/TZDB.md
    /// This function determines the Time Zone output for a local epoch
    /// nanoseconds value without an offset.
    ///
    /// Basically, if someone provides a DateTime 2017-11-05T01:30:00,
    /// we have no way of knowing if this value is in DST or STD.
    /// Furthermore, for the above example, this should return 2 time
    /// zones due to there being two 2017-11-05T01:30:00. On the other
    /// side of the transition, the DateTime 2017-03-12T02:30:00 could
    /// be provided. This time does NOT exist due to the +1 jump from
    /// 02:00 -> 03:00 (but of course it does as a nanosecond value).
    pub fn v2_estimate_tz_pair(&self, seconds: &Seconds) -> TemporalResult<LocalTimeRecordResult> {
        // We need to estimate a tz pair.
        // First search the ambiguous seconds.
        let db = self.get_data_block2()?;
        let b_search_result = db.transition_times.binary_search(seconds);

        let estimated_idx = match b_search_result {
            // TODO: Double check returning early here with tests.
            Ok(idx) => return Ok(get_local_record(db, idx).into()),
            Err(idx) if idx == 0 => {
                return Ok(LocalTimeRecordResult::Single(
                    get_local_record(db, idx).into(),
                ))
            }
            Err(idx) => {
                if db.transition_times.len() <= idx {
                    // The transition time provided is beyond the length of
                    // the available transition time, so the time zone is
                    // resolved with the POSIX tz string.
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
        let record = get_local_record(db, estimated_idx);
        let record_minus_one = get_local_record(db, estimated_idx - 1);

        // Q: Potential shift bugs with odd historical transitions? This
        //
        // Shifts the 2 rule window for positive zones that would have returned
        // a different idx.
        let shift_window = usize::from((record.utoff + record_minus_one.utoff) >= Seconds(0));

        let new_idx = estimated_idx - shift_window;

        let current_transition = db.transition_times[new_idx];
        let current_diff = *seconds - current_transition;

        let initial_record = get_local_record(db, new_idx - 1);
        let next_record = get_local_record(db, new_idx);

        // Adjust for offset inversion from northern/southern hemisphere.
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
    // NOTE: Transition type can be empty. If no transition_type exists,
    // then use 0 as the default index of local_time_type_records.
    db.local_time_type_records[db.transition_types.get(idx).copied().unwrap_or(0)]
}

#[inline]
fn resolve_posix_tz_string_for_epoch_seconds(
    posix_tz_string: &PosixTzString,
    seconds: i64,
) -> TemporalResult<LocalTimeRecord> {
    let Some(dst_variant) = &posix_tz_string.dst_info else {
        // Regardless of the time, there is one variant and we can return it.
        return Ok(LocalTimeRecord::from_standard_time(
            &posix_tz_string.std_info,
        ));
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
        TransitionType::Dst => Ok(LocalTimeRecord::from_daylight_savings_time(
            &dst_variant.variant_info,
        )),
        TransitionType::Std => Ok(LocalTimeRecord::from_standard_time(
            &posix_tz_string.std_info,
        )),
    }
}

/// Resolve the footer of a tzif file.
///
/// Seconds are epoch seconds in local time.
#[inline]
fn resolve_posix_tz_string(
    posix_tz_string: &PosixTzString,
    seconds: i64,
) -> TemporalResult<LocalTimeRecordResult> {
    let std = &posix_tz_string.std_info;
    let Some(dst) = &posix_tz_string.dst_info else {
        // Regardless of the time, there is one variant and we can return it.
        return Ok(LocalTimeRecord::from_standard_time(&posix_tz_string.std_info).into());
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
                    std: LocalTimeRecord::from_standard_time(std),
                    dst: LocalTimeRecord::from_daylight_savings_time(&dst.variant_info),
                })
            }
            _ => {}
        }
    }

    match is_dst {
        TransitionType::Dst => {
            Ok(LocalTimeRecord::from_daylight_savings_time(&dst.variant_info).into())
        }
        TransitionType::Std => {
            Ok(LocalTimeRecord::from_standard_time(&posix_tz_string.std_info).into())
        }
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

/// The month, week of month, and day of week value built into the POSIX tz string.
///
/// For more information, see the [POSIX tz string docs](https://sourceware.org/glibc/manual/2.40/html_node/Proleptic-TZ.html)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Mwd(u16, u16, u16);

impl Mwd {
    fn from_seconds(seconds: f64) -> Self {
        let month = utils::epoch_time_to_month_in_year(seconds * 1_000.0) as u16 + 1;
        let day_of_month = utils::epoch_seconds_to_day_of_month(seconds);
        let week_of_month = day_of_month / 7 + 1;
        let day_of_week = utils::epoch_seconds_to_day_of_week(seconds);
        Self(month, week_of_month, day_of_week)
    }
}

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
            let mwd = Mwd::from_seconds(seconds);
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
            let is_dst = if start > end {
                day_in_year < *end || *start <= day_in_year
            } else {
                *start <= day_in_year && day_in_year < *end
            };
            (is_transition, is_dst)
        }
        (TransitionDay::NoLeap(start), TransitionDay::NoLeap(end)) => {
            let day_in_year = utils::epoch_time_to_day_in_year(seconds * 1_000.0) as u16;
            let is_transition = *start == day_in_year || *end == day_in_year;
            let is_dst = if start > end {
                day_in_year < *end || *start <= day_in_year
            } else {
                *start <= day_in_year && day_in_year < *end
            };
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
    cache: RefCell<BTreeMap<String, Tzif>>,
}

impl FsTzdbProvider {
    pub fn get(&self, identifier: &str) -> TemporalResult<Tzif> {
        if let Some(tzif) = self.cache.borrow().get(identifier) {
            return Ok(tzif.clone());
        }
        #[cfg(any(target_os = "linux", target_os = "macos"))]
        let (identifier, tzif) = { (identifier, Tzif::read_tzif(identifier)?) };

        #[cfg(target_os = "windows")]
        let (identifier, tzif) = {
            let Some((canonical_name, data)) = jiff_tzdb::get(identifier) else {
                return Err(
                    TemporalError::range().with_message("Time zone identifier does not exist.")
                );
            };
            (canonical_name, Tzif::from_bytes(data)?)
        };

        Ok(self
            .cache
            .borrow_mut()
            .entry(identifier.into())
            .or_insert(tzif)
            .clone())
    }
}

impl TzProvider for FsTzdbProvider {
    fn check_identifier(&self, identifier: &str) -> bool {
        self.get(identifier).is_ok()
    }

    fn get_named_tz_epoch_nanoseconds(
        &self,
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
        &self,
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
    fn exactly_transition_time_after_empty_edge_case() {
        let provider = FsTzdbProvider::default();
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
        let provider = FsTzdbProvider::default();
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

        #[cfg(not(target_os = "windows"))]
        let new_york = Tzif::read_tzif("America/New_York");
        #[cfg(target_os = "windows")]
        let new_york = {
            let (_, data) = jiff_tzdb::get("America/New_York").unwrap();
            Tzif::from_bytes(data)
        };

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

        #[cfg(not(target_os = "windows"))]
        let sydney = Tzif::read_tzif("Australia/Sydney");
        #[cfg(target_os = "windows")]
        let sydney = {
            let (_, data) = jiff_tzdb::get("Australia/Sydney").unwrap();
            Tzif::from_bytes(data)
        };

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

        #[cfg(not(target_os = "windows"))]
        let new_york = Tzif::read_tzif("America/New_York");
        #[cfg(target_os = "windows")]
        let new_york = {
            let (_, data) = jiff_tzdb::get("America/New_York").unwrap();
            Tzif::from_bytes(data)
        };

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

        #[cfg(not(target_os = "windows"))]
        let sydney = Tzif::read_tzif("Australia/Sydney");
        #[cfg(target_os = "windows")]
        let sydney = {
            let (_, data) = jiff_tzdb::get("Australia/Sydney").unwrap();
            Tzif::from_bytes(data)
        };

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
        let (_, data) = jiff_tzdb::get("America/New_York").unwrap();
        let new_york = Tzif::from_bytes(data);
        assert!(new_york.is_ok());
        let new_york = new_york.unwrap();

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
        let (_, data) = jiff_tzdb::get("Australia/Sydney").unwrap();
        let sydney = Tzif::from_bytes(data);
        assert!(sydney.is_ok());
        let sydney = sydney.unwrap();

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

        #[cfg(not(target_os = "windows"))]
        let new_york = Tzif::read_tzif("America/New_York");
        #[cfg(target_os = "windows")]
        let new_york = {
            let (_, data) = jiff_tzdb::get("America/New_York").unwrap();
            Tzif::from_bytes(data)
        };

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

        #[cfg(not(target_os = "windows"))]
        let sydney = Tzif::read_tzif("Australia/Sydney");
        #[cfg(target_os = "windows")]
        let sydney = {
            let (_, data) = jiff_tzdb::get("Australia/Sydney").unwrap();
            Tzif::from_bytes(data)
        };

        assert!(sydney.is_ok());
        let sydney = sydney.unwrap();

        let locals = sydney.v2_estimate_tz_pair(&Seconds(seconds)).unwrap();
        assert!(matches!(locals, LocalTimeRecordResult::Single(_)));
    }
}
