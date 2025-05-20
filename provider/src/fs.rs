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
use std::path::PathBuf;

use alloc::collections::BTreeMap;
use alloc::string::String;
use core::cell::RefCell;

use tzif::{
    self,
    data::{
        posix::{DstTransitionInfo, PosixTzString, TransitionDay, ZoneVariantInfo},
        time::Seconds,
        tzif::{DataBlock, LocalTimeTypeRecord, TzifData, TzifHeader},
    },
};

use crate::{
    identifiers::SINGLETON_IANA_NORMALIZER, utils, PotentialLocalTime, TimeZoneOffset,
    TimeZoneProvider, TransitionDirection,
};

const UNIX_ZONEINFO_DIR: &str = "/usr/share/zoneinfo/";

#[derive(Debug)]
pub enum FsProviderError {
    Tzif(TzifError),
    NotYetImplemented,
}

impl From<TzifError> for FsProviderError {
    fn from(value: TzifError) -> Self {
        Self::Tzif(value)
    }
}

#[derive(Debug)]
pub enum TzifError {
    Tzif(tzif::error::Error),
    MissingPosixTimeZoneString,
    MissingDataBlock,
    UnalignedDayType,
}

impl From<tzif::error::Error> for TzifError {
    fn from(value: tzif::error::Error) -> Self {
        Self::Tzif(value)
    }
}

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
    pub fn from_path(path: &Path) -> Result<Self, TzifError> {
        Ok(tzif::parse_tzif_file(path).map(Into::into)?)
    }

    pub fn posix_tz_string(&self) -> Option<&PosixTzString> {
        self.footer.as_ref()
    }

    pub fn get_data_block2(&self) -> Result<&DataBlock, TzifError> {
        self.data_block2.as_ref().ok_or(TzifError::MissingDataBlock)
    }

    pub fn get(&self, epoch_seconds: &Seconds) -> Result<TimeZoneOffset, TzifError> {
        let db = self.get_data_block2()?;

        let result = db.transition_times.binary_search(epoch_seconds);

        match result {
            Ok(idx) => Ok(get_timezone_offset(db, idx - 1)),
            // <https://datatracker.ietf.org/doc/html/rfc8536#section-3.2>
            // If there are no transitions, local time for all timestamps is specified by the TZ
            // string in the footer if present and nonempty; otherwise, it is
            // specified by time type 0.
            Err(_) if db.transition_times.is_empty() => {
                if let Some(posix_tz_string) = self.posix_tz_string() {
                    resolve_posix_tz_string_for_epoch_seconds(posix_tz_string, epoch_seconds.0)
                } else {
                    Ok(TimeZoneOffset {
                        offset: db.local_time_type_records[0].utoff.0,
                        transition_epoch: None,
                    })
                }
            }
            Err(idx) if idx == 0 => Ok(get_timezone_offset(db, idx)),
            Err(idx) => {
                if db.transition_times.len() <= idx {
                    // The transition time provided is beyond the length of
                    // the available transition time, so the time zone is
                    // resolved with the POSIX tz string.
                    let mut offset = resolve_posix_tz_string_for_epoch_seconds(
                        self.posix_tz_string()
                            .ok_or(TzifError::MissingPosixTimeZoneString)?,
                        epoch_seconds.0,
                    )?;
                    offset
                        .transition_epoch
                        .get_or_insert_with(|| db.transition_times[idx - 1].0);
                    return Ok(offset);
                }
                Ok(get_timezone_offset(db, idx - 1))
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
    pub fn v2_estimate_tz_pair(&self, seconds: &Seconds) -> Result<PotentialLocalTime, TzifError> {
        // We need to estimate a tz pair.
        // First search the ambiguous seconds.
        let db = self.get_data_block2()?;
        let b_search_result = db.transition_times.binary_search(seconds);

        let estimated_idx = match b_search_result {
            // TODO: Double check returning early here with tests.
            Ok(idx) => return Ok(PotentialLocalTime::Single(get_local_record(db, idx).into())),
            Err(idx) if idx == 0 => {
                return Ok(PotentialLocalTime::Single(get_local_record(db, idx).into()))
            }
            Err(idx) => {
                if db.transition_times.len() <= idx {
                    // The transition time provided is beyond the length of
                    // the available transition time, so the time zone is
                    // resolved with the POSIX tz string.
                    return resolve_posix_tz_string(
                        self.posix_tz_string()
                            .ok_or(TzifError::MissingPosixTimeZoneString)?,
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
            true if next_record.is_dst => Ok(PotentialLocalTime::Empty),
            true => Ok(PotentialLocalTime::Ambiguous {
                dst: next_record.into(),
                std: initial_record.into(),
            }),
            false if current_diff <= initial_record.utoff => {
                Ok(PotentialLocalTime::Single(initial_record.into()))
            }
            false => Ok(PotentialLocalTime::Single(next_record.into())),
        }
    }
}

#[inline]
fn get_timezone_offset(db: &DataBlock, idx: usize) -> TimeZoneOffset {
    // NOTE: Transition type can be empty. If no transition_type exists,
    // then use 0 as the default index of local_time_type_records.
    let offset = db.local_time_type_records[db.transition_types.get(idx).copied().unwrap_or(0)];
    TimeZoneOffset {
        transition_epoch: db.transition_times.get(idx).map(|s| s.0),
        offset: offset.utoff.0,
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
) -> Result<TimeZoneOffset, TzifError> {
    let Some(dst_variant) = &posix_tz_string.dst_info else {
        // Regardless of the time, there is one variant and we can return it.
        return Ok(TimeZoneOffset {
            transition_epoch: None,
            offset: LocalTimeRecord::from_standard_time(&posix_tz_string.std_info).offset,
        });
    };

    let start = &dst_variant.start_date;
    let end = &dst_variant.end_date;

    // TODO: Resolve safety issue around utils.
    //   Using f64 is a hold over from early implementation days and should
    //   be moved away from.

    let (is_transition_day, transition) =
        cmp_seconds_to_transitions(&start.day, &end.day, seconds)?;

    let transition =
        compute_tz_for_epoch_seconds(is_transition_day, transition, seconds, dst_variant);
    let std_offset = LocalTimeRecord::from_standard_time(&posix_tz_string.std_info).offset;
    let dst_offset = LocalTimeRecord::from_daylight_savings_time(&dst_variant.variant_info).offset;
    let (old_offset, new_offset) = match transition {
        TransitionType::Dst => (std_offset, dst_offset),
        TransitionType::Std => (dst_offset, std_offset),
    };
    let transition = match transition {
        TransitionType::Dst => start,
        TransitionType::Std => end,
    };
    let year = utils::epoch_time_to_epoch_year(seconds * 1000);
    let year_epoch = utils::epoch_days_for_year(year) * 86400;
    let leap_day = utils::mathematical_in_leap_year(seconds * 1000) as u16;

    let days = match transition.day {
        TransitionDay::NoLeap(day) if day > 59 => day - 1 + leap_day,
        TransitionDay::NoLeap(day) => day - 1,
        TransitionDay::WithLeap(day) => day,
        TransitionDay::Mwd(month, week, day) => {
            let days_to_month = utils::month_to_day((month - 1) as u8, leap_day);
            let days_in_month = u16::from(utils::iso_days_in_month(year, month as u8) - 1);

            // Month starts in the day...
            let day_offset =
                (u16::from(utils::epoch_seconds_to_day_of_week(i64::from(year_epoch)))
                    + days_to_month)
                    .rem_euclid(7);

            // EXAMPLE:
            //
            // 0   1   2   3   4   5   6
            // sun mon tue wed thu fri sat
            // -   -   -   0   1   2   3
            // 4   5   6   7   8   9   10
            // 11  12  13  14  15  16  17
            // 18  19  20  21  22  23  24
            // 25  26  27  28  29  30  -
            //
            // The day_offset = 3, since the month starts on a wednesday.
            //
            // We're looking for the second friday of the month. Thus, since the month started before
            // a friday, we need to start counting from week 0:
            //
            // day_of_month = (week - u16::from(day_offset <= day)) * 7 + day - day_offset = (2 - 1) * 7 + 5 - 3 = 9
            //
            // This works if the month started on a day before the day we want (day_offset <= day). However, if that's not the
            // case, we need to start counting on week 1. For example, calculate the day of the month for the third monday
            // of the month:
            //
            // day_of_month = (week - u16::from(day_offset <= day)) * 7 + day - day_offset = (3 - 0) * 7 + 1 - 3 = 19
            let mut day_of_month = (week - u16::from(day_offset <= day)) * 7 + day - day_offset;

            // If we're on week 5, we need to clamp to the last valid day.
            if day_of_month > days_in_month - 1 {
                day_of_month -= 7
            }

            days_to_month + day_of_month
        }
    };

    // Transition time is on local time, so we need to add the UTC offset to get the correct UTC timestamp
    // for the transition.
    let transition_epoch =
        i64::from(year_epoch) + i64::from(days) * 86400 + transition.time.0 - old_offset;
    Ok(TimeZoneOffset {
        offset: new_offset,
        transition_epoch: Some(transition_epoch),
    })
}

/// Resolve the footer of a tzif file.
///
/// Seconds are epoch seconds in local time.
#[inline]
fn resolve_posix_tz_string(
    posix_tz_string: &PosixTzString,
    seconds: i64,
) -> Result<PotentialLocalTime, TzifError> {
    let std = &posix_tz_string.std_info;
    let Some(dst) = &posix_tz_string.dst_info else {
        // Regardless of the time, there is one variant and we can return it.
        return Ok(PotentialLocalTime::Single(
            LocalTimeRecord::from_standard_time(&posix_tz_string.std_info),
        ));
    };

    // TODO: Resolve safety issue around utils.
    //   Using f64 is a hold over from early implementation days and should
    //   be moved away from.

    // NOTE:
    // STD -> DST == start
    // DST -> STD == end
    let (is_transition_day, is_dst) =
        cmp_seconds_to_transitions(&dst.start_date.day, &dst.end_date.day, seconds)?;
    if is_transition_day {
        let time = utils::epoch_ms_to_ms_in_day(seconds * 1_000) as i64 / 1_000;
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
            true if is_dst == TransitionType::Dst => return Ok(PotentialLocalTime::Empty),
            true => {
                return Ok(PotentialLocalTime::Ambiguous {
                    std: LocalTimeRecord::from_standard_time(std),
                    dst: LocalTimeRecord::from_daylight_savings_time(&dst.variant_info),
                })
            }
            _ => {}
        }
    }

    match is_dst {
        TransitionType::Dst => Ok(PotentialLocalTime::Single(
            LocalTimeRecord::from_daylight_savings_time(&dst.variant_info),
        )),
        TransitionType::Std => Ok(PotentialLocalTime::Single(
            LocalTimeRecord::from_standard_time(&posix_tz_string.std_info),
        )),
    }
}

fn compute_tz_for_epoch_seconds(
    is_transition_day: bool,
    transition: TransitionType,
    seconds: i64,
    dst_variant: &DstTransitionInfo,
) -> TransitionType {
    if is_transition_day && transition == TransitionType::Dst {
        let time = utils::epoch_ms_to_ms_in_day(seconds * 1_000) / 1_000;
        let transition_time = dst_variant.start_date.time.0 - dst_variant.variant_info.offset.0;
        if i64::from(time) < transition_time {
            return TransitionType::Std;
        }
    } else if is_transition_day {
        let time = utils::epoch_ms_to_ms_in_day(seconds * 1_000) / 1_000;
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
    fn from_seconds(seconds: i64) -> Self {
        let month = utils::epoch_ms_to_month_in_year(seconds * 1_000) as u16;
        let day_of_month = utils::epoch_seconds_to_day_of_month(seconds);
        let week_of_month = day_of_month / 7 + 1;
        let day_of_week = utils::epoch_seconds_to_day_of_week(seconds);
        Self(month, week_of_month, u16::from(day_of_week))
    }
}

fn cmp_seconds_to_transitions(
    start: &TransitionDay,
    end: &TransitionDay,
    seconds: i64,
) -> Result<(bool, TransitionType), TzifError> {
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
            let day_in_year = utils::epoch_time_to_day_in_year(seconds * 1_000) as u16;
            let is_transition = *start == day_in_year || *end == day_in_year;
            let is_dst = if start > end {
                day_in_year < *end || *start <= day_in_year
            } else {
                *start <= day_in_year && day_in_year < *end
            };
            (is_transition, is_dst)
        }
        // TODO: do we need to modify the logic for leap years?
        (TransitionDay::NoLeap(start), TransitionDay::NoLeap(end)) => {
            let day_in_year = utils::epoch_time_to_day_in_year(seconds * 1_000) as u16;
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
        _ => return Err(TzifError::UnalignedDayType),
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

#[derive(Debug)]
pub struct FsTzdbProvider {
    path: PathBuf,
    cache: RefCell<BTreeMap<String, Tzif>>,
}

impl Default for FsTzdbProvider {
    fn default() -> Self {
        Self {
            path: PathBuf::from(UNIX_ZONEINFO_DIR),
            cache: RefCell::default(),
        }
    }
}

impl FsTzdbProvider {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            cache: RefCell::default(),
        }
    }

    pub fn get(&self, identifier: &str) -> Result<Tzif, FsProviderError> {
        if let Some(tzif) = self.cache.borrow().get(identifier) {
            return Ok(tzif.clone());
        }
        let path = self.path.join(identifier);
        let (identifier, tzif) = { (identifier, Tzif::from_path(&path)?) };

        Ok(self
            .cache
            .borrow_mut()
            .entry(identifier.into())
            .or_insert(tzif)
            .clone())
    }
}

impl TimeZoneProvider for FsTzdbProvider {
    type Error = FsProviderError;
    fn check_identifier(&self, identifier: &str) -> bool {
        if let Some(index) = SINGLETON_IANA_NORMALIZER.available_id_index.get(identifier) {
            return SINGLETON_IANA_NORMALIZER
                .normalized_identifiers
                .get(index)
                .is_some();
        }
        false
    }

    fn get_possible_local_time_seconds(
        &self,
        identifier: &str,
        date_time_seconds: i64,
    ) -> Result<PotentialLocalTime, Self::Error> {
        let tzif = self.get(identifier)?;
        Ok(tzif.v2_estimate_tz_pair(&Seconds(date_time_seconds))?)
    }

    fn get_time_zone_offset(
        &self,
        identifier: &str,
        epoch_seconds: i64,
    ) -> Result<TimeZoneOffset, Self::Error> {
        let tzif = self.get(identifier)?;
        Ok(tzif.get(&Seconds(epoch_seconds))?)
    }

    fn get_time_zone_transition(
        &self,
        _identifier: &str,
        _epoch_seconds: i64,
        _direction: TransitionDirection,
    ) -> Result<Option<i64>, Self::Error> {
        Err(FsProviderError::NotYetImplemented)
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use tzif::data::time::Seconds;

    use crate::{
        fs::UNIX_ZONEINFO_DIR, utils, LocalTimeRecord, PotentialLocalTime, TimeZoneProvider,
    };

    use super::{FsTzdbProvider, Tzif, SINGLETON_IANA_NORMALIZER};

    fn date_time_to_seconds(date: (i32, u8, u8), time: (u8, u8, u8)) -> i64 {
        let time_in_seconds = i64::from(time.0) * 3600 + i64::from(time.1) * 60 + i64::from(time.2);
        let epoch_days =
            utils::neri_schneider::epoch_days_from_gregorian_date(date.0, date.1, date.2);
        time_in_seconds + (epoch_days * 86_400)
    }

    fn get_singleton_identifier(id: &str) -> Option<&'static str> {
        let index = SINGLETON_IANA_NORMALIZER.available_id_index.get(id)?;
        SINGLETON_IANA_NORMALIZER.normalized_identifiers.get(index)
    }

    #[test]
    fn test_singleton() {
        let id = get_singleton_identifier("uTc");
        assert_eq!(id, Some("UTC"));
        let id = get_singleton_identifier("EURope/BeRlIn").unwrap();
        assert_eq!(id, "Europe/Berlin");
    }

    #[test]
    fn available_ids() {
        let provider = FsTzdbProvider::default();
        assert!(provider.check_identifier("uTC"));
        assert!(provider.check_identifier("Etc/uTc"));
        assert!(provider.check_identifier("AMERIca/CHIcago"));
    }

    #[test]
    fn exactly_transition_time_after_empty_edge_case() {
        let provider = FsTzdbProvider::default();
        let date = (2017, 3, 12);
        let time = (3, 0, 0);
        let today = date_time_to_seconds(date, time);

        let local = provider
            .get_possible_local_time_seconds("America/New_York", today)
            .unwrap();
        assert_eq!(
            local,
            PotentialLocalTime::Single(LocalTimeRecord {
                is_dst: true,
                offset: -14400
            })
        );
    }

    #[test]
    fn one_second_before_empty_edge_case() {
        let provider = FsTzdbProvider::default();
        let date = (2017, 3, 12);
        let time = (2, 59, 59);
        let today = date_time_to_seconds(date, time);

        let local = provider
            .get_possible_local_time_seconds("America/New_York", today)
            .unwrap();
        assert_eq!(local, PotentialLocalTime::Empty);
    }

    #[test]
    fn new_york_empty_test_case() {
        let date = (2017, 3, 12);
        let time = (2, 30, 0);
        let edge_case = date_time_to_seconds(date, time);

        let path = PathBuf::from(UNIX_ZONEINFO_DIR).join("America/New_York");
        let new_york = Tzif::from_path(&path);

        assert!(new_york.is_ok());
        let new_york = new_york.unwrap();

        let locals = new_york.v2_estimate_tz_pair(&Seconds(edge_case)).unwrap();
        assert_eq!(locals, PotentialLocalTime::Empty);
    }

    #[test]
    fn sydney_empty_test_case() {
        // Australia Daylight savings day
        let date = (2017, 10, 1);
        let time = (2, 30, 0);
        let seconds = date_time_to_seconds(date, time);

        let path = PathBuf::from(UNIX_ZONEINFO_DIR).join("Australia/Sydney");
        let sydney = Tzif::from_path(&path);

        assert!(sydney.is_ok());
        let sydney = sydney.unwrap();

        let locals = sydney.v2_estimate_tz_pair(&Seconds(seconds)).unwrap();
        assert_eq!(locals, PotentialLocalTime::Empty);
    }

    #[test]
    fn new_york_duplicate_case() {
        let date = (2017, 11, 5);
        let time = (1, 30, 0);
        let seconds = date_time_to_seconds(date, time);

        let path = PathBuf::from(UNIX_ZONEINFO_DIR).join("America/New_York");
        let new_york = Tzif::from_path(&path);

        assert!(new_york.is_ok());
        let new_york = new_york.unwrap();

        let locals = new_york.v2_estimate_tz_pair(&Seconds(seconds)).unwrap();

        assert_eq!(
            locals,
            PotentialLocalTime::Ambiguous {
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
        let date = (2017, 4, 2);
        let time = (2, 30, 0);
        let seconds = date_time_to_seconds(date, time);

        let path = PathBuf::from(UNIX_ZONEINFO_DIR).join("Australia/Sydney");
        let sydney = Tzif::from_path(&path);

        assert!(sydney.is_ok());
        let sydney = sydney.unwrap();

        let locals = sydney.v2_estimate_tz_pair(&Seconds(seconds)).unwrap();

        assert_eq!(
            locals,
            PotentialLocalTime::Ambiguous {
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
        let date = (1880, 11, 5);
        let time = (1, 30, 0);
        let seconds = date_time_to_seconds(date, time);

        let path = PathBuf::from(UNIX_ZONEINFO_DIR).join("America/New_York");
        let new_york = Tzif::from_path(&path);

        assert!(new_york.is_ok());
        let new_york = new_york.unwrap();

        let locals = new_york.v2_estimate_tz_pair(&Seconds(seconds)).unwrap();

        assert!(matches!(locals, PotentialLocalTime::Single(_)));
    }

    // TODO: Determine the validity of this test. Primarily, this test
    // goes beyond the regularly historic limit of transition_times, so
    // even when on a DST boundary the first time zone is returned. The
    // question is whether this behavior is consistent with what would
    // be expected.
    #[test]
    fn before_epoch_southern_hemisphere() {
        // Australia Daylight savings day
        let date = (1880, 4, 2);
        let time = (2, 30, 0);
        let seconds = date_time_to_seconds(date, time);

        let path = PathBuf::from(UNIX_ZONEINFO_DIR).join("Australia/Sydney");
        let sydney = Tzif::from_path(&path);

        assert!(sydney.is_ok());
        let sydney = sydney.unwrap();

        let locals = sydney.v2_estimate_tz_pair(&Seconds(seconds)).unwrap();
        assert!(matches!(locals, PotentialLocalTime::Single(_)));
    }

    #[test]
    fn mwd_transition_epoch() {
        let path = PathBuf::from(UNIX_ZONEINFO_DIR).join("Europe/Berlin");
        let tzif = Tzif::from_path(&path).unwrap();
        let date = (2028, 3, 30);
        let time = (6, 0, 0);
        let start_dt_secs = date_time_to_seconds(date, time);
        let start_seconds = &Seconds(start_dt_secs);

        assert_eq!(
            tzif.get(start_seconds).unwrap().transition_epoch.unwrap(),
            // Sun, Mar 26 at 2:00 am
            1837645200
        );

        let date = (2028, 10, 29);
        let time = (6, 0, 0);
        let end_dt_secs = date_time_to_seconds(date, time);
        let end_seconds = &Seconds(end_dt_secs);

        assert_eq!(
            tzif.get(end_seconds).unwrap().transition_epoch.unwrap(),
            // Sun, Oct 29 at 3:00 am
            1856394000
        );
    }
}
