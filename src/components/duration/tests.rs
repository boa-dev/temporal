use crate::{
    components::{calendar::Calendar, Date},
    options::ArithmeticOverflow,
};

use super::*;

fn get_round_result(
    test_duration: &Duration,
    relative_to: &RelativeTo<'_>,
    unit: TemporalUnit,
    mode: TemporalRoundingMode,
) -> Vec<i32> {
    test_duration
        .round(None, Some(unit), None, Some(mode), relative_to)
        .unwrap()
        .fields()
        .iter()
        .map(|f| *f as i32)
        .collect::<Vec<i32>>()
}

fn get_round_result_v2(
    test_duration: &Duration,
    relative_to: &RelativeTo,
    unit: TemporalUnit,
    mode: TemporalRoundingMode,
) -> Vec<i32> {
    test_duration
        .round_v2(None, Some(unit), None, Some(mode), relative_to)
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

    let result = get_round_result_v2(
        &test_duration,
        &relative_forward,
        TemporalUnit::Year,
        TemporalRoundingMode::Floor,
    );
    assert_eq!(&result, &[5, 0, 0, 0, 0, 0, 0, 0, 0, 0],);

    let result = get_round_result_v2(
        &test_duration,
        &relative_forward,
        TemporalUnit::Month,
        TemporalRoundingMode::Floor,
    );
    assert_eq!(&result, &[5, 7, 0, 0, 0, 0, 0, 0, 0, 0],);

    let result = get_round_result_v2(
        &test_duration,
        &relative_forward,
        TemporalUnit::Week,
        TemporalRoundingMode::Floor,
    );
    assert_eq!(&result, &[5, 6, 8, 0, 0, 0, 0, 0, 0, 0],);

    let result = get_round_result_v2(
        &test_duration,
        &relative_forward,
        TemporalUnit::Day,
        TemporalRoundingMode::Floor,
    );
    assert_eq!(&result, &[5, 7, 0, 27, 0, 0, 0, 0, 0, 0],);

    let result = get_round_result_v2(
        &test_duration,
        &relative_forward,
        TemporalUnit::Hour,
        TemporalRoundingMode::Floor,
    );
    assert_eq!(&result, &[5, 7, 0, 27, 16, 0, 0, 0, 0, 0],);

    let result = get_round_result_v2(
        &test_duration,
        &relative_forward,
        TemporalUnit::Minute,
        TemporalRoundingMode::Floor,
    );
    assert_eq!(&result, &[5, 7, 0, 27, 16, 30, 0, 0, 0, 0],);

    let result = get_round_result_v2(
        &test_duration,
        &relative_forward,
        TemporalUnit::Second,
        TemporalRoundingMode::Floor,
    );
    assert_eq!(&result, &[5, 7, 0, 27, 16, 30, 20, 0, 0, 0],);

    let result = get_round_result_v2(
        &test_duration,
        &relative_forward,
        TemporalUnit::Millisecond,
        TemporalRoundingMode::Floor,
    );
    assert_eq!(&result, &[5, 7, 0, 27, 16, 30, 20, 123, 0, 0],);

    let result = get_round_result_v2(
        &test_duration,
        &relative_forward,
        TemporalUnit::Microsecond,
        TemporalRoundingMode::Floor,
    );
    assert_eq!(&result, &[5, 7, 0, 27, 16, 30, 20, 123, 987, 0],);

    let result = get_round_result_v2(
        &test_duration,
        &relative_forward,
        TemporalUnit::Nanosecond,
        TemporalRoundingMode::Floor,
    );
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

    let result = get_round_result_v2(
        &test_duration.negated(),
        &relative_backward,
        TemporalUnit::Year,
        TemporalRoundingMode::Floor,
    );
    assert_eq!(&result, &[-6, 0, 0, 0, 0, 0, 0, 0, 0, 0],);

    let result = get_round_result_v2(
        &test_duration.negated(),
        &relative_backward,
        TemporalUnit::Month,
        TemporalRoundingMode::Floor,
    );
    assert_eq!(&result, &[-5, -8, 0, 0, 0, 0, 0, 0, 0, 0],);

    let result = get_round_result_v2(
        &test_duration.negated(),
        &relative_backward,
        TemporalUnit::Week,
        TemporalRoundingMode::Floor,
    );
    assert_eq!(&result, &[-5, -6, -9, 0, 0, 0, 0, 0, 0, 0],);

    let result = get_round_result_v2(
        &test_duration.negated(),
        &relative_backward,
        TemporalUnit::Day,
        TemporalRoundingMode::Floor,
    );
    assert_eq!(&result, &[-5, -7, 0, -28, 0, 0, 0, 0, 0, 0],);

    let result = get_round_result_v2(
        &test_duration.negated(),
        &relative_backward,
        TemporalUnit::Hour,
        TemporalRoundingMode::Floor,
    );
    assert_eq!(&result, &[-5, -7, 0, -27, -17, 0, 0, 0, 0, 0],);

    let result = get_round_result_v2(
        &test_duration.negated(),
        &relative_backward,
        TemporalUnit::Minute,
        TemporalRoundingMode::Floor,
    );
    assert_eq!(&result, &[-5, -7, 0, -27, -16, -31, 0, 0, 0, 0],);

    let result = get_round_result_v2(
        &test_duration.negated(),
        &relative_backward,
        TemporalUnit::Second,
        TemporalRoundingMode::Floor,
    );
    assert_eq!(&result, &[-5, -7, 0, -27, -16, -30, -21, 0, 0, 0],);

    let result = get_round_result_v2(
        &test_duration.negated(),
        &relative_backward,
        TemporalUnit::Millisecond,
        TemporalRoundingMode::Floor,
    );
    assert_eq!(&result, &[-5, -7, 0, -27, -16, -30, -20, -124, 0, 0],);

    let result = get_round_result_v2(
        &test_duration.negated(),
        &relative_backward,
        TemporalUnit::Microsecond,
        TemporalRoundingMode::Floor,
    );
    assert_eq!(&result, &[-5, -7, 0, -27, -16, -30, -20, -123, -988, 0],);

    let result = get_round_result_v2(
        &test_duration.negated(),
        &relative_backward,
        TemporalUnit::Nanosecond,
        TemporalRoundingMode::Floor,
    );
    assert_eq!(&result, &[-5, -7, 0, -27, -16, -30, -20, -123, -987, -500],);
}

// roundingmode-floor.js
#[test]
fn basic_positive_floor_rounding() {
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

    let result = get_round_result(
        &test_duration,
        &relative_forward,
        TemporalUnit::Year,
        TemporalRoundingMode::Floor,
    );
    assert_eq!(&result, &[5, 0, 0, 0, 0, 0, 0, 0, 0, 0],);

    let result = get_round_result(
        &test_duration,
        &relative_forward,
        TemporalUnit::Month,
        TemporalRoundingMode::Floor,
    );
    assert_eq!(&result, &[5, 7, 0, 0, 0, 0, 0, 0, 0, 0],);

    let result = get_round_result(
        &test_duration,
        &relative_forward,
        TemporalUnit::Week,
        TemporalRoundingMode::Floor,
    );
    assert_eq!(&result, &[5, 6, 8, 0, 0, 0, 0, 0, 0, 0],);

    let result = get_round_result(
        &test_duration,
        &relative_forward,
        TemporalUnit::Day,
        TemporalRoundingMode::Floor,
    );
    assert_eq!(&result, &[5, 7, 0, 27, 0, 0, 0, 0, 0, 0],);

    let result = get_round_result(
        &test_duration,
        &relative_forward,
        TemporalUnit::Hour,
        TemporalRoundingMode::Floor,
    );
    assert_eq!(&result, &[5, 7, 0, 27, 16, 0, 0, 0, 0, 0],);

    let result = get_round_result(
        &test_duration,
        &relative_forward,
        TemporalUnit::Minute,
        TemporalRoundingMode::Floor,
    );
    assert_eq!(&result, &[5, 7, 0, 27, 16, 30, 0, 0, 0, 0],);

    let result = get_round_result(
        &test_duration,
        &relative_forward,
        TemporalUnit::Second,
        TemporalRoundingMode::Floor,
    );
    assert_eq!(&result, &[5, 7, 0, 27, 16, 30, 20, 0, 0, 0],);

    let result = get_round_result(
        &test_duration,
        &relative_forward,
        TemporalUnit::Millisecond,
        TemporalRoundingMode::Floor,
    );
    assert_eq!(&result, &[5, 7, 0, 27, 16, 30, 20, 123, 0, 0],);

    let result = get_round_result(
        &test_duration,
        &relative_forward,
        TemporalUnit::Microsecond,
        TemporalRoundingMode::Floor,
    );
    assert_eq!(&result, &[5, 7, 0, 27, 16, 30, 20, 123, 987, 0],);

    let result = get_round_result(
        &test_duration,
        &relative_forward,
        TemporalUnit::Nanosecond,
        TemporalRoundingMode::Floor,
    );
    assert_eq!(&result, &[5, 7, 0, 27, 16, 30, 20, 123, 987, 500],);
}

#[test]
fn basic_negative_floor_rounding() {
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

    let result = get_round_result(
        &test_duration.negated(),
        &relative_backward,
        TemporalUnit::Year,
        TemporalRoundingMode::Floor,
    );
    assert_eq!(&result, &[-6, 0, 0, 0, 0, 0, 0, 0, 0, 0],);

    let result = get_round_result(
        &test_duration.negated(),
        &relative_backward,
        TemporalUnit::Month,
        TemporalRoundingMode::Floor,
    );
    assert_eq!(&result, &[-5, -8, 0, 0, 0, 0, 0, 0, 0, 0],);

    let result = get_round_result(
        &test_duration.negated(),
        &relative_backward,
        TemporalUnit::Week,
        TemporalRoundingMode::Floor,
    );
    assert_eq!(&result, &[-5, -6, -9, 0, 0, 0, 0, 0, 0, 0],);

    let result = get_round_result(
        &test_duration.negated(),
        &relative_backward,
        TemporalUnit::Day,
        TemporalRoundingMode::Floor,
    );
    assert_eq!(&result, &[-5, -7, 0, -28, 0, 0, 0, 0, 0, 0],);

    let result = get_round_result(
        &test_duration.negated(),
        &relative_backward,
        TemporalUnit::Hour,
        TemporalRoundingMode::Floor,
    );
    assert_eq!(&result, &[-5, -7, 0, -27, -17, 0, 0, 0, 0, 0],);

    let result = get_round_result(
        &test_duration.negated(),
        &relative_backward,
        TemporalUnit::Minute,
        TemporalRoundingMode::Floor,
    );
    assert_eq!(&result, &[-5, -7, 0, -27, -16, -31, 0, 0, 0, 0],);

    let result = get_round_result(
        &test_duration.negated(),
        &relative_backward,
        TemporalUnit::Second,
        TemporalRoundingMode::Floor,
    );
    assert_eq!(&result, &[-5, -7, 0, -27, -16, -30, -21, 0, 0, 0],);

    let result = get_round_result(
        &test_duration.negated(),
        &relative_backward,
        TemporalUnit::Millisecond,
        TemporalRoundingMode::Floor,
    );
    assert_eq!(&result, &[-5, -7, 0, -27, -16, -30, -20, -124, 0, 0],);

    let result = get_round_result(
        &test_duration.negated(),
        &relative_backward,
        TemporalUnit::Microsecond,
        TemporalRoundingMode::Floor,
    );
    assert_eq!(&result, &[-5, -7, 0, -27, -16, -30, -20, -123, -988, 0],);

    let result = get_round_result(
        &test_duration.negated(),
        &relative_backward,
        TemporalUnit::Nanosecond,
        TemporalRoundingMode::Floor,
    );
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

    let result = get_round_result(
        &test_duration,
        &relative_forward,
        TemporalUnit::Year,
        TemporalRoundingMode::Ceil,
    );
    assert_eq!(&result, &[6, 0, 0, 0, 0, 0, 0, 0, 0, 0],);

    let result = get_round_result(
        &test_duration,
        &relative_forward,
        TemporalUnit::Month,
        TemporalRoundingMode::Ceil,
    );
    assert_eq!(&result, &[5, 8, 0, 0, 0, 0, 0, 0, 0, 0],);

    let result = get_round_result(
        &test_duration,
        &relative_forward,
        TemporalUnit::Week,
        TemporalRoundingMode::Ceil,
    );
    assert_eq!(&result, &[5, 6, 9, 0, 0, 0, 0, 0, 0, 0],);

    let result = get_round_result(
        &test_duration,
        &relative_forward,
        TemporalUnit::Day,
        TemporalRoundingMode::Ceil,
    );
    assert_eq!(&result, &[5, 7, 0, 28, 0, 0, 0, 0, 0, 0],);

    let result = get_round_result(
        &test_duration,
        &relative_forward,
        TemporalUnit::Hour,
        TemporalRoundingMode::Ceil,
    );
    assert_eq!(&result, &[5, 7, 0, 27, 17, 0, 0, 0, 0, 0],);

    let result = get_round_result(
        &test_duration,
        &relative_forward,
        TemporalUnit::Minute,
        TemporalRoundingMode::Ceil,
    );
    assert_eq!(&result, &[5, 7, 0, 27, 16, 31, 0, 0, 0, 0],);

    let result = get_round_result(
        &test_duration,
        &relative_forward,
        TemporalUnit::Second,
        TemporalRoundingMode::Ceil,
    );
    assert_eq!(&result, &[5, 7, 0, 27, 16, 30, 21, 0, 0, 0],);

    let result = get_round_result(
        &test_duration,
        &relative_forward,
        TemporalUnit::Millisecond,
        TemporalRoundingMode::Ceil,
    );
    assert_eq!(&result, &[5, 7, 0, 27, 16, 30, 20, 124, 0, 0],);

    let result = get_round_result(
        &test_duration,
        &relative_forward,
        TemporalUnit::Microsecond,
        TemporalRoundingMode::Ceil,
    );
    assert_eq!(&result, &[5, 7, 0, 27, 16, 30, 20, 123, 988, 0],);

    let result = get_round_result(
        &test_duration,
        &relative_forward,
        TemporalUnit::Nanosecond,
        TemporalRoundingMode::Ceil,
    );
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

    let result = get_round_result(
        &test_duration.negated(),
        &relative_backward,
        TemporalUnit::Year,
        TemporalRoundingMode::Ceil,
    );
    assert_eq!(&result, &[-5, 0, 0, 0, 0, 0, 0, 0, 0, 0],);

    let result = get_round_result(
        &test_duration.negated(),
        &relative_backward,
        TemporalUnit::Month,
        TemporalRoundingMode::Ceil,
    );
    assert_eq!(&result, &[-5, -7, 0, 0, 0, 0, 0, 0, 0, 0],);

    let result = get_round_result(
        &test_duration.negated(),
        &relative_backward,
        TemporalUnit::Week,
        TemporalRoundingMode::Ceil,
    );
    assert_eq!(&result, &[-5, -6, -8, 0, 0, 0, 0, 0, 0, 0],);

    let result = get_round_result(
        &test_duration.negated(),
        &relative_backward,
        TemporalUnit::Day,
        TemporalRoundingMode::Ceil,
    );
    assert_eq!(&result, &[-5, -7, 0, -27, 0, 0, 0, 0, 0, 0],);

    let result = get_round_result(
        &test_duration.negated(),
        &relative_backward,
        TemporalUnit::Hour,
        TemporalRoundingMode::Ceil,
    );
    assert_eq!(&result, &[-5, -7, 0, -27, -16, 0, 0, 0, 0, 0],);

    let result = get_round_result(
        &test_duration.negated(),
        &relative_backward,
        TemporalUnit::Minute,
        TemporalRoundingMode::Ceil,
    );
    assert_eq!(&result, &[-5, -7, 0, -27, -16, -30, 0, 0, 0, 0],);

    let result = get_round_result(
        &test_duration.negated(),
        &relative_backward,
        TemporalUnit::Second,
        TemporalRoundingMode::Ceil,
    );
    assert_eq!(&result, &[-5, -7, 0, -27, -16, -30, -20, 0, 0, 0],);

    let result = get_round_result(
        &test_duration.negated(),
        &relative_backward,
        TemporalUnit::Millisecond,
        TemporalRoundingMode::Ceil,
    );
    assert_eq!(&result, &[-5, -7, 0, -27, -16, -30, -20, -123, 0, 0],);

    let result = get_round_result(
        &test_duration.negated(),
        &relative_backward,
        TemporalUnit::Microsecond,
        TemporalRoundingMode::Ceil,
    );
    assert_eq!(&result, &[-5, -7, 0, -27, -16, -30, -20, -123, -987, 0],);

    let result = get_round_result(
        &test_duration.negated(),
        &relative_backward,
        TemporalUnit::Nanosecond,
        TemporalRoundingMode::Ceil,
    );
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

    let result = get_round_result(
        &test_duration,
        &relative_forward,
        TemporalUnit::Year,
        TemporalRoundingMode::Ceil,
    );
    assert_eq!(&result, &[6, 0, 0, 0, 0, 0, 0, 0, 0, 0],);

    let result = get_round_result(
        &test_duration,
        &relative_forward,
        TemporalUnit::Month,
        TemporalRoundingMode::Expand,
    );
    assert_eq!(&result, &[5, 8, 0, 0, 0, 0, 0, 0, 0, 0],);

    let result = get_round_result(
        &test_duration,
        &relative_forward,
        TemporalUnit::Week,
        TemporalRoundingMode::Expand,
    );
    assert_eq!(&result, &[5, 6, 9, 0, 0, 0, 0, 0, 0, 0],);

    let result = get_round_result(
        &test_duration,
        &relative_forward,
        TemporalUnit::Day,
        TemporalRoundingMode::Expand,
    );
    assert_eq!(&result, &[5, 7, 0, 28, 0, 0, 0, 0, 0, 0],);

    let result = get_round_result(
        &test_duration,
        &relative_forward,
        TemporalUnit::Hour,
        TemporalRoundingMode::Expand,
    );
    assert_eq!(&result, &[5, 7, 0, 27, 17, 0, 0, 0, 0, 0],);

    let result = get_round_result(
        &test_duration,
        &relative_forward,
        TemporalUnit::Minute,
        TemporalRoundingMode::Expand,
    );
    assert_eq!(&result, &[5, 7, 0, 27, 16, 31, 0, 0, 0, 0],);

    let result = get_round_result(
        &test_duration,
        &relative_forward,
        TemporalUnit::Second,
        TemporalRoundingMode::Expand,
    );
    assert_eq!(&result, &[5, 7, 0, 27, 16, 30, 21, 0, 0, 0],);

    let result = get_round_result(
        &test_duration,
        &relative_forward,
        TemporalUnit::Millisecond,
        TemporalRoundingMode::Expand,
    );
    assert_eq!(&result, &[5, 7, 0, 27, 16, 30, 20, 124, 0, 0],);

    let result = get_round_result(
        &test_duration,
        &relative_forward,
        TemporalUnit::Microsecond,
        TemporalRoundingMode::Expand,
    );
    assert_eq!(&result, &[5, 7, 0, 27, 16, 30, 20, 123, 988, 0],);

    let result = get_round_result(
        &test_duration,
        &relative_forward,
        TemporalUnit::Nanosecond,
        TemporalRoundingMode::Expand,
    );
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

    let result = get_round_result(
        &test_duration.negated(),
        &relative_backward,
        TemporalUnit::Year,
        TemporalRoundingMode::Expand,
    );
    assert_eq!(&result, &[-6, 0, 0, 0, 0, 0, 0, 0, 0, 0],);

    let result = get_round_result(
        &test_duration.negated(),
        &relative_backward,
        TemporalUnit::Month,
        TemporalRoundingMode::Expand,
    );
    assert_eq!(&result, &[-5, -8, 0, 0, 0, 0, 0, 0, 0, 0],);

    let result = get_round_result(
        &test_duration.negated(),
        &relative_backward,
        TemporalUnit::Week,
        TemporalRoundingMode::Expand,
    );
    assert_eq!(&result, &[-5, -6, -9, 0, 0, 0, 0, 0, 0, 0],);

    let result = get_round_result(
        &test_duration.negated(),
        &relative_backward,
        TemporalUnit::Day,
        TemporalRoundingMode::Expand,
    );
    assert_eq!(&result, &[-5, -7, 0, -28, 0, 0, 0, 0, 0, 0],);

    let result = get_round_result(
        &test_duration.negated(),
        &relative_backward,
        TemporalUnit::Hour,
        TemporalRoundingMode::Expand,
    );
    assert_eq!(&result, &[-5, -7, 0, -27, -17, 0, 0, 0, 0, 0],);

    let result = get_round_result(
        &test_duration.negated(),
        &relative_backward,
        TemporalUnit::Minute,
        TemporalRoundingMode::Expand,
    );
    assert_eq!(&result, &[-5, -7, 0, -27, -16, -31, 0, 0, 0, 0],);

    let result = get_round_result(
        &test_duration.negated(),
        &relative_backward,
        TemporalUnit::Second,
        TemporalRoundingMode::Expand,
    );
    assert_eq!(&result, &[-5, -7, 0, -27, -16, -30, -21, 0, 0, 0],);

    let result = get_round_result(
        &test_duration.negated(),
        &relative_backward,
        TemporalUnit::Millisecond,
        TemporalRoundingMode::Expand,
    );
    assert_eq!(&result, &[-5, -7, 0, -27, -16, -30, -20, -124, 0, 0],);

    let result = get_round_result(
        &test_duration.negated(),
        &relative_backward,
        TemporalUnit::Microsecond,
        TemporalRoundingMode::Expand,
    );
    assert_eq!(&result, &[-5, -7, 0, -27, -16, -30, -20, -123, -988, 0],);

    let result = get_round_result(
        &test_duration.negated(),
        &relative_backward,
        TemporalUnit::Nanosecond,
        TemporalRoundingMode::Expand,
    );
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

    assert_eq!(
        test_duration
            .round(
                Some(RoundingIncrement::try_from(2.5).unwrap()),
                Some(TemporalUnit::Day),
                None,
                Some(TemporalRoundingMode::Expand),
                &relative_to,
            )
            .unwrap()
            .fields(),
        &[0.0, 0.0, 0.0, 2.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]
    );

    assert_eq!(
        test_duration
            .round(
                Some(RoundingIncrement::try_from(1e9 + 0.5).unwrap()),
                Some(TemporalUnit::Day),
                None,
                Some(TemporalRoundingMode::Expand),
                &relative_to,
            )
            .unwrap()
            .fields(),
        &[0.0, 0.0, 0.0, 1e9, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]
    );
}
