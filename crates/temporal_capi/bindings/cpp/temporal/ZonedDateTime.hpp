#ifndef TEMPORAL_ZonedDateTime_HPP
#define TEMPORAL_ZonedDateTime_HPP

#include "ZonedDateTime.d.hpp"

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <functional>
#include <optional>
#include <cstdlib>
#include "AnyCalendarKind.hpp"
#include "ArithmeticOverflow.hpp"
#include "Calendar.hpp"
#include "DifferenceSettings.hpp"
#include "Disambiguation.hpp"
#include "DisplayCalendar.hpp"
#include "DisplayOffset.hpp"
#include "DisplayTimeZone.hpp"
#include "Duration.hpp"
#include "I128Nanoseconds.hpp"
#include "Instant.hpp"
#include "OffsetDisambiguation.hpp"
#include "ParsedZonedDateTime.hpp"
#include "PartialZonedDateTime.hpp"
#include "PlainDate.hpp"
#include "PlainDateTime.hpp"
#include "PlainTime.hpp"
#include "Provider.hpp"
#include "RoundingOptions.hpp"
#include "TemporalError.hpp"
#include "TimeZone.hpp"
#include "ToStringRoundingOptions.hpp"
#include "TransitionDirection.hpp"
#include "diplomat_runtime.hpp"


namespace temporal {
namespace capi {
    extern "C" {

    typedef struct temporal_ZonedDateTime_try_new_result {union {temporal::capi::ZonedDateTime* ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_ZonedDateTime_try_new_result;
    temporal_ZonedDateTime_try_new_result temporal_ZonedDateTime_try_new(temporal::capi::I128Nanoseconds nanosecond, temporal::capi::AnyCalendarKind calendar, temporal::capi::TimeZone time_zone);

    typedef struct temporal_ZonedDateTime_try_new_with_provider_result {union {temporal::capi::ZonedDateTime* ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_ZonedDateTime_try_new_with_provider_result;
    temporal_ZonedDateTime_try_new_with_provider_result temporal_ZonedDateTime_try_new_with_provider(temporal::capi::I128Nanoseconds nanosecond, temporal::capi::AnyCalendarKind calendar, temporal::capi::TimeZone time_zone, const temporal::capi::Provider* p);

    typedef struct temporal_ZonedDateTime_from_partial_result {union {temporal::capi::ZonedDateTime* ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_ZonedDateTime_from_partial_result;
    temporal_ZonedDateTime_from_partial_result temporal_ZonedDateTime_from_partial(temporal::capi::PartialZonedDateTime partial, temporal::capi::ArithmeticOverflow_option overflow, temporal::capi::Disambiguation_option disambiguation, temporal::capi::OffsetDisambiguation_option offset_option);

    typedef struct temporal_ZonedDateTime_from_partial_with_provider_result {union {temporal::capi::ZonedDateTime* ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_ZonedDateTime_from_partial_with_provider_result;
    temporal_ZonedDateTime_from_partial_with_provider_result temporal_ZonedDateTime_from_partial_with_provider(temporal::capi::PartialZonedDateTime partial, temporal::capi::ArithmeticOverflow_option overflow, temporal::capi::Disambiguation_option disambiguation, temporal::capi::OffsetDisambiguation_option offset_option, const temporal::capi::Provider* p);

    typedef struct temporal_ZonedDateTime_from_parsed_result {union {temporal::capi::ZonedDateTime* ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_ZonedDateTime_from_parsed_result;
    temporal_ZonedDateTime_from_parsed_result temporal_ZonedDateTime_from_parsed(const temporal::capi::ParsedZonedDateTime* parsed, temporal::capi::Disambiguation disambiguation, temporal::capi::OffsetDisambiguation offset_option);

    typedef struct temporal_ZonedDateTime_from_parsed_with_provider_result {union {temporal::capi::ZonedDateTime* ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_ZonedDateTime_from_parsed_with_provider_result;
    temporal_ZonedDateTime_from_parsed_with_provider_result temporal_ZonedDateTime_from_parsed_with_provider(const temporal::capi::ParsedZonedDateTime* parsed, temporal::capi::Disambiguation disambiguation, temporal::capi::OffsetDisambiguation offset_option, const temporal::capi::Provider* p);

    typedef struct temporal_ZonedDateTime_from_utf8_result {union {temporal::capi::ZonedDateTime* ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_ZonedDateTime_from_utf8_result;
    temporal_ZonedDateTime_from_utf8_result temporal_ZonedDateTime_from_utf8(temporal::diplomat::capi::DiplomatStringView s, temporal::capi::Disambiguation disambiguation, temporal::capi::OffsetDisambiguation offset_disambiguation);

    typedef struct temporal_ZonedDateTime_from_utf8_with_provider_result {union {temporal::capi::ZonedDateTime* ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_ZonedDateTime_from_utf8_with_provider_result;
    temporal_ZonedDateTime_from_utf8_with_provider_result temporal_ZonedDateTime_from_utf8_with_provider(temporal::diplomat::capi::DiplomatStringView s, temporal::capi::Disambiguation disambiguation, temporal::capi::OffsetDisambiguation offset_disambiguation, const temporal::capi::Provider* p);

    typedef struct temporal_ZonedDateTime_from_utf16_result {union {temporal::capi::ZonedDateTime* ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_ZonedDateTime_from_utf16_result;
    temporal_ZonedDateTime_from_utf16_result temporal_ZonedDateTime_from_utf16(temporal::diplomat::capi::DiplomatString16View s, temporal::capi::Disambiguation disambiguation, temporal::capi::OffsetDisambiguation offset_disambiguation);

    typedef struct temporal_ZonedDateTime_from_utf16_with_provider_result {union {temporal::capi::ZonedDateTime* ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_ZonedDateTime_from_utf16_with_provider_result;
    temporal_ZonedDateTime_from_utf16_with_provider_result temporal_ZonedDateTime_from_utf16_with_provider(temporal::diplomat::capi::DiplomatString16View s, temporal::capi::Disambiguation disambiguation, temporal::capi::OffsetDisambiguation offset_disambiguation, const temporal::capi::Provider* p);

    int64_t temporal_ZonedDateTime_epoch_milliseconds(const temporal::capi::ZonedDateTime* self);

    typedef struct temporal_ZonedDateTime_from_epoch_milliseconds_result {union {temporal::capi::ZonedDateTime* ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_ZonedDateTime_from_epoch_milliseconds_result;
    temporal_ZonedDateTime_from_epoch_milliseconds_result temporal_ZonedDateTime_from_epoch_milliseconds(int64_t ms, temporal::capi::TimeZone tz);

    typedef struct temporal_ZonedDateTime_from_epoch_milliseconds_with_provider_result {union {temporal::capi::ZonedDateTime* ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_ZonedDateTime_from_epoch_milliseconds_with_provider_result;
    temporal_ZonedDateTime_from_epoch_milliseconds_with_provider_result temporal_ZonedDateTime_from_epoch_milliseconds_with_provider(int64_t ms, temporal::capi::TimeZone tz, const temporal::capi::Provider* p);

    typedef struct temporal_ZonedDateTime_from_epoch_nanoseconds_result {union {temporal::capi::ZonedDateTime* ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_ZonedDateTime_from_epoch_nanoseconds_result;
    temporal_ZonedDateTime_from_epoch_nanoseconds_result temporal_ZonedDateTime_from_epoch_nanoseconds(temporal::capi::I128Nanoseconds ns, temporal::capi::TimeZone tz);

    typedef struct temporal_ZonedDateTime_from_epoch_nanoseconds_with_provider_result {union {temporal::capi::ZonedDateTime* ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_ZonedDateTime_from_epoch_nanoseconds_with_provider_result;
    temporal_ZonedDateTime_from_epoch_nanoseconds_with_provider_result temporal_ZonedDateTime_from_epoch_nanoseconds_with_provider(temporal::capi::I128Nanoseconds ns, temporal::capi::TimeZone tz, const temporal::capi::Provider* p);

    temporal::capi::I128Nanoseconds temporal_ZonedDateTime_epoch_nanoseconds(const temporal::capi::ZonedDateTime* self);

    int64_t temporal_ZonedDateTime_offset_nanoseconds(const temporal::capi::ZonedDateTime* self);

    temporal::capi::Instant* temporal_ZonedDateTime_to_instant(const temporal::capi::ZonedDateTime* self);

    typedef struct temporal_ZonedDateTime_with_result {union {temporal::capi::ZonedDateTime* ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_ZonedDateTime_with_result;
    temporal_ZonedDateTime_with_result temporal_ZonedDateTime_with(const temporal::capi::ZonedDateTime* self, temporal::capi::PartialZonedDateTime partial, temporal::capi::Disambiguation_option disambiguation, temporal::capi::OffsetDisambiguation_option offset_option, temporal::capi::ArithmeticOverflow_option overflow);

    typedef struct temporal_ZonedDateTime_with_with_provider_result {union {temporal::capi::ZonedDateTime* ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_ZonedDateTime_with_with_provider_result;
    temporal_ZonedDateTime_with_with_provider_result temporal_ZonedDateTime_with_with_provider(const temporal::capi::ZonedDateTime* self, temporal::capi::PartialZonedDateTime partial, temporal::capi::Disambiguation_option disambiguation, temporal::capi::OffsetDisambiguation_option offset_option, temporal::capi::ArithmeticOverflow_option overflow, const temporal::capi::Provider* p);

    typedef struct temporal_ZonedDateTime_with_timezone_result {union {temporal::capi::ZonedDateTime* ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_ZonedDateTime_with_timezone_result;
    temporal_ZonedDateTime_with_timezone_result temporal_ZonedDateTime_with_timezone(const temporal::capi::ZonedDateTime* self, temporal::capi::TimeZone zone);

    typedef struct temporal_ZonedDateTime_with_timezone_with_provider_result {union {temporal::capi::ZonedDateTime* ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_ZonedDateTime_with_timezone_with_provider_result;
    temporal_ZonedDateTime_with_timezone_with_provider_result temporal_ZonedDateTime_with_timezone_with_provider(const temporal::capi::ZonedDateTime* self, temporal::capi::TimeZone zone, const temporal::capi::Provider* p);

    temporal::capi::TimeZone temporal_ZonedDateTime_timezone(const temporal::capi::ZonedDateTime* self);

    int8_t temporal_ZonedDateTime_compare_instant(const temporal::capi::ZonedDateTime* self, const temporal::capi::ZonedDateTime* other);

    bool temporal_ZonedDateTime_equals(const temporal::capi::ZonedDateTime* self, const temporal::capi::ZonedDateTime* other);

    typedef struct temporal_ZonedDateTime_equals_with_provider_result {union {bool ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_ZonedDateTime_equals_with_provider_result;
    temporal_ZonedDateTime_equals_with_provider_result temporal_ZonedDateTime_equals_with_provider(const temporal::capi::ZonedDateTime* self, const temporal::capi::ZonedDateTime* other, const temporal::capi::Provider* p);

    typedef struct temporal_ZonedDateTime_offset_result {union { temporal::capi::TemporalError err;}; bool is_ok;} temporal_ZonedDateTime_offset_result;
    temporal_ZonedDateTime_offset_result temporal_ZonedDateTime_offset(const temporal::capi::ZonedDateTime* self, temporal::diplomat::capi::DiplomatWrite* write);

    typedef struct temporal_ZonedDateTime_start_of_day_result {union {temporal::capi::ZonedDateTime* ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_ZonedDateTime_start_of_day_result;
    temporal_ZonedDateTime_start_of_day_result temporal_ZonedDateTime_start_of_day(const temporal::capi::ZonedDateTime* self);

    typedef struct temporal_ZonedDateTime_start_of_day_with_provider_result {union {temporal::capi::ZonedDateTime* ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_ZonedDateTime_start_of_day_with_provider_result;
    temporal_ZonedDateTime_start_of_day_with_provider_result temporal_ZonedDateTime_start_of_day_with_provider(const temporal::capi::ZonedDateTime* self, const temporal::capi::Provider* p);

    typedef struct temporal_ZonedDateTime_get_time_zone_transition_result {union {temporal::capi::ZonedDateTime* ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_ZonedDateTime_get_time_zone_transition_result;
    temporal_ZonedDateTime_get_time_zone_transition_result temporal_ZonedDateTime_get_time_zone_transition(const temporal::capi::ZonedDateTime* self, temporal::capi::TransitionDirection direction);

    typedef struct temporal_ZonedDateTime_get_time_zone_transition_with_provider_result {union {temporal::capi::ZonedDateTime* ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_ZonedDateTime_get_time_zone_transition_with_provider_result;
    temporal_ZonedDateTime_get_time_zone_transition_with_provider_result temporal_ZonedDateTime_get_time_zone_transition_with_provider(const temporal::capi::ZonedDateTime* self, temporal::capi::TransitionDirection direction, const temporal::capi::Provider* p);

    typedef struct temporal_ZonedDateTime_hours_in_day_result {union {double ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_ZonedDateTime_hours_in_day_result;
    temporal_ZonedDateTime_hours_in_day_result temporal_ZonedDateTime_hours_in_day(const temporal::capi::ZonedDateTime* self);

    typedef struct temporal_ZonedDateTime_hours_in_day_with_provider_result {union {double ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_ZonedDateTime_hours_in_day_with_provider_result;
    temporal_ZonedDateTime_hours_in_day_with_provider_result temporal_ZonedDateTime_hours_in_day_with_provider(const temporal::capi::ZonedDateTime* self, const temporal::capi::Provider* p);

    temporal::capi::PlainDateTime* temporal_ZonedDateTime_to_plain_datetime(const temporal::capi::ZonedDateTime* self);

    temporal::capi::PlainDate* temporal_ZonedDateTime_to_plain_date(const temporal::capi::ZonedDateTime* self);

    temporal::capi::PlainTime* temporal_ZonedDateTime_to_plain_time(const temporal::capi::ZonedDateTime* self);

    typedef struct temporal_ZonedDateTime_to_ixdtf_string_result {union { temporal::capi::TemporalError err;}; bool is_ok;} temporal_ZonedDateTime_to_ixdtf_string_result;
    temporal_ZonedDateTime_to_ixdtf_string_result temporal_ZonedDateTime_to_ixdtf_string(const temporal::capi::ZonedDateTime* self, temporal::capi::DisplayOffset display_offset, temporal::capi::DisplayTimeZone display_timezone, temporal::capi::DisplayCalendar display_calendar, temporal::capi::ToStringRoundingOptions options, temporal::diplomat::capi::DiplomatWrite* write);

    typedef struct temporal_ZonedDateTime_to_ixdtf_string_with_provider_result {union { temporal::capi::TemporalError err;}; bool is_ok;} temporal_ZonedDateTime_to_ixdtf_string_with_provider_result;
    temporal_ZonedDateTime_to_ixdtf_string_with_provider_result temporal_ZonedDateTime_to_ixdtf_string_with_provider(const temporal::capi::ZonedDateTime* self, temporal::capi::DisplayOffset display_offset, temporal::capi::DisplayTimeZone display_timezone, temporal::capi::DisplayCalendar display_calendar, temporal::capi::ToStringRoundingOptions options, const temporal::capi::Provider* p, temporal::diplomat::capi::DiplomatWrite* write);

    temporal::capi::ZonedDateTime* temporal_ZonedDateTime_with_calendar(const temporal::capi::ZonedDateTime* self, temporal::capi::AnyCalendarKind calendar);

    typedef struct temporal_ZonedDateTime_with_plain_time_result {union {temporal::capi::ZonedDateTime* ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_ZonedDateTime_with_plain_time_result;
    temporal_ZonedDateTime_with_plain_time_result temporal_ZonedDateTime_with_plain_time(const temporal::capi::ZonedDateTime* self, const temporal::capi::PlainTime* time);

    typedef struct temporal_ZonedDateTime_with_plain_time_and_provider_result {union {temporal::capi::ZonedDateTime* ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_ZonedDateTime_with_plain_time_and_provider_result;
    temporal_ZonedDateTime_with_plain_time_and_provider_result temporal_ZonedDateTime_with_plain_time_and_provider(const temporal::capi::ZonedDateTime* self, const temporal::capi::PlainTime* time, const temporal::capi::Provider* p);

    typedef struct temporal_ZonedDateTime_add_result {union {temporal::capi::ZonedDateTime* ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_ZonedDateTime_add_result;
    temporal_ZonedDateTime_add_result temporal_ZonedDateTime_add(const temporal::capi::ZonedDateTime* self, const temporal::capi::Duration* duration, temporal::capi::ArithmeticOverflow_option overflow);

    typedef struct temporal_ZonedDateTime_add_with_provider_result {union {temporal::capi::ZonedDateTime* ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_ZonedDateTime_add_with_provider_result;
    temporal_ZonedDateTime_add_with_provider_result temporal_ZonedDateTime_add_with_provider(const temporal::capi::ZonedDateTime* self, const temporal::capi::Duration* duration, temporal::capi::ArithmeticOverflow_option overflow, const temporal::capi::Provider* p);

    typedef struct temporal_ZonedDateTime_subtract_result {union {temporal::capi::ZonedDateTime* ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_ZonedDateTime_subtract_result;
    temporal_ZonedDateTime_subtract_result temporal_ZonedDateTime_subtract(const temporal::capi::ZonedDateTime* self, const temporal::capi::Duration* duration, temporal::capi::ArithmeticOverflow_option overflow);

    typedef struct temporal_ZonedDateTime_subtract_with_provider_result {union {temporal::capi::ZonedDateTime* ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_ZonedDateTime_subtract_with_provider_result;
    temporal_ZonedDateTime_subtract_with_provider_result temporal_ZonedDateTime_subtract_with_provider(const temporal::capi::ZonedDateTime* self, const temporal::capi::Duration* duration, temporal::capi::ArithmeticOverflow_option overflow, const temporal::capi::Provider* p);

    typedef struct temporal_ZonedDateTime_until_result {union {temporal::capi::Duration* ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_ZonedDateTime_until_result;
    temporal_ZonedDateTime_until_result temporal_ZonedDateTime_until(const temporal::capi::ZonedDateTime* self, const temporal::capi::ZonedDateTime* other, temporal::capi::DifferenceSettings settings);

    typedef struct temporal_ZonedDateTime_until_with_provider_result {union {temporal::capi::Duration* ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_ZonedDateTime_until_with_provider_result;
    temporal_ZonedDateTime_until_with_provider_result temporal_ZonedDateTime_until_with_provider(const temporal::capi::ZonedDateTime* self, const temporal::capi::ZonedDateTime* other, temporal::capi::DifferenceSettings settings, const temporal::capi::Provider* p);

    typedef struct temporal_ZonedDateTime_since_result {union {temporal::capi::Duration* ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_ZonedDateTime_since_result;
    temporal_ZonedDateTime_since_result temporal_ZonedDateTime_since(const temporal::capi::ZonedDateTime* self, const temporal::capi::ZonedDateTime* other, temporal::capi::DifferenceSettings settings);

    typedef struct temporal_ZonedDateTime_since_with_provider_result {union {temporal::capi::Duration* ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_ZonedDateTime_since_with_provider_result;
    temporal_ZonedDateTime_since_with_provider_result temporal_ZonedDateTime_since_with_provider(const temporal::capi::ZonedDateTime* self, const temporal::capi::ZonedDateTime* other, temporal::capi::DifferenceSettings settings, const temporal::capi::Provider* p);

    typedef struct temporal_ZonedDateTime_round_result {union {temporal::capi::ZonedDateTime* ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_ZonedDateTime_round_result;
    temporal_ZonedDateTime_round_result temporal_ZonedDateTime_round(const temporal::capi::ZonedDateTime* self, temporal::capi::RoundingOptions options);

    typedef struct temporal_ZonedDateTime_round_with_provider_result {union {temporal::capi::ZonedDateTime* ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_ZonedDateTime_round_with_provider_result;
    temporal_ZonedDateTime_round_with_provider_result temporal_ZonedDateTime_round_with_provider(const temporal::capi::ZonedDateTime* self, temporal::capi::RoundingOptions options, const temporal::capi::Provider* p);

    uint8_t temporal_ZonedDateTime_hour(const temporal::capi::ZonedDateTime* self);

    uint8_t temporal_ZonedDateTime_minute(const temporal::capi::ZonedDateTime* self);

    uint8_t temporal_ZonedDateTime_second(const temporal::capi::ZonedDateTime* self);

    uint16_t temporal_ZonedDateTime_millisecond(const temporal::capi::ZonedDateTime* self);

    uint16_t temporal_ZonedDateTime_microsecond(const temporal::capi::ZonedDateTime* self);

    uint16_t temporal_ZonedDateTime_nanosecond(const temporal::capi::ZonedDateTime* self);

    const temporal::capi::Calendar* temporal_ZonedDateTime_calendar(const temporal::capi::ZonedDateTime* self);

    int32_t temporal_ZonedDateTime_year(const temporal::capi::ZonedDateTime* self);

    uint8_t temporal_ZonedDateTime_month(const temporal::capi::ZonedDateTime* self);

    void temporal_ZonedDateTime_month_code(const temporal::capi::ZonedDateTime* self, temporal::diplomat::capi::DiplomatWrite* write);

    uint8_t temporal_ZonedDateTime_day(const temporal::capi::ZonedDateTime* self);

    uint16_t temporal_ZonedDateTime_day_of_week(const temporal::capi::ZonedDateTime* self);

    uint16_t temporal_ZonedDateTime_day_of_year(const temporal::capi::ZonedDateTime* self);

    typedef struct temporal_ZonedDateTime_week_of_year_result {union {uint8_t ok; }; bool is_ok;} temporal_ZonedDateTime_week_of_year_result;
    temporal_ZonedDateTime_week_of_year_result temporal_ZonedDateTime_week_of_year(const temporal::capi::ZonedDateTime* self);

    typedef struct temporal_ZonedDateTime_year_of_week_result {union {int32_t ok; }; bool is_ok;} temporal_ZonedDateTime_year_of_week_result;
    temporal_ZonedDateTime_year_of_week_result temporal_ZonedDateTime_year_of_week(const temporal::capi::ZonedDateTime* self);

    uint16_t temporal_ZonedDateTime_days_in_week(const temporal::capi::ZonedDateTime* self);

    uint16_t temporal_ZonedDateTime_days_in_month(const temporal::capi::ZonedDateTime* self);

    uint16_t temporal_ZonedDateTime_days_in_year(const temporal::capi::ZonedDateTime* self);

    uint16_t temporal_ZonedDateTime_months_in_year(const temporal::capi::ZonedDateTime* self);

    bool temporal_ZonedDateTime_in_leap_year(const temporal::capi::ZonedDateTime* self);

    void temporal_ZonedDateTime_era(const temporal::capi::ZonedDateTime* self, temporal::diplomat::capi::DiplomatWrite* write);

    typedef struct temporal_ZonedDateTime_era_year_result {union {int32_t ok; }; bool is_ok;} temporal_ZonedDateTime_era_year_result;
    temporal_ZonedDateTime_era_year_result temporal_ZonedDateTime_era_year(const temporal::capi::ZonedDateTime* self);

    temporal::capi::ZonedDateTime* temporal_ZonedDateTime_clone(const temporal::capi::ZonedDateTime* self);

    void temporal_ZonedDateTime_destroy(ZonedDateTime* self);

    } // extern "C"
} // namespace capi
} // namespace

inline temporal::diplomat::result<std::unique_ptr<temporal::ZonedDateTime>, temporal::TemporalError> temporal::ZonedDateTime::try_new(temporal::I128Nanoseconds nanosecond, temporal::AnyCalendarKind calendar, temporal::TimeZone time_zone) {
    auto result = temporal::capi::temporal_ZonedDateTime_try_new(nanosecond.AsFFI(),
        calendar.AsFFI(),
        time_zone.AsFFI());
    return result.is_ok ? temporal::diplomat::result<std::unique_ptr<temporal::ZonedDateTime>, temporal::TemporalError>(temporal::diplomat::Ok<std::unique_ptr<temporal::ZonedDateTime>>(std::unique_ptr<temporal::ZonedDateTime>(temporal::ZonedDateTime::FromFFI(result.ok)))) : temporal::diplomat::result<std::unique_ptr<temporal::ZonedDateTime>, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline temporal::diplomat::result<std::unique_ptr<temporal::ZonedDateTime>, temporal::TemporalError> temporal::ZonedDateTime::try_new_with_provider(temporal::I128Nanoseconds nanosecond, temporal::AnyCalendarKind calendar, temporal::TimeZone time_zone, const temporal::Provider& p) {
    auto result = temporal::capi::temporal_ZonedDateTime_try_new_with_provider(nanosecond.AsFFI(),
        calendar.AsFFI(),
        time_zone.AsFFI(),
        p.AsFFI());
    return result.is_ok ? temporal::diplomat::result<std::unique_ptr<temporal::ZonedDateTime>, temporal::TemporalError>(temporal::diplomat::Ok<std::unique_ptr<temporal::ZonedDateTime>>(std::unique_ptr<temporal::ZonedDateTime>(temporal::ZonedDateTime::FromFFI(result.ok)))) : temporal::diplomat::result<std::unique_ptr<temporal::ZonedDateTime>, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline temporal::diplomat::result<std::unique_ptr<temporal::ZonedDateTime>, temporal::TemporalError> temporal::ZonedDateTime::from_partial(temporal::PartialZonedDateTime partial, std::optional<temporal::ArithmeticOverflow> overflow, std::optional<temporal::Disambiguation> disambiguation, std::optional<temporal::OffsetDisambiguation> offset_option) {
    auto result = temporal::capi::temporal_ZonedDateTime_from_partial(partial.AsFFI(),
        overflow.has_value() ? (temporal::capi::ArithmeticOverflow_option{ { overflow.value().AsFFI() }, true }) : (temporal::capi::ArithmeticOverflow_option{ {}, false }),
        disambiguation.has_value() ? (temporal::capi::Disambiguation_option{ { disambiguation.value().AsFFI() }, true }) : (temporal::capi::Disambiguation_option{ {}, false }),
        offset_option.has_value() ? (temporal::capi::OffsetDisambiguation_option{ { offset_option.value().AsFFI() }, true }) : (temporal::capi::OffsetDisambiguation_option{ {}, false }));
    return result.is_ok ? temporal::diplomat::result<std::unique_ptr<temporal::ZonedDateTime>, temporal::TemporalError>(temporal::diplomat::Ok<std::unique_ptr<temporal::ZonedDateTime>>(std::unique_ptr<temporal::ZonedDateTime>(temporal::ZonedDateTime::FromFFI(result.ok)))) : temporal::diplomat::result<std::unique_ptr<temporal::ZonedDateTime>, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline temporal::diplomat::result<std::unique_ptr<temporal::ZonedDateTime>, temporal::TemporalError> temporal::ZonedDateTime::from_partial_with_provider(temporal::PartialZonedDateTime partial, std::optional<temporal::ArithmeticOverflow> overflow, std::optional<temporal::Disambiguation> disambiguation, std::optional<temporal::OffsetDisambiguation> offset_option, const temporal::Provider& p) {
    auto result = temporal::capi::temporal_ZonedDateTime_from_partial_with_provider(partial.AsFFI(),
        overflow.has_value() ? (temporal::capi::ArithmeticOverflow_option{ { overflow.value().AsFFI() }, true }) : (temporal::capi::ArithmeticOverflow_option{ {}, false }),
        disambiguation.has_value() ? (temporal::capi::Disambiguation_option{ { disambiguation.value().AsFFI() }, true }) : (temporal::capi::Disambiguation_option{ {}, false }),
        offset_option.has_value() ? (temporal::capi::OffsetDisambiguation_option{ { offset_option.value().AsFFI() }, true }) : (temporal::capi::OffsetDisambiguation_option{ {}, false }),
        p.AsFFI());
    return result.is_ok ? temporal::diplomat::result<std::unique_ptr<temporal::ZonedDateTime>, temporal::TemporalError>(temporal::diplomat::Ok<std::unique_ptr<temporal::ZonedDateTime>>(std::unique_ptr<temporal::ZonedDateTime>(temporal::ZonedDateTime::FromFFI(result.ok)))) : temporal::diplomat::result<std::unique_ptr<temporal::ZonedDateTime>, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline temporal::diplomat::result<std::unique_ptr<temporal::ZonedDateTime>, temporal::TemporalError> temporal::ZonedDateTime::from_parsed(const temporal::ParsedZonedDateTime& parsed, temporal::Disambiguation disambiguation, temporal::OffsetDisambiguation offset_option) {
    auto result = temporal::capi::temporal_ZonedDateTime_from_parsed(parsed.AsFFI(),
        disambiguation.AsFFI(),
        offset_option.AsFFI());
    return result.is_ok ? temporal::diplomat::result<std::unique_ptr<temporal::ZonedDateTime>, temporal::TemporalError>(temporal::diplomat::Ok<std::unique_ptr<temporal::ZonedDateTime>>(std::unique_ptr<temporal::ZonedDateTime>(temporal::ZonedDateTime::FromFFI(result.ok)))) : temporal::diplomat::result<std::unique_ptr<temporal::ZonedDateTime>, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline temporal::diplomat::result<std::unique_ptr<temporal::ZonedDateTime>, temporal::TemporalError> temporal::ZonedDateTime::from_parsed_with_provider(const temporal::ParsedZonedDateTime& parsed, temporal::Disambiguation disambiguation, temporal::OffsetDisambiguation offset_option, const temporal::Provider& p) {
    auto result = temporal::capi::temporal_ZonedDateTime_from_parsed_with_provider(parsed.AsFFI(),
        disambiguation.AsFFI(),
        offset_option.AsFFI(),
        p.AsFFI());
    return result.is_ok ? temporal::diplomat::result<std::unique_ptr<temporal::ZonedDateTime>, temporal::TemporalError>(temporal::diplomat::Ok<std::unique_ptr<temporal::ZonedDateTime>>(std::unique_ptr<temporal::ZonedDateTime>(temporal::ZonedDateTime::FromFFI(result.ok)))) : temporal::diplomat::result<std::unique_ptr<temporal::ZonedDateTime>, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline temporal::diplomat::result<std::unique_ptr<temporal::ZonedDateTime>, temporal::TemporalError> temporal::ZonedDateTime::from_utf8(std::string_view s, temporal::Disambiguation disambiguation, temporal::OffsetDisambiguation offset_disambiguation) {
    auto result = temporal::capi::temporal_ZonedDateTime_from_utf8({s.data(), s.size()},
        disambiguation.AsFFI(),
        offset_disambiguation.AsFFI());
    return result.is_ok ? temporal::diplomat::result<std::unique_ptr<temporal::ZonedDateTime>, temporal::TemporalError>(temporal::diplomat::Ok<std::unique_ptr<temporal::ZonedDateTime>>(std::unique_ptr<temporal::ZonedDateTime>(temporal::ZonedDateTime::FromFFI(result.ok)))) : temporal::diplomat::result<std::unique_ptr<temporal::ZonedDateTime>, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline temporal::diplomat::result<std::unique_ptr<temporal::ZonedDateTime>, temporal::TemporalError> temporal::ZonedDateTime::from_utf8_with_provider(std::string_view s, temporal::Disambiguation disambiguation, temporal::OffsetDisambiguation offset_disambiguation, const temporal::Provider& p) {
    auto result = temporal::capi::temporal_ZonedDateTime_from_utf8_with_provider({s.data(), s.size()},
        disambiguation.AsFFI(),
        offset_disambiguation.AsFFI(),
        p.AsFFI());
    return result.is_ok ? temporal::diplomat::result<std::unique_ptr<temporal::ZonedDateTime>, temporal::TemporalError>(temporal::diplomat::Ok<std::unique_ptr<temporal::ZonedDateTime>>(std::unique_ptr<temporal::ZonedDateTime>(temporal::ZonedDateTime::FromFFI(result.ok)))) : temporal::diplomat::result<std::unique_ptr<temporal::ZonedDateTime>, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline temporal::diplomat::result<std::unique_ptr<temporal::ZonedDateTime>, temporal::TemporalError> temporal::ZonedDateTime::from_utf16(std::u16string_view s, temporal::Disambiguation disambiguation, temporal::OffsetDisambiguation offset_disambiguation) {
    auto result = temporal::capi::temporal_ZonedDateTime_from_utf16({s.data(), s.size()},
        disambiguation.AsFFI(),
        offset_disambiguation.AsFFI());
    return result.is_ok ? temporal::diplomat::result<std::unique_ptr<temporal::ZonedDateTime>, temporal::TemporalError>(temporal::diplomat::Ok<std::unique_ptr<temporal::ZonedDateTime>>(std::unique_ptr<temporal::ZonedDateTime>(temporal::ZonedDateTime::FromFFI(result.ok)))) : temporal::diplomat::result<std::unique_ptr<temporal::ZonedDateTime>, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline temporal::diplomat::result<std::unique_ptr<temporal::ZonedDateTime>, temporal::TemporalError> temporal::ZonedDateTime::from_utf16_with_provider(std::u16string_view s, temporal::Disambiguation disambiguation, temporal::OffsetDisambiguation offset_disambiguation, const temporal::Provider& p) {
    auto result = temporal::capi::temporal_ZonedDateTime_from_utf16_with_provider({s.data(), s.size()},
        disambiguation.AsFFI(),
        offset_disambiguation.AsFFI(),
        p.AsFFI());
    return result.is_ok ? temporal::diplomat::result<std::unique_ptr<temporal::ZonedDateTime>, temporal::TemporalError>(temporal::diplomat::Ok<std::unique_ptr<temporal::ZonedDateTime>>(std::unique_ptr<temporal::ZonedDateTime>(temporal::ZonedDateTime::FromFFI(result.ok)))) : temporal::diplomat::result<std::unique_ptr<temporal::ZonedDateTime>, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline int64_t temporal::ZonedDateTime::epoch_milliseconds() const {
    auto result = temporal::capi::temporal_ZonedDateTime_epoch_milliseconds(this->AsFFI());
    return result;
}

inline temporal::diplomat::result<std::unique_ptr<temporal::ZonedDateTime>, temporal::TemporalError> temporal::ZonedDateTime::from_epoch_milliseconds(int64_t ms, temporal::TimeZone tz) {
    auto result = temporal::capi::temporal_ZonedDateTime_from_epoch_milliseconds(ms,
        tz.AsFFI());
    return result.is_ok ? temporal::diplomat::result<std::unique_ptr<temporal::ZonedDateTime>, temporal::TemporalError>(temporal::diplomat::Ok<std::unique_ptr<temporal::ZonedDateTime>>(std::unique_ptr<temporal::ZonedDateTime>(temporal::ZonedDateTime::FromFFI(result.ok)))) : temporal::diplomat::result<std::unique_ptr<temporal::ZonedDateTime>, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline temporal::diplomat::result<std::unique_ptr<temporal::ZonedDateTime>, temporal::TemporalError> temporal::ZonedDateTime::from_epoch_milliseconds_with_provider(int64_t ms, temporal::TimeZone tz, const temporal::Provider& p) {
    auto result = temporal::capi::temporal_ZonedDateTime_from_epoch_milliseconds_with_provider(ms,
        tz.AsFFI(),
        p.AsFFI());
    return result.is_ok ? temporal::diplomat::result<std::unique_ptr<temporal::ZonedDateTime>, temporal::TemporalError>(temporal::diplomat::Ok<std::unique_ptr<temporal::ZonedDateTime>>(std::unique_ptr<temporal::ZonedDateTime>(temporal::ZonedDateTime::FromFFI(result.ok)))) : temporal::diplomat::result<std::unique_ptr<temporal::ZonedDateTime>, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline temporal::diplomat::result<std::unique_ptr<temporal::ZonedDateTime>, temporal::TemporalError> temporal::ZonedDateTime::from_epoch_nanoseconds(temporal::I128Nanoseconds ns, temporal::TimeZone tz) {
    auto result = temporal::capi::temporal_ZonedDateTime_from_epoch_nanoseconds(ns.AsFFI(),
        tz.AsFFI());
    return result.is_ok ? temporal::diplomat::result<std::unique_ptr<temporal::ZonedDateTime>, temporal::TemporalError>(temporal::diplomat::Ok<std::unique_ptr<temporal::ZonedDateTime>>(std::unique_ptr<temporal::ZonedDateTime>(temporal::ZonedDateTime::FromFFI(result.ok)))) : temporal::diplomat::result<std::unique_ptr<temporal::ZonedDateTime>, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline temporal::diplomat::result<std::unique_ptr<temporal::ZonedDateTime>, temporal::TemporalError> temporal::ZonedDateTime::from_epoch_nanoseconds_with_provider(temporal::I128Nanoseconds ns, temporal::TimeZone tz, const temporal::Provider& p) {
    auto result = temporal::capi::temporal_ZonedDateTime_from_epoch_nanoseconds_with_provider(ns.AsFFI(),
        tz.AsFFI(),
        p.AsFFI());
    return result.is_ok ? temporal::diplomat::result<std::unique_ptr<temporal::ZonedDateTime>, temporal::TemporalError>(temporal::diplomat::Ok<std::unique_ptr<temporal::ZonedDateTime>>(std::unique_ptr<temporal::ZonedDateTime>(temporal::ZonedDateTime::FromFFI(result.ok)))) : temporal::diplomat::result<std::unique_ptr<temporal::ZonedDateTime>, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline temporal::I128Nanoseconds temporal::ZonedDateTime::epoch_nanoseconds() const {
    auto result = temporal::capi::temporal_ZonedDateTime_epoch_nanoseconds(this->AsFFI());
    return temporal::I128Nanoseconds::FromFFI(result);
}

inline int64_t temporal::ZonedDateTime::offset_nanoseconds() const {
    auto result = temporal::capi::temporal_ZonedDateTime_offset_nanoseconds(this->AsFFI());
    return result;
}

inline std::unique_ptr<temporal::Instant> temporal::ZonedDateTime::to_instant() const {
    auto result = temporal::capi::temporal_ZonedDateTime_to_instant(this->AsFFI());
    return std::unique_ptr<temporal::Instant>(temporal::Instant::FromFFI(result));
}

inline temporal::diplomat::result<std::unique_ptr<temporal::ZonedDateTime>, temporal::TemporalError> temporal::ZonedDateTime::with(temporal::PartialZonedDateTime partial, std::optional<temporal::Disambiguation> disambiguation, std::optional<temporal::OffsetDisambiguation> offset_option, std::optional<temporal::ArithmeticOverflow> overflow) const {
    auto result = temporal::capi::temporal_ZonedDateTime_with(this->AsFFI(),
        partial.AsFFI(),
        disambiguation.has_value() ? (temporal::capi::Disambiguation_option{ { disambiguation.value().AsFFI() }, true }) : (temporal::capi::Disambiguation_option{ {}, false }),
        offset_option.has_value() ? (temporal::capi::OffsetDisambiguation_option{ { offset_option.value().AsFFI() }, true }) : (temporal::capi::OffsetDisambiguation_option{ {}, false }),
        overflow.has_value() ? (temporal::capi::ArithmeticOverflow_option{ { overflow.value().AsFFI() }, true }) : (temporal::capi::ArithmeticOverflow_option{ {}, false }));
    return result.is_ok ? temporal::diplomat::result<std::unique_ptr<temporal::ZonedDateTime>, temporal::TemporalError>(temporal::diplomat::Ok<std::unique_ptr<temporal::ZonedDateTime>>(std::unique_ptr<temporal::ZonedDateTime>(temporal::ZonedDateTime::FromFFI(result.ok)))) : temporal::diplomat::result<std::unique_ptr<temporal::ZonedDateTime>, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline temporal::diplomat::result<std::unique_ptr<temporal::ZonedDateTime>, temporal::TemporalError> temporal::ZonedDateTime::with_with_provider(temporal::PartialZonedDateTime partial, std::optional<temporal::Disambiguation> disambiguation, std::optional<temporal::OffsetDisambiguation> offset_option, std::optional<temporal::ArithmeticOverflow> overflow, const temporal::Provider& p) const {
    auto result = temporal::capi::temporal_ZonedDateTime_with_with_provider(this->AsFFI(),
        partial.AsFFI(),
        disambiguation.has_value() ? (temporal::capi::Disambiguation_option{ { disambiguation.value().AsFFI() }, true }) : (temporal::capi::Disambiguation_option{ {}, false }),
        offset_option.has_value() ? (temporal::capi::OffsetDisambiguation_option{ { offset_option.value().AsFFI() }, true }) : (temporal::capi::OffsetDisambiguation_option{ {}, false }),
        overflow.has_value() ? (temporal::capi::ArithmeticOverflow_option{ { overflow.value().AsFFI() }, true }) : (temporal::capi::ArithmeticOverflow_option{ {}, false }),
        p.AsFFI());
    return result.is_ok ? temporal::diplomat::result<std::unique_ptr<temporal::ZonedDateTime>, temporal::TemporalError>(temporal::diplomat::Ok<std::unique_ptr<temporal::ZonedDateTime>>(std::unique_ptr<temporal::ZonedDateTime>(temporal::ZonedDateTime::FromFFI(result.ok)))) : temporal::diplomat::result<std::unique_ptr<temporal::ZonedDateTime>, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline temporal::diplomat::result<std::unique_ptr<temporal::ZonedDateTime>, temporal::TemporalError> temporal::ZonedDateTime::with_timezone(temporal::TimeZone zone) const {
    auto result = temporal::capi::temporal_ZonedDateTime_with_timezone(this->AsFFI(),
        zone.AsFFI());
    return result.is_ok ? temporal::diplomat::result<std::unique_ptr<temporal::ZonedDateTime>, temporal::TemporalError>(temporal::diplomat::Ok<std::unique_ptr<temporal::ZonedDateTime>>(std::unique_ptr<temporal::ZonedDateTime>(temporal::ZonedDateTime::FromFFI(result.ok)))) : temporal::diplomat::result<std::unique_ptr<temporal::ZonedDateTime>, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline temporal::diplomat::result<std::unique_ptr<temporal::ZonedDateTime>, temporal::TemporalError> temporal::ZonedDateTime::with_timezone_with_provider(temporal::TimeZone zone, const temporal::Provider& p) const {
    auto result = temporal::capi::temporal_ZonedDateTime_with_timezone_with_provider(this->AsFFI(),
        zone.AsFFI(),
        p.AsFFI());
    return result.is_ok ? temporal::diplomat::result<std::unique_ptr<temporal::ZonedDateTime>, temporal::TemporalError>(temporal::diplomat::Ok<std::unique_ptr<temporal::ZonedDateTime>>(std::unique_ptr<temporal::ZonedDateTime>(temporal::ZonedDateTime::FromFFI(result.ok)))) : temporal::diplomat::result<std::unique_ptr<temporal::ZonedDateTime>, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline temporal::TimeZone temporal::ZonedDateTime::timezone() const {
    auto result = temporal::capi::temporal_ZonedDateTime_timezone(this->AsFFI());
    return temporal::TimeZone::FromFFI(result);
}

inline int8_t temporal::ZonedDateTime::compare_instant(const temporal::ZonedDateTime& other) const {
    auto result = temporal::capi::temporal_ZonedDateTime_compare_instant(this->AsFFI(),
        other.AsFFI());
    return result;
}

inline bool temporal::ZonedDateTime::equals(const temporal::ZonedDateTime& other) const {
    auto result = temporal::capi::temporal_ZonedDateTime_equals(this->AsFFI(),
        other.AsFFI());
    return result;
}

inline temporal::diplomat::result<bool, temporal::TemporalError> temporal::ZonedDateTime::equals_with_provider(const temporal::ZonedDateTime& other, const temporal::Provider& p) const {
    auto result = temporal::capi::temporal_ZonedDateTime_equals_with_provider(this->AsFFI(),
        other.AsFFI(),
        p.AsFFI());
    return result.is_ok ? temporal::diplomat::result<bool, temporal::TemporalError>(temporal::diplomat::Ok<bool>(result.ok)) : temporal::diplomat::result<bool, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline temporal::diplomat::result<std::string, temporal::TemporalError> temporal::ZonedDateTime::offset() const {
    std::string output;
    temporal::diplomat::capi::DiplomatWrite write = temporal::diplomat::WriteFromString(output);
    auto result = temporal::capi::temporal_ZonedDateTime_offset(this->AsFFI(),
        &write);
    return result.is_ok ? temporal::diplomat::result<std::string, temporal::TemporalError>(temporal::diplomat::Ok<std::string>(std::move(output))) : temporal::diplomat::result<std::string, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}
template<typename W>
inline temporal::diplomat::result<std::monostate, temporal::TemporalError> temporal::ZonedDateTime::offset_write(W& writeable) const {
    temporal::diplomat::capi::DiplomatWrite write = temporal::diplomat::WriteTrait<W>::Construct(writeable);
    auto result = temporal::capi::temporal_ZonedDateTime_offset(this->AsFFI(),
        &write);
    return result.is_ok ? temporal::diplomat::result<std::monostate, temporal::TemporalError>(temporal::diplomat::Ok<std::monostate>()) : temporal::diplomat::result<std::monostate, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline temporal::diplomat::result<std::unique_ptr<temporal::ZonedDateTime>, temporal::TemporalError> temporal::ZonedDateTime::start_of_day() const {
    auto result = temporal::capi::temporal_ZonedDateTime_start_of_day(this->AsFFI());
    return result.is_ok ? temporal::diplomat::result<std::unique_ptr<temporal::ZonedDateTime>, temporal::TemporalError>(temporal::diplomat::Ok<std::unique_ptr<temporal::ZonedDateTime>>(std::unique_ptr<temporal::ZonedDateTime>(temporal::ZonedDateTime::FromFFI(result.ok)))) : temporal::diplomat::result<std::unique_ptr<temporal::ZonedDateTime>, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline temporal::diplomat::result<std::unique_ptr<temporal::ZonedDateTime>, temporal::TemporalError> temporal::ZonedDateTime::start_of_day_with_provider(const temporal::Provider& p) const {
    auto result = temporal::capi::temporal_ZonedDateTime_start_of_day_with_provider(this->AsFFI(),
        p.AsFFI());
    return result.is_ok ? temporal::diplomat::result<std::unique_ptr<temporal::ZonedDateTime>, temporal::TemporalError>(temporal::diplomat::Ok<std::unique_ptr<temporal::ZonedDateTime>>(std::unique_ptr<temporal::ZonedDateTime>(temporal::ZonedDateTime::FromFFI(result.ok)))) : temporal::diplomat::result<std::unique_ptr<temporal::ZonedDateTime>, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline temporal::diplomat::result<std::unique_ptr<temporal::ZonedDateTime>, temporal::TemporalError> temporal::ZonedDateTime::get_time_zone_transition(temporal::TransitionDirection direction) const {
    auto result = temporal::capi::temporal_ZonedDateTime_get_time_zone_transition(this->AsFFI(),
        direction.AsFFI());
    return result.is_ok ? temporal::diplomat::result<std::unique_ptr<temporal::ZonedDateTime>, temporal::TemporalError>(temporal::diplomat::Ok<std::unique_ptr<temporal::ZonedDateTime>>(std::unique_ptr<temporal::ZonedDateTime>(temporal::ZonedDateTime::FromFFI(result.ok)))) : temporal::diplomat::result<std::unique_ptr<temporal::ZonedDateTime>, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline temporal::diplomat::result<std::unique_ptr<temporal::ZonedDateTime>, temporal::TemporalError> temporal::ZonedDateTime::get_time_zone_transition_with_provider(temporal::TransitionDirection direction, const temporal::Provider& p) const {
    auto result = temporal::capi::temporal_ZonedDateTime_get_time_zone_transition_with_provider(this->AsFFI(),
        direction.AsFFI(),
        p.AsFFI());
    return result.is_ok ? temporal::diplomat::result<std::unique_ptr<temporal::ZonedDateTime>, temporal::TemporalError>(temporal::diplomat::Ok<std::unique_ptr<temporal::ZonedDateTime>>(std::unique_ptr<temporal::ZonedDateTime>(temporal::ZonedDateTime::FromFFI(result.ok)))) : temporal::diplomat::result<std::unique_ptr<temporal::ZonedDateTime>, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline temporal::diplomat::result<double, temporal::TemporalError> temporal::ZonedDateTime::hours_in_day() const {
    auto result = temporal::capi::temporal_ZonedDateTime_hours_in_day(this->AsFFI());
    return result.is_ok ? temporal::diplomat::result<double, temporal::TemporalError>(temporal::diplomat::Ok<double>(result.ok)) : temporal::diplomat::result<double, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline temporal::diplomat::result<double, temporal::TemporalError> temporal::ZonedDateTime::hours_in_day_with_provider(const temporal::Provider& p) const {
    auto result = temporal::capi::temporal_ZonedDateTime_hours_in_day_with_provider(this->AsFFI(),
        p.AsFFI());
    return result.is_ok ? temporal::diplomat::result<double, temporal::TemporalError>(temporal::diplomat::Ok<double>(result.ok)) : temporal::diplomat::result<double, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline std::unique_ptr<temporal::PlainDateTime> temporal::ZonedDateTime::to_plain_datetime() const {
    auto result = temporal::capi::temporal_ZonedDateTime_to_plain_datetime(this->AsFFI());
    return std::unique_ptr<temporal::PlainDateTime>(temporal::PlainDateTime::FromFFI(result));
}

inline std::unique_ptr<temporal::PlainDate> temporal::ZonedDateTime::to_plain_date() const {
    auto result = temporal::capi::temporal_ZonedDateTime_to_plain_date(this->AsFFI());
    return std::unique_ptr<temporal::PlainDate>(temporal::PlainDate::FromFFI(result));
}

inline std::unique_ptr<temporal::PlainTime> temporal::ZonedDateTime::to_plain_time() const {
    auto result = temporal::capi::temporal_ZonedDateTime_to_plain_time(this->AsFFI());
    return std::unique_ptr<temporal::PlainTime>(temporal::PlainTime::FromFFI(result));
}

inline temporal::diplomat::result<std::string, temporal::TemporalError> temporal::ZonedDateTime::to_ixdtf_string(temporal::DisplayOffset display_offset, temporal::DisplayTimeZone display_timezone, temporal::DisplayCalendar display_calendar, temporal::ToStringRoundingOptions options) const {
    std::string output;
    temporal::diplomat::capi::DiplomatWrite write = temporal::diplomat::WriteFromString(output);
    auto result = temporal::capi::temporal_ZonedDateTime_to_ixdtf_string(this->AsFFI(),
        display_offset.AsFFI(),
        display_timezone.AsFFI(),
        display_calendar.AsFFI(),
        options.AsFFI(),
        &write);
    return result.is_ok ? temporal::diplomat::result<std::string, temporal::TemporalError>(temporal::diplomat::Ok<std::string>(std::move(output))) : temporal::diplomat::result<std::string, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}
template<typename W>
inline temporal::diplomat::result<std::monostate, temporal::TemporalError> temporal::ZonedDateTime::to_ixdtf_string_write(temporal::DisplayOffset display_offset, temporal::DisplayTimeZone display_timezone, temporal::DisplayCalendar display_calendar, temporal::ToStringRoundingOptions options, W& writeable) const {
    temporal::diplomat::capi::DiplomatWrite write = temporal::diplomat::WriteTrait<W>::Construct(writeable);
    auto result = temporal::capi::temporal_ZonedDateTime_to_ixdtf_string(this->AsFFI(),
        display_offset.AsFFI(),
        display_timezone.AsFFI(),
        display_calendar.AsFFI(),
        options.AsFFI(),
        &write);
    return result.is_ok ? temporal::diplomat::result<std::monostate, temporal::TemporalError>(temporal::diplomat::Ok<std::monostate>()) : temporal::diplomat::result<std::monostate, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline temporal::diplomat::result<std::string, temporal::TemporalError> temporal::ZonedDateTime::to_ixdtf_string_with_provider(temporal::DisplayOffset display_offset, temporal::DisplayTimeZone display_timezone, temporal::DisplayCalendar display_calendar, temporal::ToStringRoundingOptions options, const temporal::Provider& p) const {
    std::string output;
    temporal::diplomat::capi::DiplomatWrite write = temporal::diplomat::WriteFromString(output);
    auto result = temporal::capi::temporal_ZonedDateTime_to_ixdtf_string_with_provider(this->AsFFI(),
        display_offset.AsFFI(),
        display_timezone.AsFFI(),
        display_calendar.AsFFI(),
        options.AsFFI(),
        p.AsFFI(),
        &write);
    return result.is_ok ? temporal::diplomat::result<std::string, temporal::TemporalError>(temporal::diplomat::Ok<std::string>(std::move(output))) : temporal::diplomat::result<std::string, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}
template<typename W>
inline temporal::diplomat::result<std::monostate, temporal::TemporalError> temporal::ZonedDateTime::to_ixdtf_string_with_provider_write(temporal::DisplayOffset display_offset, temporal::DisplayTimeZone display_timezone, temporal::DisplayCalendar display_calendar, temporal::ToStringRoundingOptions options, const temporal::Provider& p, W& writeable) const {
    temporal::diplomat::capi::DiplomatWrite write = temporal::diplomat::WriteTrait<W>::Construct(writeable);
    auto result = temporal::capi::temporal_ZonedDateTime_to_ixdtf_string_with_provider(this->AsFFI(),
        display_offset.AsFFI(),
        display_timezone.AsFFI(),
        display_calendar.AsFFI(),
        options.AsFFI(),
        p.AsFFI(),
        &write);
    return result.is_ok ? temporal::diplomat::result<std::monostate, temporal::TemporalError>(temporal::diplomat::Ok<std::monostate>()) : temporal::diplomat::result<std::monostate, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline std::unique_ptr<temporal::ZonedDateTime> temporal::ZonedDateTime::with_calendar(temporal::AnyCalendarKind calendar) const {
    auto result = temporal::capi::temporal_ZonedDateTime_with_calendar(this->AsFFI(),
        calendar.AsFFI());
    return std::unique_ptr<temporal::ZonedDateTime>(temporal::ZonedDateTime::FromFFI(result));
}

inline temporal::diplomat::result<std::unique_ptr<temporal::ZonedDateTime>, temporal::TemporalError> temporal::ZonedDateTime::with_plain_time(const temporal::PlainTime* time) const {
    auto result = temporal::capi::temporal_ZonedDateTime_with_plain_time(this->AsFFI(),
        time ? time->AsFFI() : nullptr);
    return result.is_ok ? temporal::diplomat::result<std::unique_ptr<temporal::ZonedDateTime>, temporal::TemporalError>(temporal::diplomat::Ok<std::unique_ptr<temporal::ZonedDateTime>>(std::unique_ptr<temporal::ZonedDateTime>(temporal::ZonedDateTime::FromFFI(result.ok)))) : temporal::diplomat::result<std::unique_ptr<temporal::ZonedDateTime>, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline temporal::diplomat::result<std::unique_ptr<temporal::ZonedDateTime>, temporal::TemporalError> temporal::ZonedDateTime::with_plain_time_and_provider(const temporal::PlainTime* time, const temporal::Provider& p) const {
    auto result = temporal::capi::temporal_ZonedDateTime_with_plain_time_and_provider(this->AsFFI(),
        time ? time->AsFFI() : nullptr,
        p.AsFFI());
    return result.is_ok ? temporal::diplomat::result<std::unique_ptr<temporal::ZonedDateTime>, temporal::TemporalError>(temporal::diplomat::Ok<std::unique_ptr<temporal::ZonedDateTime>>(std::unique_ptr<temporal::ZonedDateTime>(temporal::ZonedDateTime::FromFFI(result.ok)))) : temporal::diplomat::result<std::unique_ptr<temporal::ZonedDateTime>, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline temporal::diplomat::result<std::unique_ptr<temporal::ZonedDateTime>, temporal::TemporalError> temporal::ZonedDateTime::add(const temporal::Duration& duration, std::optional<temporal::ArithmeticOverflow> overflow) const {
    auto result = temporal::capi::temporal_ZonedDateTime_add(this->AsFFI(),
        duration.AsFFI(),
        overflow.has_value() ? (temporal::capi::ArithmeticOverflow_option{ { overflow.value().AsFFI() }, true }) : (temporal::capi::ArithmeticOverflow_option{ {}, false }));
    return result.is_ok ? temporal::diplomat::result<std::unique_ptr<temporal::ZonedDateTime>, temporal::TemporalError>(temporal::diplomat::Ok<std::unique_ptr<temporal::ZonedDateTime>>(std::unique_ptr<temporal::ZonedDateTime>(temporal::ZonedDateTime::FromFFI(result.ok)))) : temporal::diplomat::result<std::unique_ptr<temporal::ZonedDateTime>, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline temporal::diplomat::result<std::unique_ptr<temporal::ZonedDateTime>, temporal::TemporalError> temporal::ZonedDateTime::add_with_provider(const temporal::Duration& duration, std::optional<temporal::ArithmeticOverflow> overflow, const temporal::Provider& p) const {
    auto result = temporal::capi::temporal_ZonedDateTime_add_with_provider(this->AsFFI(),
        duration.AsFFI(),
        overflow.has_value() ? (temporal::capi::ArithmeticOverflow_option{ { overflow.value().AsFFI() }, true }) : (temporal::capi::ArithmeticOverflow_option{ {}, false }),
        p.AsFFI());
    return result.is_ok ? temporal::diplomat::result<std::unique_ptr<temporal::ZonedDateTime>, temporal::TemporalError>(temporal::diplomat::Ok<std::unique_ptr<temporal::ZonedDateTime>>(std::unique_ptr<temporal::ZonedDateTime>(temporal::ZonedDateTime::FromFFI(result.ok)))) : temporal::diplomat::result<std::unique_ptr<temporal::ZonedDateTime>, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline temporal::diplomat::result<std::unique_ptr<temporal::ZonedDateTime>, temporal::TemporalError> temporal::ZonedDateTime::subtract(const temporal::Duration& duration, std::optional<temporal::ArithmeticOverflow> overflow) const {
    auto result = temporal::capi::temporal_ZonedDateTime_subtract(this->AsFFI(),
        duration.AsFFI(),
        overflow.has_value() ? (temporal::capi::ArithmeticOverflow_option{ { overflow.value().AsFFI() }, true }) : (temporal::capi::ArithmeticOverflow_option{ {}, false }));
    return result.is_ok ? temporal::diplomat::result<std::unique_ptr<temporal::ZonedDateTime>, temporal::TemporalError>(temporal::diplomat::Ok<std::unique_ptr<temporal::ZonedDateTime>>(std::unique_ptr<temporal::ZonedDateTime>(temporal::ZonedDateTime::FromFFI(result.ok)))) : temporal::diplomat::result<std::unique_ptr<temporal::ZonedDateTime>, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline temporal::diplomat::result<std::unique_ptr<temporal::ZonedDateTime>, temporal::TemporalError> temporal::ZonedDateTime::subtract_with_provider(const temporal::Duration& duration, std::optional<temporal::ArithmeticOverflow> overflow, const temporal::Provider& p) const {
    auto result = temporal::capi::temporal_ZonedDateTime_subtract_with_provider(this->AsFFI(),
        duration.AsFFI(),
        overflow.has_value() ? (temporal::capi::ArithmeticOverflow_option{ { overflow.value().AsFFI() }, true }) : (temporal::capi::ArithmeticOverflow_option{ {}, false }),
        p.AsFFI());
    return result.is_ok ? temporal::diplomat::result<std::unique_ptr<temporal::ZonedDateTime>, temporal::TemporalError>(temporal::diplomat::Ok<std::unique_ptr<temporal::ZonedDateTime>>(std::unique_ptr<temporal::ZonedDateTime>(temporal::ZonedDateTime::FromFFI(result.ok)))) : temporal::diplomat::result<std::unique_ptr<temporal::ZonedDateTime>, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline temporal::diplomat::result<std::unique_ptr<temporal::Duration>, temporal::TemporalError> temporal::ZonedDateTime::until(const temporal::ZonedDateTime& other, temporal::DifferenceSettings settings) const {
    auto result = temporal::capi::temporal_ZonedDateTime_until(this->AsFFI(),
        other.AsFFI(),
        settings.AsFFI());
    return result.is_ok ? temporal::diplomat::result<std::unique_ptr<temporal::Duration>, temporal::TemporalError>(temporal::diplomat::Ok<std::unique_ptr<temporal::Duration>>(std::unique_ptr<temporal::Duration>(temporal::Duration::FromFFI(result.ok)))) : temporal::diplomat::result<std::unique_ptr<temporal::Duration>, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline temporal::diplomat::result<std::unique_ptr<temporal::Duration>, temporal::TemporalError> temporal::ZonedDateTime::until_with_provider(const temporal::ZonedDateTime& other, temporal::DifferenceSettings settings, const temporal::Provider& p) const {
    auto result = temporal::capi::temporal_ZonedDateTime_until_with_provider(this->AsFFI(),
        other.AsFFI(),
        settings.AsFFI(),
        p.AsFFI());
    return result.is_ok ? temporal::diplomat::result<std::unique_ptr<temporal::Duration>, temporal::TemporalError>(temporal::diplomat::Ok<std::unique_ptr<temporal::Duration>>(std::unique_ptr<temporal::Duration>(temporal::Duration::FromFFI(result.ok)))) : temporal::diplomat::result<std::unique_ptr<temporal::Duration>, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline temporal::diplomat::result<std::unique_ptr<temporal::Duration>, temporal::TemporalError> temporal::ZonedDateTime::since(const temporal::ZonedDateTime& other, temporal::DifferenceSettings settings) const {
    auto result = temporal::capi::temporal_ZonedDateTime_since(this->AsFFI(),
        other.AsFFI(),
        settings.AsFFI());
    return result.is_ok ? temporal::diplomat::result<std::unique_ptr<temporal::Duration>, temporal::TemporalError>(temporal::diplomat::Ok<std::unique_ptr<temporal::Duration>>(std::unique_ptr<temporal::Duration>(temporal::Duration::FromFFI(result.ok)))) : temporal::diplomat::result<std::unique_ptr<temporal::Duration>, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline temporal::diplomat::result<std::unique_ptr<temporal::Duration>, temporal::TemporalError> temporal::ZonedDateTime::since_with_provider(const temporal::ZonedDateTime& other, temporal::DifferenceSettings settings, const temporal::Provider& p) const {
    auto result = temporal::capi::temporal_ZonedDateTime_since_with_provider(this->AsFFI(),
        other.AsFFI(),
        settings.AsFFI(),
        p.AsFFI());
    return result.is_ok ? temporal::diplomat::result<std::unique_ptr<temporal::Duration>, temporal::TemporalError>(temporal::diplomat::Ok<std::unique_ptr<temporal::Duration>>(std::unique_ptr<temporal::Duration>(temporal::Duration::FromFFI(result.ok)))) : temporal::diplomat::result<std::unique_ptr<temporal::Duration>, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline temporal::diplomat::result<std::unique_ptr<temporal::ZonedDateTime>, temporal::TemporalError> temporal::ZonedDateTime::round(temporal::RoundingOptions options) const {
    auto result = temporal::capi::temporal_ZonedDateTime_round(this->AsFFI(),
        options.AsFFI());
    return result.is_ok ? temporal::diplomat::result<std::unique_ptr<temporal::ZonedDateTime>, temporal::TemporalError>(temporal::diplomat::Ok<std::unique_ptr<temporal::ZonedDateTime>>(std::unique_ptr<temporal::ZonedDateTime>(temporal::ZonedDateTime::FromFFI(result.ok)))) : temporal::diplomat::result<std::unique_ptr<temporal::ZonedDateTime>, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline temporal::diplomat::result<std::unique_ptr<temporal::ZonedDateTime>, temporal::TemporalError> temporal::ZonedDateTime::round_with_provider(temporal::RoundingOptions options, const temporal::Provider& p) const {
    auto result = temporal::capi::temporal_ZonedDateTime_round_with_provider(this->AsFFI(),
        options.AsFFI(),
        p.AsFFI());
    return result.is_ok ? temporal::diplomat::result<std::unique_ptr<temporal::ZonedDateTime>, temporal::TemporalError>(temporal::diplomat::Ok<std::unique_ptr<temporal::ZonedDateTime>>(std::unique_ptr<temporal::ZonedDateTime>(temporal::ZonedDateTime::FromFFI(result.ok)))) : temporal::diplomat::result<std::unique_ptr<temporal::ZonedDateTime>, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline uint8_t temporal::ZonedDateTime::hour() const {
    auto result = temporal::capi::temporal_ZonedDateTime_hour(this->AsFFI());
    return result;
}

inline uint8_t temporal::ZonedDateTime::minute() const {
    auto result = temporal::capi::temporal_ZonedDateTime_minute(this->AsFFI());
    return result;
}

inline uint8_t temporal::ZonedDateTime::second() const {
    auto result = temporal::capi::temporal_ZonedDateTime_second(this->AsFFI());
    return result;
}

inline uint16_t temporal::ZonedDateTime::millisecond() const {
    auto result = temporal::capi::temporal_ZonedDateTime_millisecond(this->AsFFI());
    return result;
}

inline uint16_t temporal::ZonedDateTime::microsecond() const {
    auto result = temporal::capi::temporal_ZonedDateTime_microsecond(this->AsFFI());
    return result;
}

inline uint16_t temporal::ZonedDateTime::nanosecond() const {
    auto result = temporal::capi::temporal_ZonedDateTime_nanosecond(this->AsFFI());
    return result;
}

inline const temporal::Calendar& temporal::ZonedDateTime::calendar() const DIPLOMAT_LIFETIME_BOUND {
    auto result = temporal::capi::temporal_ZonedDateTime_calendar(this->AsFFI());
    return *temporal::Calendar::FromFFI(result);
}

inline int32_t temporal::ZonedDateTime::year() const {
    auto result = temporal::capi::temporal_ZonedDateTime_year(this->AsFFI());
    return result;
}

inline uint8_t temporal::ZonedDateTime::month() const {
    auto result = temporal::capi::temporal_ZonedDateTime_month(this->AsFFI());
    return result;
}

inline std::string temporal::ZonedDateTime::month_code() const {
    std::string output;
    temporal::diplomat::capi::DiplomatWrite write = temporal::diplomat::WriteFromString(output);
    temporal::capi::temporal_ZonedDateTime_month_code(this->AsFFI(),
        &write);
    return output;
}
template<typename W>
inline void temporal::ZonedDateTime::month_code_write(W& writeable) const {
    temporal::diplomat::capi::DiplomatWrite write = temporal::diplomat::WriteTrait<W>::Construct(writeable);
    temporal::capi::temporal_ZonedDateTime_month_code(this->AsFFI(),
        &write);
}

inline uint8_t temporal::ZonedDateTime::day() const {
    auto result = temporal::capi::temporal_ZonedDateTime_day(this->AsFFI());
    return result;
}

inline uint16_t temporal::ZonedDateTime::day_of_week() const {
    auto result = temporal::capi::temporal_ZonedDateTime_day_of_week(this->AsFFI());
    return result;
}

inline uint16_t temporal::ZonedDateTime::day_of_year() const {
    auto result = temporal::capi::temporal_ZonedDateTime_day_of_year(this->AsFFI());
    return result;
}

inline std::optional<uint8_t> temporal::ZonedDateTime::week_of_year() const {
    auto result = temporal::capi::temporal_ZonedDateTime_week_of_year(this->AsFFI());
    return result.is_ok ? std::optional<uint8_t>(result.ok) : std::nullopt;
}

inline std::optional<int32_t> temporal::ZonedDateTime::year_of_week() const {
    auto result = temporal::capi::temporal_ZonedDateTime_year_of_week(this->AsFFI());
    return result.is_ok ? std::optional<int32_t>(result.ok) : std::nullopt;
}

inline uint16_t temporal::ZonedDateTime::days_in_week() const {
    auto result = temporal::capi::temporal_ZonedDateTime_days_in_week(this->AsFFI());
    return result;
}

inline uint16_t temporal::ZonedDateTime::days_in_month() const {
    auto result = temporal::capi::temporal_ZonedDateTime_days_in_month(this->AsFFI());
    return result;
}

inline uint16_t temporal::ZonedDateTime::days_in_year() const {
    auto result = temporal::capi::temporal_ZonedDateTime_days_in_year(this->AsFFI());
    return result;
}

inline uint16_t temporal::ZonedDateTime::months_in_year() const {
    auto result = temporal::capi::temporal_ZonedDateTime_months_in_year(this->AsFFI());
    return result;
}

inline bool temporal::ZonedDateTime::in_leap_year() const {
    auto result = temporal::capi::temporal_ZonedDateTime_in_leap_year(this->AsFFI());
    return result;
}

inline std::string temporal::ZonedDateTime::era() const {
    std::string output;
    temporal::diplomat::capi::DiplomatWrite write = temporal::diplomat::WriteFromString(output);
    temporal::capi::temporal_ZonedDateTime_era(this->AsFFI(),
        &write);
    return output;
}
template<typename W>
inline void temporal::ZonedDateTime::era_write(W& writeable) const {
    temporal::diplomat::capi::DiplomatWrite write = temporal::diplomat::WriteTrait<W>::Construct(writeable);
    temporal::capi::temporal_ZonedDateTime_era(this->AsFFI(),
        &write);
}

inline std::optional<int32_t> temporal::ZonedDateTime::era_year() const {
    auto result = temporal::capi::temporal_ZonedDateTime_era_year(this->AsFFI());
    return result.is_ok ? std::optional<int32_t>(result.ok) : std::nullopt;
}

inline std::unique_ptr<temporal::ZonedDateTime> temporal::ZonedDateTime::clone() const {
    auto result = temporal::capi::temporal_ZonedDateTime_clone(this->AsFFI());
    return std::unique_ptr<temporal::ZonedDateTime>(temporal::ZonedDateTime::FromFFI(result));
}

inline const temporal::capi::ZonedDateTime* temporal::ZonedDateTime::AsFFI() const {
    return reinterpret_cast<const temporal::capi::ZonedDateTime*>(this);
}

inline temporal::capi::ZonedDateTime* temporal::ZonedDateTime::AsFFI() {
    return reinterpret_cast<temporal::capi::ZonedDateTime*>(this);
}

inline const temporal::ZonedDateTime* temporal::ZonedDateTime::FromFFI(const temporal::capi::ZonedDateTime* ptr) {
    return reinterpret_cast<const temporal::ZonedDateTime*>(ptr);
}

inline temporal::ZonedDateTime* temporal::ZonedDateTime::FromFFI(temporal::capi::ZonedDateTime* ptr) {
    return reinterpret_cast<temporal::ZonedDateTime*>(ptr);
}

inline void temporal::ZonedDateTime::operator delete(void* ptr) {
    temporal::capi::temporal_ZonedDateTime_destroy(reinterpret_cast<temporal::capi::ZonedDateTime*>(ptr));
}


#endif // TEMPORAL_ZonedDateTime_HPP
