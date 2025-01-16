#[cfg(feature = "experimental")]
use crate::{
    components::{calendar::Calendar, PlainDate},
    options::{RoundingIncrement, TemporalRoundingMode},
    TimeZone,
};

use super::*;

#[cfg(feature = "experimental")]
fn get_round_result(
    test_duration: &Duration,
    relative_to: RelativeTo,
    options: RoundingOptions,
) -> Vec<i32> {
    test_duration
        .round(options, Some(relative_to))
        .unwrap()
        .fields()
        .iter()
        .map(|f| f.as_date_value().unwrap())
        .collect::<Vec<i32>>()
}

// roundingmode-floor.js
#[cfg(feature = "experimental")]
#[test]
fn basic_positive_floor_rounding_v2() {
    let test_duration = Duration::new(
        FiniteF64(5.0),
        FiniteF64(6.0),
        FiniteF64(7.0),
        FiniteF64(8.0),
        FiniteF64(40.0),
        FiniteF64(30.0),
        FiniteF64(20.0),
        FiniteF64(123.0),
        FiniteF64(987.0),
        FiniteF64(500.0),
    )
    .unwrap();
    let forward_date = PlainDate::new(2020, 4, 1, Calendar::from_str("iso8601").unwrap()).unwrap();

    let relative_forward = RelativeTo::PlainDate(forward_date);

    let mut options = RoundingOptions {
        largest_unit: None,
        smallest_unit: None,
        increment: None,
        rounding_mode: Some(TemporalRoundingMode::Floor),
    };

    let _ = options.smallest_unit.insert(TemporalUnit::Year);
    let result = get_round_result(&test_duration, relative_forward.clone(), options);
    assert_eq!(&result, &[5, 0, 0, 0, 0, 0, 0, 0, 0, 0],);

    let _ = options.smallest_unit.insert(TemporalUnit::Month);
    let result = get_round_result(&test_duration, relative_forward.clone(), options);
    assert_eq!(&result, &[5, 7, 0, 0, 0, 0, 0, 0, 0, 0],);

    let _ = options.smallest_unit.insert(TemporalUnit::Week);
    let result = get_round_result(&test_duration, relative_forward.clone(), options);
    assert_eq!(&result, &[5, 7, 3, 0, 0, 0, 0, 0, 0, 0],);

    let _ = options.smallest_unit.insert(TemporalUnit::Day);
    let result = get_round_result(&test_duration, relative_forward.clone(), options);
    assert_eq!(&result, &[5, 7, 0, 27, 0, 0, 0, 0, 0, 0],);

    let _ = options.smallest_unit.insert(TemporalUnit::Hour);
    let result = get_round_result(&test_duration, relative_forward.clone(), options);
    assert_eq!(&result, &[5, 7, 0, 27, 16, 0, 0, 0, 0, 0],);

    let _ = options.smallest_unit.insert(TemporalUnit::Minute);
    let result = get_round_result(&test_duration, relative_forward.clone(), options);
    assert_eq!(&result, &[5, 7, 0, 27, 16, 30, 0, 0, 0, 0],);

    let _ = options.smallest_unit.insert(TemporalUnit::Second);
    let result = get_round_result(&test_duration, relative_forward.clone(), options);
    assert_eq!(&result, &[5, 7, 0, 27, 16, 30, 20, 0, 0, 0],);

    let _ = options.smallest_unit.insert(TemporalUnit::Millisecond);
    let result = get_round_result(&test_duration, relative_forward.clone(), options);
    assert_eq!(&result, &[5, 7, 0, 27, 16, 30, 20, 123, 0, 0],);

    let _ = options.smallest_unit.insert(TemporalUnit::Microsecond);
    let result = get_round_result(&test_duration, relative_forward.clone(), options);
    assert_eq!(&result, &[5, 7, 0, 27, 16, 30, 20, 123, 987, 0],);

    let _ = options.smallest_unit.insert(TemporalUnit::Nanosecond);
    let result = get_round_result(&test_duration, relative_forward.clone(), options);
    assert_eq!(&result, &[5, 7, 0, 27, 16, 30, 20, 123, 987, 500],);
}

#[test]
#[cfg(feature = "experimental")]
fn basic_negative_floor_rounding_v2() {
    // Test setup
    let test_duration = Duration::new(
        FiniteF64(5.0),
        FiniteF64(6.0),
        FiniteF64(7.0),
        FiniteF64(8.0),
        FiniteF64(40.0),
        FiniteF64(30.0),
        FiniteF64(20.0),
        FiniteF64(123.0),
        FiniteF64(987.0),
        FiniteF64(500.0),
    )
    .unwrap();
    let backward_date =
        PlainDate::new(2020, 12, 1, Calendar::from_str("iso8601").unwrap()).unwrap();

    let relative_backward = RelativeTo::PlainDate(backward_date);

    let mut options = RoundingOptions {
        largest_unit: None,
        smallest_unit: None,
        increment: None,
        rounding_mode: Some(TemporalRoundingMode::Floor),
    };

    let _ = options.smallest_unit.insert(TemporalUnit::Year);
    let result = get_round_result(&test_duration.negated(), relative_backward.clone(), options);
    assert_eq!(&result, &[-6, 0, 0, 0, 0, 0, 0, 0, 0, 0],);

    let _ = options.smallest_unit.insert(TemporalUnit::Month);
    let result = get_round_result(&test_duration.negated(), relative_backward.clone(), options);
    assert_eq!(&result, &[-5, -8, 0, 0, 0, 0, 0, 0, 0, 0],);

    let _ = options.smallest_unit.insert(TemporalUnit::Week);
    let result = get_round_result(&test_duration.negated(), relative_backward.clone(), options);
    assert_eq!(&result, &[-5, -7, -4, 0, 0, 0, 0, 0, 0, 0],);

    let _ = options.smallest_unit.insert(TemporalUnit::Day);
    let result = get_round_result(&test_duration.negated(), relative_backward.clone(), options);
    assert_eq!(&result, &[-5, -7, 0, -28, 0, 0, 0, 0, 0, 0],);

    let _ = options.smallest_unit.insert(TemporalUnit::Hour);
    let result = get_round_result(&test_duration.negated(), relative_backward.clone(), options);
    assert_eq!(&result, &[-5, -7, 0, -27, -17, 0, 0, 0, 0, 0],);

    let _ = options.smallest_unit.insert(TemporalUnit::Minute);
    let result = get_round_result(&test_duration.negated(), relative_backward.clone(), options);
    assert_eq!(&result, &[-5, -7, 0, -27, -16, -31, 0, 0, 0, 0],);

    let _ = options.smallest_unit.insert(TemporalUnit::Second);
    let result = get_round_result(&test_duration.negated(), relative_backward.clone(), options);
    assert_eq!(&result, &[-5, -7, 0, -27, -16, -30, -21, 0, 0, 0],);

    let _ = options.smallest_unit.insert(TemporalUnit::Millisecond);
    let result = get_round_result(&test_duration.negated(), relative_backward.clone(), options);
    assert_eq!(&result, &[-5, -7, 0, -27, -16, -30, -20, -124, 0, 0],);

    let _ = options.smallest_unit.insert(TemporalUnit::Microsecond);
    let result = get_round_result(&test_duration.negated(), relative_backward.clone(), options);
    assert_eq!(&result, &[-5, -7, 0, -27, -16, -30, -20, -123, -988, 0],);

    let _ = options.smallest_unit.insert(TemporalUnit::Nanosecond);
    let result = get_round_result(&test_duration.negated(), relative_backward.clone(), options);
    assert_eq!(&result, &[-5, -7, 0, -27, -16, -30, -20, -123, -987, -500],);
}

// roundingmode-ceil.js
#[cfg(feature = "experimental")]
#[test]
fn basic_positive_ceil_rounding() {
    let test_duration = Duration::new(
        FiniteF64(5.0),
        FiniteF64(6.0),
        FiniteF64(7.0),
        FiniteF64(8.0),
        FiniteF64(40.0),
        FiniteF64(30.0),
        FiniteF64(20.0),
        FiniteF64(123.0),
        FiniteF64(987.0),
        FiniteF64(500.0),
    )
    .unwrap();
    let forward_date = PlainDate::new(2020, 4, 1, Calendar::from_str("iso8601").unwrap()).unwrap();

    let relative_forward = RelativeTo::PlainDate(forward_date);

    let mut options = RoundingOptions {
        largest_unit: None,
        smallest_unit: None,
        increment: None,
        rounding_mode: Some(TemporalRoundingMode::Ceil),
    };

    let _ = options.smallest_unit.insert(TemporalUnit::Year);
    let result = get_round_result(&test_duration, relative_forward.clone(), options);
    assert_eq!(&result, &[6, 0, 0, 0, 0, 0, 0, 0, 0, 0],);

    let _ = options.smallest_unit.insert(TemporalUnit::Month);
    let result = get_round_result(&test_duration, relative_forward.clone(), options);
    assert_eq!(&result, &[5, 8, 0, 0, 0, 0, 0, 0, 0, 0],);

    let _ = options.smallest_unit.insert(TemporalUnit::Week);
    let result = get_round_result(&test_duration, relative_forward.clone(), options);
    assert_eq!(&result, &[5, 7, 4, 0, 0, 0, 0, 0, 0, 0],);

    let _ = options.smallest_unit.insert(TemporalUnit::Day);
    let result = get_round_result(&test_duration, relative_forward.clone(), options);
    assert_eq!(&result, &[5, 7, 0, 28, 0, 0, 0, 0, 0, 0],);

    let _ = options.smallest_unit.insert(TemporalUnit::Hour);
    let result = get_round_result(&test_duration, relative_forward.clone(), options);
    assert_eq!(&result, &[5, 7, 0, 27, 17, 0, 0, 0, 0, 0],);

    let _ = options.smallest_unit.insert(TemporalUnit::Minute);
    let result = get_round_result(&test_duration, relative_forward.clone(), options);
    assert_eq!(&result, &[5, 7, 0, 27, 16, 31, 0, 0, 0, 0],);

    let _ = options.smallest_unit.insert(TemporalUnit::Second);
    let result = get_round_result(&test_duration, relative_forward.clone(), options);
    assert_eq!(&result, &[5, 7, 0, 27, 16, 30, 21, 0, 0, 0],);

    let _ = options.smallest_unit.insert(TemporalUnit::Millisecond);
    let result = get_round_result(&test_duration, relative_forward.clone(), options);
    assert_eq!(&result, &[5, 7, 0, 27, 16, 30, 20, 124, 0, 0],);

    let _ = options.smallest_unit.insert(TemporalUnit::Microsecond);
    let result = get_round_result(&test_duration, relative_forward.clone(), options);
    assert_eq!(&result, &[5, 7, 0, 27, 16, 30, 20, 123, 988, 0],);

    let _ = options.smallest_unit.insert(TemporalUnit::Nanosecond);
    let result = get_round_result(&test_duration, relative_forward.clone(), options);
    assert_eq!(&result, &[5, 7, 0, 27, 16, 30, 20, 123, 987, 500],);
}

#[cfg(feature = "experimental")]
#[test]
fn basic_negative_ceil_rounding() {
    let test_duration = Duration::new(
        FiniteF64(5.0),
        FiniteF64(6.0),
        FiniteF64(7.0),
        FiniteF64(8.0),
        FiniteF64(40.0),
        FiniteF64(30.0),
        FiniteF64(20.0),
        FiniteF64(123.0),
        FiniteF64(987.0),
        FiniteF64(500.0),
    )
    .unwrap();
    let backward_date =
        PlainDate::new(2020, 12, 1, Calendar::from_str("iso8601").unwrap()).unwrap();
    let relative_backward = RelativeTo::PlainDate(backward_date);

    let mut options = RoundingOptions {
        largest_unit: None,
        smallest_unit: None,
        increment: None,
        rounding_mode: Some(TemporalRoundingMode::Ceil),
    };

    let _ = options.smallest_unit.insert(TemporalUnit::Year);
    let result = get_round_result(&test_duration.negated(), relative_backward.clone(), options);
    assert_eq!(&result, &[-5, 0, 0, 0, 0, 0, 0, 0, 0, 0],);

    let _ = options.smallest_unit.insert(TemporalUnit::Month);
    let result = get_round_result(&test_duration.negated(), relative_backward.clone(), options);
    assert_eq!(&result, &[-5, -7, 0, 0, 0, 0, 0, 0, 0, 0],);

    let _ = options.smallest_unit.insert(TemporalUnit::Week);
    let result = get_round_result(&test_duration.negated(), relative_backward.clone(), options);
    assert_eq!(&result, &[-5, -7, -3, 0, 0, 0, 0, 0, 0, 0],);

    let _ = options.smallest_unit.insert(TemporalUnit::Day);
    let result = get_round_result(&test_duration.negated(), relative_backward.clone(), options);
    assert_eq!(&result, &[-5, -7, 0, -27, 0, 0, 0, 0, 0, 0],);

    let _ = options.smallest_unit.insert(TemporalUnit::Hour);
    let result = get_round_result(&test_duration.negated(), relative_backward.clone(), options);
    assert_eq!(&result, &[-5, -7, 0, -27, -16, 0, 0, 0, 0, 0],);

    let _ = options.smallest_unit.insert(TemporalUnit::Minute);
    let result = get_round_result(&test_duration.negated(), relative_backward.clone(), options);
    assert_eq!(&result, &[-5, -7, 0, -27, -16, -30, 0, 0, 0, 0],);

    let _ = options.smallest_unit.insert(TemporalUnit::Second);
    let result = get_round_result(&test_duration.negated(), relative_backward.clone(), options);
    assert_eq!(&result, &[-5, -7, 0, -27, -16, -30, -20, 0, 0, 0],);

    let _ = options.smallest_unit.insert(TemporalUnit::Millisecond);
    let result = get_round_result(&test_duration.negated(), relative_backward.clone(), options);
    assert_eq!(&result, &[-5, -7, 0, -27, -16, -30, -20, -123, 0, 0],);

    let _ = options.smallest_unit.insert(TemporalUnit::Microsecond);
    let result = get_round_result(&test_duration.negated(), relative_backward.clone(), options);
    assert_eq!(&result, &[-5, -7, 0, -27, -16, -30, -20, -123, -987, 0],);

    let _ = options.smallest_unit.insert(TemporalUnit::Nanosecond);
    let result = get_round_result(&test_duration.negated(), relative_backward.clone(), options);
    assert_eq!(&result, &[-5, -7, 0, -27, -16, -30, -20, -123, -987, -500],);
}

// roundingmode-expand.js
#[cfg(feature = "experimental")]
#[test]
fn basic_positive_expand_rounding() {
    let test_duration = Duration::new(
        FiniteF64(5.0),
        FiniteF64(6.0),
        FiniteF64(7.0),
        FiniteF64(8.0),
        FiniteF64(40.0),
        FiniteF64(30.0),
        FiniteF64(20.0),
        FiniteF64(123.0),
        FiniteF64(987.0),
        FiniteF64(500.0),
    )
    .unwrap();
    let forward_date = PlainDate::new(2020, 4, 1, Calendar::from_str("iso8601").unwrap()).unwrap();
    let relative_forward = RelativeTo::PlainDate(forward_date);

    let mut options = RoundingOptions {
        largest_unit: None,
        smallest_unit: None,
        increment: None,
        rounding_mode: Some(TemporalRoundingMode::Expand),
    };

    let _ = options.smallest_unit.insert(TemporalUnit::Year);
    let result = get_round_result(&test_duration, relative_forward.clone(), options);
    assert_eq!(&result, &[6, 0, 0, 0, 0, 0, 0, 0, 0, 0],);

    let _ = options.smallest_unit.insert(TemporalUnit::Month);
    let result = get_round_result(&test_duration, relative_forward.clone(), options);
    assert_eq!(&result, &[5, 8, 0, 0, 0, 0, 0, 0, 0, 0],);

    let _ = options.smallest_unit.insert(TemporalUnit::Week);
    let result = get_round_result(&test_duration, relative_forward.clone(), options);
    assert_eq!(&result, &[5, 7, 4, 0, 0, 0, 0, 0, 0, 0],);

    let _ = options.smallest_unit.insert(TemporalUnit::Day);
    let result = get_round_result(&test_duration, relative_forward.clone(), options);
    assert_eq!(&result, &[5, 7, 0, 28, 0, 0, 0, 0, 0, 0],);

    let _ = options.smallest_unit.insert(TemporalUnit::Hour);
    let result = get_round_result(&test_duration, relative_forward.clone(), options);
    assert_eq!(&result, &[5, 7, 0, 27, 17, 0, 0, 0, 0, 0],);

    let _ = options.smallest_unit.insert(TemporalUnit::Minute);
    let result = get_round_result(&test_duration, relative_forward.clone(), options);
    assert_eq!(&result, &[5, 7, 0, 27, 16, 31, 0, 0, 0, 0],);

    let _ = options.smallest_unit.insert(TemporalUnit::Second);
    let result = get_round_result(&test_duration, relative_forward.clone(), options);
    assert_eq!(&result, &[5, 7, 0, 27, 16, 30, 21, 0, 0, 0],);

    let _ = options.smallest_unit.insert(TemporalUnit::Millisecond);
    let result = get_round_result(&test_duration, relative_forward.clone(), options);
    assert_eq!(&result, &[5, 7, 0, 27, 16, 30, 20, 124, 0, 0],);

    let _ = options.smallest_unit.insert(TemporalUnit::Microsecond);
    let result = get_round_result(&test_duration, relative_forward.clone(), options);
    assert_eq!(&result, &[5, 7, 0, 27, 16, 30, 20, 123, 988, 0],);

    let _ = options.smallest_unit.insert(TemporalUnit::Nanosecond);
    let result = get_round_result(&test_duration, relative_forward.clone(), options);
    assert_eq!(&result, &[5, 7, 0, 27, 16, 30, 20, 123, 987, 500],);
}

#[cfg(feature = "experimental")]
#[test]
fn basic_negative_expand_rounding() {
    let test_duration = Duration::new(
        FiniteF64(5.0),
        FiniteF64(6.0),
        FiniteF64(7.0),
        FiniteF64(8.0),
        FiniteF64(40.0),
        FiniteF64(30.0),
        FiniteF64(20.0),
        FiniteF64(123.0),
        FiniteF64(987.0),
        FiniteF64(500.0),
    )
    .unwrap();

    let backward_date =
        PlainDate::new(2020, 12, 1, Calendar::from_str("iso8601").unwrap()).unwrap();

    let relative_backward = RelativeTo::PlainDate(backward_date);

    let mut options = RoundingOptions {
        largest_unit: None,
        smallest_unit: None,
        increment: None,
        rounding_mode: Some(TemporalRoundingMode::Expand),
    };

    let _ = options.smallest_unit.insert(TemporalUnit::Year);
    let result = get_round_result(&test_duration.negated(), relative_backward.clone(), options);
    assert_eq!(&result, &[-6, 0, 0, 0, 0, 0, 0, 0, 0, 0],);

    let _ = options.smallest_unit.insert(TemporalUnit::Month);
    let result = get_round_result(&test_duration.negated(), relative_backward.clone(), options);
    assert_eq!(&result, &[-5, -8, 0, 0, 0, 0, 0, 0, 0, 0],);

    let _ = options.smallest_unit.insert(TemporalUnit::Week);
    let result = get_round_result(&test_duration.negated(), relative_backward.clone(), options);
    assert_eq!(&result, &[-5, -7, -4, 0, 0, 0, 0, 0, 0, 0],);

    let _ = options.smallest_unit.insert(TemporalUnit::Day);
    let result = get_round_result(&test_duration.negated(), relative_backward.clone(), options);
    assert_eq!(&result, &[-5, -7, 0, -28, 0, 0, 0, 0, 0, 0],);

    let _ = options.smallest_unit.insert(TemporalUnit::Hour);
    let result = get_round_result(&test_duration.negated(), relative_backward.clone(), options);
    assert_eq!(&result, &[-5, -7, 0, -27, -17, 0, 0, 0, 0, 0],);

    let _ = options.smallest_unit.insert(TemporalUnit::Minute);
    let result = get_round_result(&test_duration.negated(), relative_backward.clone(), options);
    assert_eq!(&result, &[-5, -7, 0, -27, -16, -31, 0, 0, 0, 0],);

    let _ = options.smallest_unit.insert(TemporalUnit::Second);
    let result = get_round_result(&test_duration.negated(), relative_backward.clone(), options);
    assert_eq!(&result, &[-5, -7, 0, -27, -16, -30, -21, 0, 0, 0],);

    let _ = options.smallest_unit.insert(TemporalUnit::Millisecond);
    let result = get_round_result(&test_duration.negated(), relative_backward.clone(), options);
    assert_eq!(&result, &[-5, -7, 0, -27, -16, -30, -20, -124, 0, 0],);

    let _ = options.smallest_unit.insert(TemporalUnit::Microsecond);
    let result = get_round_result(&test_duration.negated(), relative_backward.clone(), options);
    assert_eq!(&result, &[-5, -7, 0, -27, -16, -30, -20, -123, -988, 0],);

    let _ = options.smallest_unit.insert(TemporalUnit::Nanosecond);
    let result = get_round_result(&test_duration.negated(), relative_backward.clone(), options);
    assert_eq!(&result, &[-5, -7, 0, -27, -16, -30, -20, -123, -987, -500],);
}

// test262/test/built-ins/Temporal/Duration/prototype/round/roundingincrement-non-integer.js
#[cfg(feature = "experimental")]
#[test]
fn rounding_increment_non_integer() {
    let test_duration = Duration::from(
        DateDuration::new(
            FiniteF64::default(),
            FiniteF64::default(),
            FiniteF64::default(),
            FiniteF64(1.0),
        )
        .unwrap(),
    );
    let binding = PlainDate::new(2000, 1, 1, Calendar::from_str("iso8601").unwrap()).unwrap();
    let relative_to = RelativeTo::PlainDate(binding);

    let mut options = RoundingOptions {
        largest_unit: None,
        smallest_unit: Some(TemporalUnit::Day),
        increment: None,
        rounding_mode: Some(TemporalRoundingMode::Expand),
    };

    let _ = options
        .increment
        .insert(RoundingIncrement::try_from(2.5).unwrap());
    let result = test_duration
        .round(options, Some(relative_to.clone()))
        .unwrap();

    assert_eq!(
        result.fields(),
        &[
            FiniteF64::default(),
            FiniteF64::default(),
            FiniteF64::default(),
            FiniteF64(2.0),
            FiniteF64::default(),
            FiniteF64::default(),
            FiniteF64::default(),
            FiniteF64::default(),
            FiniteF64::default(),
            FiniteF64::default()
        ]
    );

    let _ = options
        .increment
        .insert(RoundingIncrement::try_from(1e9 + 0.5).unwrap());
    let result = test_duration.round(options, Some(relative_to)).unwrap();
    assert_eq!(
        result.fields(),
        &[
            FiniteF64::default(),
            FiniteF64::default(),
            FiniteF64::default(),
            FiniteF64(1e9),
            FiniteF64::default(),
            FiniteF64::default(),
            FiniteF64::default(),
            FiniteF64::default(),
            FiniteF64::default(),
            FiniteF64::default()
        ]
    );
}

#[test]
fn basic_add_duration() {
    let base = Duration::new(
        FiniteF64::default(),
        FiniteF64::default(),
        FiniteF64::default(),
        FiniteF64(1.0),
        FiniteF64::default(),
        FiniteF64(5.0),
        FiniteF64::default(),
        FiniteF64::default(),
        FiniteF64::default(),
        FiniteF64::default(),
    )
    .unwrap();
    let other = Duration::new(
        FiniteF64(0.0),
        FiniteF64::default(),
        FiniteF64::default(),
        FiniteF64(2.0),
        FiniteF64::default(),
        FiniteF64(5.0),
        FiniteF64::default(),
        FiniteF64::default(),
        FiniteF64::default(),
        FiniteF64::default(),
    )
    .unwrap();
    let result = base.add(&other).unwrap();
    assert_eq!(result.days(), 3.0);
    assert_eq!(result.minutes(), 10.0);

    let other = Duration::new(
        FiniteF64::default(),
        FiniteF64::default(),
        FiniteF64::default(),
        FiniteF64(-3.0),
        FiniteF64::default(),
        FiniteF64(-15.0),
        FiniteF64::default(),
        FiniteF64::default(),
        FiniteF64::default(),
        FiniteF64::default(),
    )
    .unwrap();
    let result = base.add(&other).unwrap();
    assert_eq!(result.days(), -2.0);
    assert_eq!(result.minutes(), -10.0);
}

#[test]
fn basic_subtract_duration() {
    let base = Duration::new(
        FiniteF64::default(),
        FiniteF64::default(),
        FiniteF64::default(),
        FiniteF64(3.0),
        FiniteF64::default(),
        FiniteF64(15.0),
        FiniteF64::default(),
        FiniteF64::default(),
        FiniteF64::default(),
        FiniteF64::default(),
    )
    .unwrap();
    let other = Duration::new(
        FiniteF64::default(),
        FiniteF64::default(),
        FiniteF64::default(),
        FiniteF64(1.0),
        FiniteF64::default(),
        FiniteF64(5.0),
        FiniteF64::default(),
        FiniteF64::default(),
        FiniteF64::default(),
        FiniteF64::default(),
    )
    .unwrap();
    let result = base.subtract(&other).unwrap();
    assert_eq!(result.days(), 2.0);
    assert_eq!(result.minutes(), 10.0);

    let other = Duration::new(
        FiniteF64::default(),
        FiniteF64::default(),
        FiniteF64::default(),
        FiniteF64(-3.0),
        FiniteF64::default(),
        FiniteF64(-15.0),
        FiniteF64::default(),
        FiniteF64::default(),
        FiniteF64::default(),
        FiniteF64::default(),
    )
    .unwrap();
    let result = base.subtract(&other).unwrap();
    assert_eq!(result.days(), 6.0);
    assert_eq!(result.minutes(), 30.0);
}

#[test]
fn partial_duration_empty() {
    let err = Duration::from_partial_duration(PartialDuration::default());
    assert!(err.is_err())
}

#[test]
fn partial_duration_values() {
    let mut partial = PartialDuration::default();
    let _ = partial.years.insert(FiniteF64(20.0));
    let result = Duration::from_partial_duration(partial).unwrap();
    assert_eq!(result.years(), 20.0);
}

// days-24-hours-relative-to-zoned-date-time.js
#[cfg(feature = "experimental")]
#[test]
fn round_relative_to_zoned_datetime() {
    let duration = Duration::from(
        TimeDuration::new(
            25.into(),
            FiniteF64::default(),
            FiniteF64::default(),
            FiniteF64::default(),
            FiniteF64::default(),
            FiniteF64::default(),
        )
        .unwrap(),
    );
    let zdt = ZonedDateTime::try_new(
        1_000_000_000_000_000_000,
        Calendar::default(),
        TimeZone::try_from_str("+04:30").unwrap(),
    )
    .unwrap();
    let options = RoundingOptions {
        largest_unit: Some(TemporalUnit::Day),
        smallest_unit: None,
        rounding_mode: None,
        increment: None,
    };
    let result = duration
        .round(options, Some(RelativeTo::ZonedDateTime(zdt)))
        .unwrap();
    // Result duration should be: (0, 0, 0, 1, 1, 0, 0, 0, 0, 0)
    assert_eq!(result.days(), 1.0);
    assert_eq!(result.hours(), 1.0);
}

#[test]
fn default_duration_string() {
    let duration = Duration::default();

    let options = ToStringRoundingOptions {
        precision: Precision::Auto,
        smallest_unit: None,
        rounding_mode: None,
    };
    let result = duration.to_temporal_string(options).unwrap();
    assert_eq!(&result, "PT0S");

    let options = ToStringRoundingOptions {
        precision: Precision::Digit(0),
        smallest_unit: None,
        rounding_mode: None,
    };
    let result = duration.to_temporal_string(options).unwrap();
    assert_eq!(&result, "PT0S");

    let options = ToStringRoundingOptions {
        precision: Precision::Digit(1),
        smallest_unit: None,
        rounding_mode: None,
    };
    let result = duration.to_temporal_string(options).unwrap();
    assert_eq!(&result, "PT0.0S");

    let options = ToStringRoundingOptions {
        precision: Precision::Digit(3),
        smallest_unit: None,
        rounding_mode: None,
    };
    let result = duration.to_temporal_string(options).unwrap();
    assert_eq!(&result, "PT0.000S");
}

#[test]
fn duration_to_string_auto_precision() {
    let duration = Duration::new(1.into(), 2.into(), 3.into(), 4.into(), 5.into(), 6.into(), 7.into(), FiniteF64::default(), FiniteF64::default(), FiniteF64::default()).unwrap();
    let result = duration.to_temporal_string(ToStringRoundingOptions::default()).unwrap();
    assert_eq!(&result, "P1Y2M3W4DT5H6M7S");

    let duration = Duration::new(1.into(), 2.into(), 3.into(), 4.into(), 5.into(), 6.into(), 7.into(), 987.into(), 650.into(), FiniteF64::default()).unwrap();
    let result = duration.to_temporal_string(ToStringRoundingOptions::default()).unwrap();
    assert_eq!(&result, "P1Y2M3W4DT5H6M7.98765S");
}

#[test]
fn empty_date_duration() {
    let duration = Duration::from_partial_duration(PartialDuration {
        hours: Some(1.into()),
        ..Default::default()
    }).unwrap();
    let result = duration.to_temporal_string(ToStringRoundingOptions::default()).unwrap();
    assert_eq!(&result, "PT1H");
}

#[test]
fn negative_fields_to_string() {
    let duration = Duration::from_partial_duration(PartialDuration {
        years: Some(FiniteF64::from(-1)),
        months: Some(FiniteF64::from(-1)),
        weeks: Some(FiniteF64::from(-1)),
        days: Some(FiniteF64::from(-1)),
        hours: Some(FiniteF64::from(-1)),
        minutes: Some(FiniteF64::from(-1)),
        seconds: Some(FiniteF64::from(-1)),
        milliseconds: Some(FiniteF64::from(-1)),
        microseconds: Some(FiniteF64::from(-1)),
        nanoseconds: Some(FiniteF64::from(-1)),
    }).unwrap();
    let result = duration.to_temporal_string(ToStringRoundingOptions::default()).unwrap();
    assert_eq!(&result, "-P1Y1M1W1DT1H1M1.001001001S");

    let duration = Duration::from_partial_duration(PartialDuration {
        milliseconds: Some(FiniteF64::from(-250)),
        ..Default::default()
    }).unwrap();
    let result = duration.to_temporal_string(ToStringRoundingOptions::default()).unwrap();
    assert_eq!(&result, "-PT0.25S");

    let duration = Duration::from_partial_duration(PartialDuration {
        milliseconds: Some(FiniteF64::from(-3500)),
        ..Default::default()
    }).unwrap();
    let result = duration.to_temporal_string(ToStringRoundingOptions::default()).unwrap();
    assert_eq!(&result, "-PT3.5S");

    let duration = Duration::from_partial_duration(PartialDuration {
        milliseconds: Some(FiniteF64::from(-3500)),
        ..Default::default()
    }).unwrap();
    let result = duration.to_temporal_string(ToStringRoundingOptions::default()).unwrap();
    assert_eq!(&result, "-PT3.5S");

    let duration = Duration::from_partial_duration(PartialDuration {
        weeks: Some(FiniteF64::from(-1)),
        days: Some(FiniteF64::from(-1)),
        ..Default::default()
    }).unwrap();
    let result = duration.to_temporal_string(ToStringRoundingOptions::default()).unwrap();
    assert_eq!(&result, "-P1W1D");
}
