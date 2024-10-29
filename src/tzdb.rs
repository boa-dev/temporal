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

use tzif::{
    self,
    data::{
        time::Seconds,
        tzif::{DataBlock, LocalTimeTypeRecord, TzifData},
    },
};

use crate::{components::tz::TzProvider, iso::IsoDateTime, TemporalError, TemporalResult};

const ZONEINFO_DIR: &str = "/usr/share/zoneinfo/";

pub type TransitionInfo = [Option<LocalTimeTypeRecord>; 2];

#[derive(Debug)]
pub struct Tzif(TzifData);

impl Tzif {
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

    // There are ultimately
    pub fn get(&self, epoch_seconds: &Seconds) -> TemporalResult<LocalTimeTypeRecord> {
        let Some(result) = self.binary_search(epoch_seconds) else {
            return Err(TemporalError::general("Only Tzif v2+ is supported."));
        };

        let db = self
            .0
            .data_block2
            .as_ref()
            .expect("binary search throws error if datablock doesn't exist.");

        match result {
            Ok(idx) => Ok(get_local_record(db, idx - 1)),
            Err(idx) if idx == 0 => Ok(get_local_record(db, idx)),
            Err(idx) => {
                if db.transition_times.len() <= idx {
                    return Err(TemporalError::general("TODO: Support POSIX tz string."));
                }
                Ok(get_local_record(db, idx - 1))
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

    pub fn v2_estimate_tz_pair(
        &self,
        seconds: &Seconds,
    ) -> TemporalResult<Vec<LocalTimeTypeRecord>> {
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
            Err(idx) if idx == 0 => return Ok(vec![get_local_record(data_block, idx)]),
            Err(idx) => {
                if data_block.transition_times.len() <= idx {
                    return Err(TemporalError::general("TODO: Support POSIX tz string."));
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

        let offset_range = if initial_record.utoff < next_record.utoff {
            initial_record.utoff..next_record.utoff
        } else {
            next_record.utoff..initial_record.utoff
        };
        match offset_range.contains(&current_diff) {
            true if next_record.is_dst => Ok(Vec::default()),
            true => Ok(vec![next_record, initial_record]),
            false => Ok(vec![initial_record]),
        }
    }
}

#[inline]
fn get_local_record(db: &DataBlock, idx: usize) -> LocalTimeTypeRecord {
    db.local_time_type_records[db.transition_types[idx]]
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
        Ok(local_time_record_result
            .iter()
            .map(|r| r.utoff.0 as i128 * 1_000_000_000)
            .collect())
    }

    fn get_named_tz_offset_nanoseconds(
        &mut self,
        identifier: &str,
        epoch_nanoseconds: i128,
    ) -> TemporalResult<i128> {
        let tzif = self.get(identifier)?;
        let seconds = (epoch_nanoseconds / 1_000_000_000) as i64;
        let local_time_record_result = tzif.get(&Seconds(seconds))?;
        Ok(local_time_record_result.utoff.0 as i128 * 1_000_000_000)
    }
}
//

#[cfg(test)]
mod tests {
    use tzif::data::time::Seconds;

    use crate::{iso::IsoDateTime, tzdb::TzProvider};

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
        assert!(locals.is_empty());
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
        assert!(locals.is_empty());
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

        assert_eq!(locals.len(), 2);
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
        assert_eq!(locals.len(), 2);
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

        assert_eq!(locals.len(), 1);
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
        assert_eq!(locals.len(), 1);
    }
}
