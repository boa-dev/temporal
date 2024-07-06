use crate::{
    components::{calendar::Calendar, Date},
    options::{ArithmeticOverflow, RoundingIncrement, TemporalRoundingMode},
};

use super::*;

fn get_round_result(
    test_duration: &Duration,
    relative_to: &RelativeTo,
    options: RoundingOptions,
) -> Vec<i32> {
    test_duration
        .round(options, relative_to)
        .unwrap()
        .fields()
        .iter()
        .map(|f| *f as i32)
        .collect::<Vec<i32>>()
}

// roundingmode-floor.js
#[test]
fn basic_positive_floor_rounding_v2() {
    let test_duration =
        Duration::new(5.0, 6.0, 7.0, 8.0, 40.0, 30.0, 20.0, 123.0, 987.0, 500.0).unwrap();
    let forward_date = Date::new(
        2020,
        4,
        1,
        Calendar::from_str("iso8601").unwrap(),
        ArithmeticOverflow::Reject,
    )
    .unwrap();

    let relative_forward = RelativeTo {
        date: Some(&forward_date),
        zdt: None,
    };

    let mut options = RoundingOptions {
        largest_unit: None,
        smallest_unit: None,
        increment: None,
        rounding_mode: Some(TemporalRoundingMode::Floor),
    };

    let _ = options.smallest_unit.insert(TemporalUnit::Year);
    let result = get_round_result(&test_duration, &relative_forward, options);
    assert_eq!(&result, &[5, 0, 0, 0, 0, 0, 0, 0, 0, 0],);

    let _ = options.smallest_unit.insert(TemporalUnit::Month);
    let result = get_round_result(&test_duration, &relative_forward, options);
    assert_eq!(&result, &[5, 7, 0, 0, 0, 0, 0, 0, 0, 0],);

    let _ = options.smallest_unit.insert(TemporalUnit::Week);
    let result = get_round_result(&test_duration, &relative_forward, options);
    assert_eq!(&result, &[5, 7, 3, 0, 0, 0, 0, 0, 0, 0],);

    let _ = options.smallest_unit.insert(TemporalUnit::Day);
    let result = get_round_result(&test_duration, &relative_forward, options);
    assert_eq!(&result, &[5, 7, 0, 27, 0, 0, 0, 0, 0, 0],);

    let _ = options.smallest_unit.insert(TemporalUnit::Hour);
    let result = get_round_result(&test_duration, &relative_forward, options);
    assert_eq!(&result, &[5, 7, 0, 27, 16, 0, 0, 0, 0, 0],);

    let _ = options.smallest_unit.insert(TemporalUnit::Minute);
    let result = get_round_result(&test_duration, &relative_forward, options);
    assert_eq!(&result, &[5, 7, 0, 27, 16, 30, 0, 0, 0, 0],);

    let _ = options.smallest_unit.insert(TemporalUnit::Second);
    let result = get_round_result(&test_duration, &relative_forward, options);
    assert_eq!(&result, &[5, 7, 0, 27, 16, 30, 20, 0, 0, 0],);

    let _ = options.smallest_unit.insert(TemporalUnit::Millisecond);
    let result = get_round_result(&test_duration, &relative_forward, options);
    assert_eq!(&result, &[5, 7, 0, 27, 16, 30, 20, 123, 0, 0],);

    let _ = options.smallest_unit.insert(TemporalUnit::Microsecond);
    let result = get_round_result(&test_duration, &relative_forward, options);
    assert_eq!(&result, &[5, 7, 0, 27, 16, 30, 20, 123, 987, 0],);

    let _ = options.smallest_unit.insert(TemporalUnit::Nanosecond);
    let result = get_round_result(&test_duration, &relative_forward, options);
    assert_eq!(&result, &[5, 7, 0, 27, 16, 30, 20, 123, 987, 500],);
}

#[test]
fn basic_negative_floor_rounding_v2() {
    // Test setup
    let test_duration =
        Duration::new(5.0, 6.0, 7.0, 8.0, 40.0, 30.0, 20.0, 123.0, 987.0, 500.0).unwrap();
    let backward_date = Date::new(
        2020,
        12,
        1,
        Calendar::from_str("iso8601").unwrap(),
        ArithmeticOverflow::Reject,
    )
    .unwrap();

    let relative_backward = RelativeTo {
        date: Some(&backward_date),
        zdt: None,
    };

    let mut options = RoundingOptions {
        largest_unit: None,
        smallest_unit: None,
        increment: None,
        rounding_mode: Some(TemporalRoundingMode::Floor),
    };

    let _ = options.smallest_unit.insert(TemporalUnit::Year);
    let result = get_round_result(&test_duration.negated(), &relative_backward, options);
    assert_eq!(&result, &[-6, 0, 0, 0, 0, 0, 0, 0, 0, 0],);

    let _ = options.smallest_unit.insert(TemporalUnit::Month);
    let result = get_round_result(&test_duration.negated(), &relative_backward, options);
    assert_eq!(&result, &[-5, -8, 0, 0, 0, 0, 0, 0, 0, 0],);

    let _ = options.smallest_unit.insert(TemporalUnit::Week);
    let result = get_round_result(&test_duration.negated(), &relative_backward, options);
    assert_eq!(&result, &[-5, -7, -4, 0, 0, 0, 0, 0, 0, 0],);

    let _ = options.smallest_unit.insert(TemporalUnit::Day);
    let result = get_round_result(&test_duration.negated(), &relative_backward, options);
    assert_eq!(&result, &[-5, -7, 0, -28, 0, 0, 0, 0, 0, 0],);

    let _ = options.smallest_unit.insert(TemporalUnit::Hour);
    let result = get_round_result(&test_duration.negated(), &relative_backward, options);
    assert_eq!(&result, &[-5, -7, 0, -27, -17, 0, 0, 0, 0, 0],);

    let _ = options.smallest_unit.insert(TemporalUnit::Minute);
    let result = get_round_result(&test_duration.negated(), &relative_backward, options);
    assert_eq!(&result, &[-5, -7, 0, -27, -16, -31, 0, 0, 0, 0],);

    let _ = options.smallest_unit.insert(TemporalUnit::Second);
    let result = get_round_result(&test_duration.negated(), &relative_backward, options);
    assert_eq!(&result, &[-5, -7, 0, -27, -16, -30, -21, 0, 0, 0],);

    let _ = options.smallest_unit.insert(TemporalUnit::Millisecond);
    let result = get_round_result(&test_duration.negated(), &relative_backward, options);
    assert_eq!(&result, &[-5, -7, 0, -27, -16, -30, -20, -124, 0, 0],);

    let _ = options.smallest_unit.insert(TemporalUnit::Microsecond);
    let result = get_round_result(&test_duration.negated(), &relative_backward, options);
    assert_eq!(&result, &[-5, -7, 0, -27, -16, -30, -20, -123, -988, 0],);

    let _ = options.smallest_unit.insert(TemporalUnit::Nanosecond);
    let result = get_round_result(&test_duration.negated(), &relative_backward, options);
    assert_eq!(&result, &[-5, -7, 0, -27, -16, -30, -20, -123, -987, -500],);
}

// roundingmode-ceil.js
#[test]
fn basic_positive_ceil_rounding() {
    let test_duration =
        Duration::new(5.0, 6.0, 7.0, 8.0, 40.0, 30.0, 20.0, 123.0, 987.0, 500.0).unwrap();
    let forward_date = Date::new(
        2020,
        4,
        1,
        Calendar::from_str("iso8601").unwrap(),
        ArithmeticOverflow::Reject,
    )
    .unwrap();

    let relative_forward = RelativeTo {
        date: Some(&forward_date),
        zdt: None,
    };

    let mut options = RoundingOptions {
        largest_unit: None,
        smallest_unit: None,
        increment: None,
        rounding_mode: Some(TemporalRoundingMode::Ceil),
    };

    let _ = options.smallest_unit.insert(TemporalUnit::Year);
    let result = get_round_result(&test_duration, &relative_forward, options);
    assert_eq!(&result, &[6, 0, 0, 0, 0, 0, 0, 0, 0, 0],);

    let _ = options.smallest_unit.insert(TemporalUnit::Month);
    let result = get_round_result(&test_duration, &relative_forward, options);
    assert_eq!(&result, &[5, 8, 0, 0, 0, 0, 0, 0, 0, 0],);

    let _ = options.smallest_unit.insert(TemporalUnit::Week);
    let result = get_round_result(&test_duration, &relative_forward, options);
    assert_eq!(&result, &[5, 7, 4, 0, 0, 0, 0, 0, 0, 0],);

    let _ = options.smallest_unit.insert(TemporalUnit::Day);
    let result = get_round_result(&test_duration, &relative_forward, options);
    assert_eq!(&result, &[5, 7, 0, 28, 0, 0, 0, 0, 0, 0],);

    let _ = options.smallest_unit.insert(TemporalUnit::Hour);
    let result = get_round_result(&test_duration, &relative_forward, options);
    assert_eq!(&result, &[5, 7, 0, 27, 17, 0, 0, 0, 0, 0],);

    let _ = options.smallest_unit.insert(TemporalUnit::Minute);
    let result = get_round_result(&test_duration, &relative_forward, options);
    assert_eq!(&result, &[5, 7, 0, 27, 16, 31, 0, 0, 0, 0],);

    let _ = options.smallest_unit.insert(TemporalUnit::Second);
    let result = get_round_result(&test_duration, &relative_forward, options);
    assert_eq!(&result, &[5, 7, 0, 27, 16, 30, 21, 0, 0, 0],);

    let _ = options.smallest_unit.insert(TemporalUnit::Millisecond);
    let result = get_round_result(&test_duration, &relative_forward, options);
    assert_eq!(&result, &[5, 7, 0, 27, 16, 30, 20, 124, 0, 0],);

    let _ = options.smallest_unit.insert(TemporalUnit::Microsecond);
    let result = get_round_result(&test_duration, &relative_forward, options);
    assert_eq!(&result, &[5, 7, 0, 27, 16, 30, 20, 123, 988, 0],);

    let _ = options.smallest_unit.insert(TemporalUnit::Nanosecond);
    let result = get_round_result(&test_duration, &relative_forward, options);
    assert_eq!(&result, &[5, 7, 0, 27, 16, 30, 20, 123, 987, 500],);
}

#[test]
fn basic_negative_ceil_rounding() {
    let test_duration =
        Duration::new(5.0, 6.0, 7.0, 8.0, 40.0, 30.0, 20.0, 123.0, 987.0, 500.0).unwrap();
    let backward_date = Date::new(
        2020,
        12,
        1,
        Calendar::from_str("iso8601").unwrap(),
        ArithmeticOverflow::Reject,
    )
    .unwrap();
    let relative_backward = RelativeTo {
        date: Some(&backward_date),
        zdt: None,
    };

    let mut options = RoundingOptions {
        largest_unit: None,
        smallest_unit: None,
        increment: None,
        rounding_mode: Some(TemporalRoundingMode::Ceil),
    };

    let _ = options.smallest_unit.insert(TemporalUnit::Year);
    let result = get_round_result(&test_duration.negated(), &relative_backward, options);
    assert_eq!(&result, &[-5, 0, 0, 0, 0, 0, 0, 0, 0, 0],);

    let _ = options.smallest_unit.insert(TemporalUnit::Month);
    let result = get_round_result(&test_duration.negated(), &relative_backward, options);
    assert_eq!(&result, &[-5, -7, 0, 0, 0, 0, 0, 0, 0, 0],);

    let _ = options.smallest_unit.insert(TemporalUnit::Week);
    let result = get_round_result(&test_duration.negated(), &relative_backward, options);
    assert_eq!(&result, &[-5, -7, -3, 0, 0, 0, 0, 0, 0, 0],);

    let _ = options.smallest_unit.insert(TemporalUnit::Day);
    let result = get_round_result(&test_duration.negated(), &relative_backward, options);
    assert_eq!(&result, &[-5, -7, 0, -27, 0, 0, 0, 0, 0, 0],);

    let _ = options.smallest_unit.insert(TemporalUnit::Hour);
    let result = get_round_result(&test_duration.negated(), &relative_backward, options);
    assert_eq!(&result, &[-5, -7, 0, -27, -16, 0, 0, 0, 0, 0],);

    let _ = options.smallest_unit.insert(TemporalUnit::Minute);
    let result = get_round_result(&test_duration.negated(), &relative_backward, options);
    assert_eq!(&result, &[-5, -7, 0, -27, -16, -30, 0, 0, 0, 0],);

    let _ = options.smallest_unit.insert(TemporalUnit::Second);
    let result = get_round_result(&test_duration.negated(), &relative_backward, options);
    assert_eq!(&result, &[-5, -7, 0, -27, -16, -30, -20, 0, 0, 0],);

    let _ = options.smallest_unit.insert(TemporalUnit::Millisecond);
    let result = get_round_result(&test_duration.negated(), &relative_backward, options);
    assert_eq!(&result, &[-5, -7, 0, -27, -16, -30, -20, -123, 0, 0],);

    let _ = options.smallest_unit.insert(TemporalUnit::Microsecond);
    let result = get_round_result(&test_duration.negated(), &relative_backward, options);
    assert_eq!(&result, &[-5, -7, 0, -27, -16, -30, -20, -123, -987, 0],);

    let _ = options.smallest_unit.insert(TemporalUnit::Nanosecond);
    let result = get_round_result(&test_duration.negated(), &relative_backward, options);
    assert_eq!(&result, &[-5, -7, 0, -27, -16, -30, -20, -123, -987, -500],);
}

// roundingmode-expand.js
#[test]
fn basic_positive_expand_rounding() {
    let test_duration =
        Duration::new(5.0, 6.0, 7.0, 8.0, 40.0, 30.0, 20.0, 123.0, 987.0, 500.0).unwrap();
    let forward_date = Date::new(
        2020,
        4,
        1,
        Calendar::from_str("iso8601").unwrap(),
        ArithmeticOverflow::Reject,
    )
    .unwrap();

    let relative_forward = RelativeTo {
        date: Some(&forward_date),
        zdt: None,
    };

    let mut options = RoundingOptions {
        largest_unit: None,
        smallest_unit: None,
        increment: None,
        rounding_mode: Some(TemporalRoundingMode::Expand),
    };

    let _ = options.smallest_unit.insert(TemporalUnit::Year);
    let result = get_round_result(&test_duration, &relative_forward, options);
    assert_eq!(&result, &[6, 0, 0, 0, 0, 0, 0, 0, 0, 0],);

    let _ = options.smallest_unit.insert(TemporalUnit::Month);
    let result = get_round_result(&test_duration, &relative_forward, options);
    assert_eq!(&result, &[5, 8, 0, 0, 0, 0, 0, 0, 0, 0],);

    let _ = options.smallest_unit.insert(TemporalUnit::Week);
    let result = get_round_result(&test_duration, &relative_forward, options);
    assert_eq!(&result, &[5, 7, 4, 0, 0, 0, 0, 0, 0, 0],);

    let _ = options.smallest_unit.insert(TemporalUnit::Day);
    let result = get_round_result(&test_duration, &relative_forward, options);
    assert_eq!(&result, &[5, 7, 0, 28, 0, 0, 0, 0, 0, 0],);

    let _ = options.smallest_unit.insert(TemporalUnit::Hour);
    let result = get_round_result(&test_duration, &relative_forward, options);
    assert_eq!(&result, &[5, 7, 0, 27, 17, 0, 0, 0, 0, 0],);

    let _ = options.smallest_unit.insert(TemporalUnit::Minute);
    let result = get_round_result(&test_duration, &relative_forward, options);
    assert_eq!(&result, &[5, 7, 0, 27, 16, 31, 0, 0, 0, 0],);

    let _ = options.smallest_unit.insert(TemporalUnit::Second);
    let result = get_round_result(&test_duration, &relative_forward, options);
    assert_eq!(&result, &[5, 7, 0, 27, 16, 30, 21, 0, 0, 0],);

    let _ = options.smallest_unit.insert(TemporalUnit::Millisecond);
    let result = get_round_result(&test_duration, &relative_forward, options);
    assert_eq!(&result, &[5, 7, 0, 27, 16, 30, 20, 124, 0, 0],);

    let _ = options.smallest_unit.insert(TemporalUnit::Microsecond);
    let result = get_round_result(&test_duration, &relative_forward, options);
    assert_eq!(&result, &[5, 7, 0, 27, 16, 30, 20, 123, 988, 0],);

    let _ = options.smallest_unit.insert(TemporalUnit::Nanosecond);
    let result = get_round_result(&test_duration, &relative_forward, options);
    assert_eq!(&result, &[5, 7, 0, 27, 16, 30, 20, 123, 987, 500],);
}

#[test]
fn basic_negative_expand_rounding() {
    let test_duration =
        Duration::new(5.0, 6.0, 7.0, 8.0, 40.0, 30.0, 20.0, 123.0, 987.0, 500.0).unwrap();

    let backward_date = Date::new(
        2020,
        12,
        1,
        Calendar::from_str("iso8601").unwrap(),
        ArithmeticOverflow::Reject,
    )
    .unwrap();

    let relative_backward = RelativeTo {
        date: Some(&backward_date),
        zdt: None,
    };

    let mut options = RoundingOptions {
        largest_unit: None,
        smallest_unit: None,
        increment: None,
        rounding_mode: Some(TemporalRoundingMode::Expand),
    };

    let _ = options.smallest_unit.insert(TemporalUnit::Year);
    let result = get_round_result(&test_duration.negated(), &relative_backward, options);
    assert_eq!(&result, &[-6, 0, 0, 0, 0, 0, 0, 0, 0, 0],);

    let _ = options.smallest_unit.insert(TemporalUnit::Month);
    let result = get_round_result(&test_duration.negated(), &relative_backward, options);
    assert_eq!(&result, &[-5, -8, 0, 0, 0, 0, 0, 0, 0, 0],);

    let _ = options.smallest_unit.insert(TemporalUnit::Week);
    let result = get_round_result(&test_duration.negated(), &relative_backward, options);
    assert_eq!(&result, &[-5, -7, -4, 0, 0, 0, 0, 0, 0, 0],);

    let _ = options.smallest_unit.insert(TemporalUnit::Day);
    let result = get_round_result(&test_duration.negated(), &relative_backward, options);
    assert_eq!(&result, &[-5, -7, 0, -28, 0, 0, 0, 0, 0, 0],);

    let _ = options.smallest_unit.insert(TemporalUnit::Hour);
    let result = get_round_result(&test_duration.negated(), &relative_backward, options);
    assert_eq!(&result, &[-5, -7, 0, -27, -17, 0, 0, 0, 0, 0],);

    let _ = options.smallest_unit.insert(TemporalUnit::Minute);
    let result = get_round_result(&test_duration.negated(), &relative_backward, options);
    assert_eq!(&result, &[-5, -7, 0, -27, -16, -31, 0, 0, 0, 0],);

    let _ = options.smallest_unit.insert(TemporalUnit::Second);
    let result = get_round_result(&test_duration.negated(), &relative_backward, options);
    assert_eq!(&result, &[-5, -7, 0, -27, -16, -30, -21, 0, 0, 0],);

    let _ = options.smallest_unit.insert(TemporalUnit::Millisecond);
    let result = get_round_result(&test_duration.negated(), &relative_backward, options);
    assert_eq!(&result, &[-5, -7, 0, -27, -16, -30, -20, -124, 0, 0],);

    let _ = options.smallest_unit.insert(TemporalUnit::Microsecond);
    let result = get_round_result(&test_duration.negated(), &relative_backward, options);
    assert_eq!(&result, &[-5, -7, 0, -27, -16, -30, -20, -123, -988, 0],);

    let _ = options.smallest_unit.insert(TemporalUnit::Nanosecond);
    let result = get_round_result(&test_duration.negated(), &relative_backward, options);
    assert_eq!(&result, &[-5, -7, 0, -27, -16, -30, -20, -123, -987, -500],);
}

// test262/test/built-ins/Temporal/Duration/prototype/round/roundingincrement-non-integer.js
#[test]
fn rounding_increment_non_integer() {
    let test_duration =
        Duration::from_date_duration(&DateDuration::new(0.0, 0.0, 0.0, 1.0).unwrap());
    let binding = Date::new(
        2000,
        1,
        1,
        Calendar::from_str("iso8601").unwrap(),
        ArithmeticOverflow::Reject,
    )
    .unwrap();
    let relative_to = RelativeTo {
        date: Some(&binding),
        zdt: None,
    };

    let mut options = RoundingOptions {
        largest_unit: None,
        smallest_unit: Some(TemporalUnit::Day),
        increment: None,
        rounding_mode: Some(TemporalRoundingMode::Expand),
    };

    let _ = options
        .increment
        .insert(RoundingIncrement::try_from(2.5).unwrap());
    let result = test_duration.round(options, &relative_to).unwrap();

    assert_eq!(
        result.fields(),
        &[0.0, 0.0, 0.0, 2.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]
    );

    let _ = options
        .increment
        .insert(RoundingIncrement::try_from(1e9 + 0.5).unwrap());
    let result = test_duration.round(options, &relative_to).unwrap();
    assert_eq!(
        result.fields(),
        &[0.0, 0.0, 0.0, 1e9, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]
    );
}
