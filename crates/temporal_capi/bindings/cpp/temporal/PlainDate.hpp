#ifndef TEMPORAL_PlainDate_HPP
#define TEMPORAL_PlainDate_HPP

#include "PlainDate.d.hpp"

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
#include "DisplayCalendar.hpp"
#include "Duration.hpp"
#include "I128Nanoseconds.hpp"
#include "ParsedDate.hpp"
#include "PartialDate.hpp"
#include "PlainDateTime.hpp"
#include "PlainMonthDay.hpp"
#include "PlainTime.hpp"
#include "PlainYearMonth.hpp"
#include "Provider.hpp"
#include "TemporalError.hpp"
#include "TimeZone.hpp"
#include "ZonedDateTime.hpp"
#include "diplomat_runtime.hpp"


namespace temporal {
namespace capi {
    extern "C" {

    typedef struct temporal_PlainDate_try_new_constrain_result {union {temporal::capi::PlainDate* ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_PlainDate_try_new_constrain_result;
    temporal_PlainDate_try_new_constrain_result temporal_PlainDate_try_new_constrain(int32_t year, uint8_t month, uint8_t day, temporal::capi::AnyCalendarKind calendar);

    typedef struct temporal_PlainDate_try_new_result {union {temporal::capi::PlainDate* ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_PlainDate_try_new_result;
    temporal_PlainDate_try_new_result temporal_PlainDate_try_new(int32_t year, uint8_t month, uint8_t day, temporal::capi::AnyCalendarKind calendar);

    typedef struct temporal_PlainDate_try_new_with_overflow_result {union {temporal::capi::PlainDate* ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_PlainDate_try_new_with_overflow_result;
    temporal_PlainDate_try_new_with_overflow_result temporal_PlainDate_try_new_with_overflow(int32_t year, uint8_t month, uint8_t day, temporal::capi::AnyCalendarKind calendar, temporal::capi::ArithmeticOverflow overflow);

    typedef struct temporal_PlainDate_from_partial_result {union {temporal::capi::PlainDate* ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_PlainDate_from_partial_result;
    temporal_PlainDate_from_partial_result temporal_PlainDate_from_partial(temporal::capi::PartialDate partial, temporal::capi::ArithmeticOverflow_option overflow);

    typedef struct temporal_PlainDate_from_parsed_result {union {temporal::capi::PlainDate* ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_PlainDate_from_parsed_result;
    temporal_PlainDate_from_parsed_result temporal_PlainDate_from_parsed(const temporal::capi::ParsedDate* parsed);

    typedef struct temporal_PlainDate_from_epoch_milliseconds_result {union {temporal::capi::PlainDate* ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_PlainDate_from_epoch_milliseconds_result;
    temporal_PlainDate_from_epoch_milliseconds_result temporal_PlainDate_from_epoch_milliseconds(int64_t ms, temporal::capi::TimeZone tz);

    typedef struct temporal_PlainDate_from_epoch_milliseconds_with_provider_result {union {temporal::capi::PlainDate* ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_PlainDate_from_epoch_milliseconds_with_provider_result;
    temporal_PlainDate_from_epoch_milliseconds_with_provider_result temporal_PlainDate_from_epoch_milliseconds_with_provider(int64_t ms, temporal::capi::TimeZone tz, const temporal::capi::Provider* p);

    typedef struct temporal_PlainDate_from_epoch_nanoseconds_result {union {temporal::capi::PlainDate* ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_PlainDate_from_epoch_nanoseconds_result;
    temporal_PlainDate_from_epoch_nanoseconds_result temporal_PlainDate_from_epoch_nanoseconds(temporal::capi::I128Nanoseconds ns, temporal::capi::TimeZone tz);

    typedef struct temporal_PlainDate_from_epoch_nanoseconds_with_provider_result {union {temporal::capi::PlainDate* ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_PlainDate_from_epoch_nanoseconds_with_provider_result;
    temporal_PlainDate_from_epoch_nanoseconds_with_provider_result temporal_PlainDate_from_epoch_nanoseconds_with_provider(temporal::capi::I128Nanoseconds ns, temporal::capi::TimeZone tz, const temporal::capi::Provider* p);

    typedef struct temporal_PlainDate_with_result {union {temporal::capi::PlainDate* ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_PlainDate_with_result;
    temporal_PlainDate_with_result temporal_PlainDate_with(const temporal::capi::PlainDate* self, temporal::capi::PartialDate partial, temporal::capi::ArithmeticOverflow_option overflow);

    temporal::capi::PlainDate* temporal_PlainDate_with_calendar(const temporal::capi::PlainDate* self, temporal::capi::AnyCalendarKind calendar);

    typedef struct temporal_PlainDate_from_utf8_result {union {temporal::capi::PlainDate* ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_PlainDate_from_utf8_result;
    temporal_PlainDate_from_utf8_result temporal_PlainDate_from_utf8(temporal::diplomat::capi::DiplomatStringView s);

    typedef struct temporal_PlainDate_from_utf16_result {union {temporal::capi::PlainDate* ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_PlainDate_from_utf16_result;
    temporal_PlainDate_from_utf16_result temporal_PlainDate_from_utf16(temporal::diplomat::capi::DiplomatString16View s);

    const temporal::capi::Calendar* temporal_PlainDate_calendar(const temporal::capi::PlainDate* self);

    bool temporal_PlainDate_is_valid(const temporal::capi::PlainDate* self);

    typedef struct temporal_PlainDate_add_result {union {temporal::capi::PlainDate* ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_PlainDate_add_result;
    temporal_PlainDate_add_result temporal_PlainDate_add(const temporal::capi::PlainDate* self, const temporal::capi::Duration* duration, temporal::capi::ArithmeticOverflow_option overflow);

    typedef struct temporal_PlainDate_subtract_result {union {temporal::capi::PlainDate* ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_PlainDate_subtract_result;
    temporal_PlainDate_subtract_result temporal_PlainDate_subtract(const temporal::capi::PlainDate* self, const temporal::capi::Duration* duration, temporal::capi::ArithmeticOverflow_option overflow);

    typedef struct temporal_PlainDate_until_result {union {temporal::capi::Duration* ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_PlainDate_until_result;
    temporal_PlainDate_until_result temporal_PlainDate_until(const temporal::capi::PlainDate* self, const temporal::capi::PlainDate* other, temporal::capi::DifferenceSettings settings);

    typedef struct temporal_PlainDate_since_result {union {temporal::capi::Duration* ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_PlainDate_since_result;
    temporal_PlainDate_since_result temporal_PlainDate_since(const temporal::capi::PlainDate* self, const temporal::capi::PlainDate* other, temporal::capi::DifferenceSettings settings);

    bool temporal_PlainDate_equals(const temporal::capi::PlainDate* self, const temporal::capi::PlainDate* other);

    int8_t temporal_PlainDate_compare(const temporal::capi::PlainDate* one, const temporal::capi::PlainDate* two);

    int32_t temporal_PlainDate_year(const temporal::capi::PlainDate* self);

    uint8_t temporal_PlainDate_month(const temporal::capi::PlainDate* self);

    void temporal_PlainDate_month_code(const temporal::capi::PlainDate* self, temporal::diplomat::capi::DiplomatWrite* write);

    uint8_t temporal_PlainDate_day(const temporal::capi::PlainDate* self);

    uint16_t temporal_PlainDate_day_of_week(const temporal::capi::PlainDate* self);

    uint16_t temporal_PlainDate_day_of_year(const temporal::capi::PlainDate* self);

    typedef struct temporal_PlainDate_week_of_year_result {union {uint8_t ok; }; bool is_ok;} temporal_PlainDate_week_of_year_result;
    temporal_PlainDate_week_of_year_result temporal_PlainDate_week_of_year(const temporal::capi::PlainDate* self);

    typedef struct temporal_PlainDate_year_of_week_result {union {int32_t ok; }; bool is_ok;} temporal_PlainDate_year_of_week_result;
    temporal_PlainDate_year_of_week_result temporal_PlainDate_year_of_week(const temporal::capi::PlainDate* self);

    uint16_t temporal_PlainDate_days_in_week(const temporal::capi::PlainDate* self);

    uint16_t temporal_PlainDate_days_in_month(const temporal::capi::PlainDate* self);

    uint16_t temporal_PlainDate_days_in_year(const temporal::capi::PlainDate* self);

    uint16_t temporal_PlainDate_months_in_year(const temporal::capi::PlainDate* self);

    bool temporal_PlainDate_in_leap_year(const temporal::capi::PlainDate* self);

    void temporal_PlainDate_era(const temporal::capi::PlainDate* self, temporal::diplomat::capi::DiplomatWrite* write);

    typedef struct temporal_PlainDate_era_year_result {union {int32_t ok; }; bool is_ok;} temporal_PlainDate_era_year_result;
    temporal_PlainDate_era_year_result temporal_PlainDate_era_year(const temporal::capi::PlainDate* self);

    typedef struct temporal_PlainDate_to_plain_date_time_result {union {temporal::capi::PlainDateTime* ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_PlainDate_to_plain_date_time_result;
    temporal_PlainDate_to_plain_date_time_result temporal_PlainDate_to_plain_date_time(const temporal::capi::PlainDate* self, const temporal::capi::PlainTime* time);

    typedef struct temporal_PlainDate_to_plain_month_day_result {union {temporal::capi::PlainMonthDay* ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_PlainDate_to_plain_month_day_result;
    temporal_PlainDate_to_plain_month_day_result temporal_PlainDate_to_plain_month_day(const temporal::capi::PlainDate* self);

    typedef struct temporal_PlainDate_to_plain_year_month_result {union {temporal::capi::PlainYearMonth* ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_PlainDate_to_plain_year_month_result;
    temporal_PlainDate_to_plain_year_month_result temporal_PlainDate_to_plain_year_month(const temporal::capi::PlainDate* self);

    typedef struct temporal_PlainDate_to_zoned_date_time_result {union {temporal::capi::ZonedDateTime* ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_PlainDate_to_zoned_date_time_result;
    temporal_PlainDate_to_zoned_date_time_result temporal_PlainDate_to_zoned_date_time(const temporal::capi::PlainDate* self, temporal::capi::TimeZone time_zone, const temporal::capi::PlainTime* time);

    typedef struct temporal_PlainDate_to_zoned_date_time_with_provider_result {union {temporal::capi::ZonedDateTime* ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_PlainDate_to_zoned_date_time_with_provider_result;
    temporal_PlainDate_to_zoned_date_time_with_provider_result temporal_PlainDate_to_zoned_date_time_with_provider(const temporal::capi::PlainDate* self, temporal::capi::TimeZone time_zone, const temporal::capi::PlainTime* time, const temporal::capi::Provider* p);

    typedef struct temporal_PlainDate_epoch_ms_for_utc_result {union {int64_t ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_PlainDate_epoch_ms_for_utc_result;
    temporal_PlainDate_epoch_ms_for_utc_result temporal_PlainDate_epoch_ms_for_utc(const temporal::capi::PlainDate* self);

    void temporal_PlainDate_to_ixdtf_string(const temporal::capi::PlainDate* self, temporal::capi::DisplayCalendar display_calendar, temporal::diplomat::capi::DiplomatWrite* write);

    temporal::capi::PlainDate* temporal_PlainDate_clone(const temporal::capi::PlainDate* self);

    void temporal_PlainDate_destroy(PlainDate* self);

    } // extern "C"
} // namespace capi
} // namespace

inline temporal::diplomat::result<std::unique_ptr<temporal::PlainDate>, temporal::TemporalError> temporal::PlainDate::try_new_constrain(int32_t year, uint8_t month, uint8_t day, temporal::AnyCalendarKind calendar) {
    auto result = temporal::capi::temporal_PlainDate_try_new_constrain(year,
        month,
        day,
        calendar.AsFFI());
    return result.is_ok ? temporal::diplomat::result<std::unique_ptr<temporal::PlainDate>, temporal::TemporalError>(temporal::diplomat::Ok<std::unique_ptr<temporal::PlainDate>>(std::unique_ptr<temporal::PlainDate>(temporal::PlainDate::FromFFI(result.ok)))) : temporal::diplomat::result<std::unique_ptr<temporal::PlainDate>, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline temporal::diplomat::result<std::unique_ptr<temporal::PlainDate>, temporal::TemporalError> temporal::PlainDate::try_new(int32_t year, uint8_t month, uint8_t day, temporal::AnyCalendarKind calendar) {
    auto result = temporal::capi::temporal_PlainDate_try_new(year,
        month,
        day,
        calendar.AsFFI());
    return result.is_ok ? temporal::diplomat::result<std::unique_ptr<temporal::PlainDate>, temporal::TemporalError>(temporal::diplomat::Ok<std::unique_ptr<temporal::PlainDate>>(std::unique_ptr<temporal::PlainDate>(temporal::PlainDate::FromFFI(result.ok)))) : temporal::diplomat::result<std::unique_ptr<temporal::PlainDate>, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline temporal::diplomat::result<std::unique_ptr<temporal::PlainDate>, temporal::TemporalError> temporal::PlainDate::try_new_with_overflow(int32_t year, uint8_t month, uint8_t day, temporal::AnyCalendarKind calendar, temporal::ArithmeticOverflow overflow) {
    auto result = temporal::capi::temporal_PlainDate_try_new_with_overflow(year,
        month,
        day,
        calendar.AsFFI(),
        overflow.AsFFI());
    return result.is_ok ? temporal::diplomat::result<std::unique_ptr<temporal::PlainDate>, temporal::TemporalError>(temporal::diplomat::Ok<std::unique_ptr<temporal::PlainDate>>(std::unique_ptr<temporal::PlainDate>(temporal::PlainDate::FromFFI(result.ok)))) : temporal::diplomat::result<std::unique_ptr<temporal::PlainDate>, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline temporal::diplomat::result<std::unique_ptr<temporal::PlainDate>, temporal::TemporalError> temporal::PlainDate::from_partial(temporal::PartialDate partial, std::optional<temporal::ArithmeticOverflow> overflow) {
    auto result = temporal::capi::temporal_PlainDate_from_partial(partial.AsFFI(),
        overflow.has_value() ? (temporal::capi::ArithmeticOverflow_option{ { overflow.value().AsFFI() }, true }) : (temporal::capi::ArithmeticOverflow_option{ {}, false }));
    return result.is_ok ? temporal::diplomat::result<std::unique_ptr<temporal::PlainDate>, temporal::TemporalError>(temporal::diplomat::Ok<std::unique_ptr<temporal::PlainDate>>(std::unique_ptr<temporal::PlainDate>(temporal::PlainDate::FromFFI(result.ok)))) : temporal::diplomat::result<std::unique_ptr<temporal::PlainDate>, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline temporal::diplomat::result<std::unique_ptr<temporal::PlainDate>, temporal::TemporalError> temporal::PlainDate::from_parsed(const temporal::ParsedDate& parsed) {
    auto result = temporal::capi::temporal_PlainDate_from_parsed(parsed.AsFFI());
    return result.is_ok ? temporal::diplomat::result<std::unique_ptr<temporal::PlainDate>, temporal::TemporalError>(temporal::diplomat::Ok<std::unique_ptr<temporal::PlainDate>>(std::unique_ptr<temporal::PlainDate>(temporal::PlainDate::FromFFI(result.ok)))) : temporal::diplomat::result<std::unique_ptr<temporal::PlainDate>, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline temporal::diplomat::result<std::unique_ptr<temporal::PlainDate>, temporal::TemporalError> temporal::PlainDate::from_epoch_milliseconds(int64_t ms, temporal::TimeZone tz) {
    auto result = temporal::capi::temporal_PlainDate_from_epoch_milliseconds(ms,
        tz.AsFFI());
    return result.is_ok ? temporal::diplomat::result<std::unique_ptr<temporal::PlainDate>, temporal::TemporalError>(temporal::diplomat::Ok<std::unique_ptr<temporal::PlainDate>>(std::unique_ptr<temporal::PlainDate>(temporal::PlainDate::FromFFI(result.ok)))) : temporal::diplomat::result<std::unique_ptr<temporal::PlainDate>, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline temporal::diplomat::result<std::unique_ptr<temporal::PlainDate>, temporal::TemporalError> temporal::PlainDate::from_epoch_milliseconds_with_provider(int64_t ms, temporal::TimeZone tz, const temporal::Provider& p) {
    auto result = temporal::capi::temporal_PlainDate_from_epoch_milliseconds_with_provider(ms,
        tz.AsFFI(),
        p.AsFFI());
    return result.is_ok ? temporal::diplomat::result<std::unique_ptr<temporal::PlainDate>, temporal::TemporalError>(temporal::diplomat::Ok<std::unique_ptr<temporal::PlainDate>>(std::unique_ptr<temporal::PlainDate>(temporal::PlainDate::FromFFI(result.ok)))) : temporal::diplomat::result<std::unique_ptr<temporal::PlainDate>, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline temporal::diplomat::result<std::unique_ptr<temporal::PlainDate>, temporal::TemporalError> temporal::PlainDate::from_epoch_nanoseconds(temporal::I128Nanoseconds ns, temporal::TimeZone tz) {
    auto result = temporal::capi::temporal_PlainDate_from_epoch_nanoseconds(ns.AsFFI(),
        tz.AsFFI());
    return result.is_ok ? temporal::diplomat::result<std::unique_ptr<temporal::PlainDate>, temporal::TemporalError>(temporal::diplomat::Ok<std::unique_ptr<temporal::PlainDate>>(std::unique_ptr<temporal::PlainDate>(temporal::PlainDate::FromFFI(result.ok)))) : temporal::diplomat::result<std::unique_ptr<temporal::PlainDate>, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline temporal::diplomat::result<std::unique_ptr<temporal::PlainDate>, temporal::TemporalError> temporal::PlainDate::from_epoch_nanoseconds_with_provider(temporal::I128Nanoseconds ns, temporal::TimeZone tz, const temporal::Provider& p) {
    auto result = temporal::capi::temporal_PlainDate_from_epoch_nanoseconds_with_provider(ns.AsFFI(),
        tz.AsFFI(),
        p.AsFFI());
    return result.is_ok ? temporal::diplomat::result<std::unique_ptr<temporal::PlainDate>, temporal::TemporalError>(temporal::diplomat::Ok<std::unique_ptr<temporal::PlainDate>>(std::unique_ptr<temporal::PlainDate>(temporal::PlainDate::FromFFI(result.ok)))) : temporal::diplomat::result<std::unique_ptr<temporal::PlainDate>, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline temporal::diplomat::result<std::unique_ptr<temporal::PlainDate>, temporal::TemporalError> temporal::PlainDate::with(temporal::PartialDate partial, std::optional<temporal::ArithmeticOverflow> overflow) const {
    auto result = temporal::capi::temporal_PlainDate_with(this->AsFFI(),
        partial.AsFFI(),
        overflow.has_value() ? (temporal::capi::ArithmeticOverflow_option{ { overflow.value().AsFFI() }, true }) : (temporal::capi::ArithmeticOverflow_option{ {}, false }));
    return result.is_ok ? temporal::diplomat::result<std::unique_ptr<temporal::PlainDate>, temporal::TemporalError>(temporal::diplomat::Ok<std::unique_ptr<temporal::PlainDate>>(std::unique_ptr<temporal::PlainDate>(temporal::PlainDate::FromFFI(result.ok)))) : temporal::diplomat::result<std::unique_ptr<temporal::PlainDate>, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline std::unique_ptr<temporal::PlainDate> temporal::PlainDate::with_calendar(temporal::AnyCalendarKind calendar) const {
    auto result = temporal::capi::temporal_PlainDate_with_calendar(this->AsFFI(),
        calendar.AsFFI());
    return std::unique_ptr<temporal::PlainDate>(temporal::PlainDate::FromFFI(result));
}

inline temporal::diplomat::result<std::unique_ptr<temporal::PlainDate>, temporal::TemporalError> temporal::PlainDate::from_utf8(std::string_view s) {
    auto result = temporal::capi::temporal_PlainDate_from_utf8({s.data(), s.size()});
    return result.is_ok ? temporal::diplomat::result<std::unique_ptr<temporal::PlainDate>, temporal::TemporalError>(temporal::diplomat::Ok<std::unique_ptr<temporal::PlainDate>>(std::unique_ptr<temporal::PlainDate>(temporal::PlainDate::FromFFI(result.ok)))) : temporal::diplomat::result<std::unique_ptr<temporal::PlainDate>, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline temporal::diplomat::result<std::unique_ptr<temporal::PlainDate>, temporal::TemporalError> temporal::PlainDate::from_utf16(std::u16string_view s) {
    auto result = temporal::capi::temporal_PlainDate_from_utf16({s.data(), s.size()});
    return result.is_ok ? temporal::diplomat::result<std::unique_ptr<temporal::PlainDate>, temporal::TemporalError>(temporal::diplomat::Ok<std::unique_ptr<temporal::PlainDate>>(std::unique_ptr<temporal::PlainDate>(temporal::PlainDate::FromFFI(result.ok)))) : temporal::diplomat::result<std::unique_ptr<temporal::PlainDate>, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline const temporal::Calendar& temporal::PlainDate::calendar() const DIPLOMAT_LIFETIME_BOUND {
    auto result = temporal::capi::temporal_PlainDate_calendar(this->AsFFI());
    return *temporal::Calendar::FromFFI(result);
}

inline bool temporal::PlainDate::is_valid() const {
    auto result = temporal::capi::temporal_PlainDate_is_valid(this->AsFFI());
    return result;
}

inline temporal::diplomat::result<std::unique_ptr<temporal::PlainDate>, temporal::TemporalError> temporal::PlainDate::add(const temporal::Duration& duration, std::optional<temporal::ArithmeticOverflow> overflow) const {
    auto result = temporal::capi::temporal_PlainDate_add(this->AsFFI(),
        duration.AsFFI(),
        overflow.has_value() ? (temporal::capi::ArithmeticOverflow_option{ { overflow.value().AsFFI() }, true }) : (temporal::capi::ArithmeticOverflow_option{ {}, false }));
    return result.is_ok ? temporal::diplomat::result<std::unique_ptr<temporal::PlainDate>, temporal::TemporalError>(temporal::diplomat::Ok<std::unique_ptr<temporal::PlainDate>>(std::unique_ptr<temporal::PlainDate>(temporal::PlainDate::FromFFI(result.ok)))) : temporal::diplomat::result<std::unique_ptr<temporal::PlainDate>, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline temporal::diplomat::result<std::unique_ptr<temporal::PlainDate>, temporal::TemporalError> temporal::PlainDate::subtract(const temporal::Duration& duration, std::optional<temporal::ArithmeticOverflow> overflow) const {
    auto result = temporal::capi::temporal_PlainDate_subtract(this->AsFFI(),
        duration.AsFFI(),
        overflow.has_value() ? (temporal::capi::ArithmeticOverflow_option{ { overflow.value().AsFFI() }, true }) : (temporal::capi::ArithmeticOverflow_option{ {}, false }));
    return result.is_ok ? temporal::diplomat::result<std::unique_ptr<temporal::PlainDate>, temporal::TemporalError>(temporal::diplomat::Ok<std::unique_ptr<temporal::PlainDate>>(std::unique_ptr<temporal::PlainDate>(temporal::PlainDate::FromFFI(result.ok)))) : temporal::diplomat::result<std::unique_ptr<temporal::PlainDate>, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline temporal::diplomat::result<std::unique_ptr<temporal::Duration>, temporal::TemporalError> temporal::PlainDate::until(const temporal::PlainDate& other, temporal::DifferenceSettings settings) const {
    auto result = temporal::capi::temporal_PlainDate_until(this->AsFFI(),
        other.AsFFI(),
        settings.AsFFI());
    return result.is_ok ? temporal::diplomat::result<std::unique_ptr<temporal::Duration>, temporal::TemporalError>(temporal::diplomat::Ok<std::unique_ptr<temporal::Duration>>(std::unique_ptr<temporal::Duration>(temporal::Duration::FromFFI(result.ok)))) : temporal::diplomat::result<std::unique_ptr<temporal::Duration>, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline temporal::diplomat::result<std::unique_ptr<temporal::Duration>, temporal::TemporalError> temporal::PlainDate::since(const temporal::PlainDate& other, temporal::DifferenceSettings settings) const {
    auto result = temporal::capi::temporal_PlainDate_since(this->AsFFI(),
        other.AsFFI(),
        settings.AsFFI());
    return result.is_ok ? temporal::diplomat::result<std::unique_ptr<temporal::Duration>, temporal::TemporalError>(temporal::diplomat::Ok<std::unique_ptr<temporal::Duration>>(std::unique_ptr<temporal::Duration>(temporal::Duration::FromFFI(result.ok)))) : temporal::diplomat::result<std::unique_ptr<temporal::Duration>, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline bool temporal::PlainDate::equals(const temporal::PlainDate& other) const {
    auto result = temporal::capi::temporal_PlainDate_equals(this->AsFFI(),
        other.AsFFI());
    return result;
}

inline int8_t temporal::PlainDate::compare(const temporal::PlainDate& one, const temporal::PlainDate& two) {
    auto result = temporal::capi::temporal_PlainDate_compare(one.AsFFI(),
        two.AsFFI());
    return result;
}

inline int32_t temporal::PlainDate::year() const {
    auto result = temporal::capi::temporal_PlainDate_year(this->AsFFI());
    return result;
}

inline uint8_t temporal::PlainDate::month() const {
    auto result = temporal::capi::temporal_PlainDate_month(this->AsFFI());
    return result;
}

inline std::string temporal::PlainDate::month_code() const {
    std::string output;
    temporal::diplomat::capi::DiplomatWrite write = temporal::diplomat::WriteFromString(output);
    temporal::capi::temporal_PlainDate_month_code(this->AsFFI(),
        &write);
    return output;
}
template<typename W>
inline void temporal::PlainDate::month_code_write(W& writeable) const {
    temporal::diplomat::capi::DiplomatWrite write = temporal::diplomat::WriteTrait<W>::Construct(writeable);
    temporal::capi::temporal_PlainDate_month_code(this->AsFFI(),
        &write);
}

inline uint8_t temporal::PlainDate::day() const {
    auto result = temporal::capi::temporal_PlainDate_day(this->AsFFI());
    return result;
}

inline uint16_t temporal::PlainDate::day_of_week() const {
    auto result = temporal::capi::temporal_PlainDate_day_of_week(this->AsFFI());
    return result;
}

inline uint16_t temporal::PlainDate::day_of_year() const {
    auto result = temporal::capi::temporal_PlainDate_day_of_year(this->AsFFI());
    return result;
}

inline std::optional<uint8_t> temporal::PlainDate::week_of_year() const {
    auto result = temporal::capi::temporal_PlainDate_week_of_year(this->AsFFI());
    return result.is_ok ? std::optional<uint8_t>(result.ok) : std::nullopt;
}

inline std::optional<int32_t> temporal::PlainDate::year_of_week() const {
    auto result = temporal::capi::temporal_PlainDate_year_of_week(this->AsFFI());
    return result.is_ok ? std::optional<int32_t>(result.ok) : std::nullopt;
}

inline uint16_t temporal::PlainDate::days_in_week() const {
    auto result = temporal::capi::temporal_PlainDate_days_in_week(this->AsFFI());
    return result;
}

inline uint16_t temporal::PlainDate::days_in_month() const {
    auto result = temporal::capi::temporal_PlainDate_days_in_month(this->AsFFI());
    return result;
}

inline uint16_t temporal::PlainDate::days_in_year() const {
    auto result = temporal::capi::temporal_PlainDate_days_in_year(this->AsFFI());
    return result;
}

inline uint16_t temporal::PlainDate::months_in_year() const {
    auto result = temporal::capi::temporal_PlainDate_months_in_year(this->AsFFI());
    return result;
}

inline bool temporal::PlainDate::in_leap_year() const {
    auto result = temporal::capi::temporal_PlainDate_in_leap_year(this->AsFFI());
    return result;
}

inline std::string temporal::PlainDate::era() const {
    std::string output;
    temporal::diplomat::capi::DiplomatWrite write = temporal::diplomat::WriteFromString(output);
    temporal::capi::temporal_PlainDate_era(this->AsFFI(),
        &write);
    return output;
}
template<typename W>
inline void temporal::PlainDate::era_write(W& writeable) const {
    temporal::diplomat::capi::DiplomatWrite write = temporal::diplomat::WriteTrait<W>::Construct(writeable);
    temporal::capi::temporal_PlainDate_era(this->AsFFI(),
        &write);
}

inline std::optional<int32_t> temporal::PlainDate::era_year() const {
    auto result = temporal::capi::temporal_PlainDate_era_year(this->AsFFI());
    return result.is_ok ? std::optional<int32_t>(result.ok) : std::nullopt;
}

inline temporal::diplomat::result<std::unique_ptr<temporal::PlainDateTime>, temporal::TemporalError> temporal::PlainDate::to_plain_date_time(const temporal::PlainTime* time) const {
    auto result = temporal::capi::temporal_PlainDate_to_plain_date_time(this->AsFFI(),
        time ? time->AsFFI() : nullptr);
    return result.is_ok ? temporal::diplomat::result<std::unique_ptr<temporal::PlainDateTime>, temporal::TemporalError>(temporal::diplomat::Ok<std::unique_ptr<temporal::PlainDateTime>>(std::unique_ptr<temporal::PlainDateTime>(temporal::PlainDateTime::FromFFI(result.ok)))) : temporal::diplomat::result<std::unique_ptr<temporal::PlainDateTime>, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline temporal::diplomat::result<std::unique_ptr<temporal::PlainMonthDay>, temporal::TemporalError> temporal::PlainDate::to_plain_month_day() const {
    auto result = temporal::capi::temporal_PlainDate_to_plain_month_day(this->AsFFI());
    return result.is_ok ? temporal::diplomat::result<std::unique_ptr<temporal::PlainMonthDay>, temporal::TemporalError>(temporal::diplomat::Ok<std::unique_ptr<temporal::PlainMonthDay>>(std::unique_ptr<temporal::PlainMonthDay>(temporal::PlainMonthDay::FromFFI(result.ok)))) : temporal::diplomat::result<std::unique_ptr<temporal::PlainMonthDay>, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline temporal::diplomat::result<std::unique_ptr<temporal::PlainYearMonth>, temporal::TemporalError> temporal::PlainDate::to_plain_year_month() const {
    auto result = temporal::capi::temporal_PlainDate_to_plain_year_month(this->AsFFI());
    return result.is_ok ? temporal::diplomat::result<std::unique_ptr<temporal::PlainYearMonth>, temporal::TemporalError>(temporal::diplomat::Ok<std::unique_ptr<temporal::PlainYearMonth>>(std::unique_ptr<temporal::PlainYearMonth>(temporal::PlainYearMonth::FromFFI(result.ok)))) : temporal::diplomat::result<std::unique_ptr<temporal::PlainYearMonth>, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline temporal::diplomat::result<std::unique_ptr<temporal::ZonedDateTime>, temporal::TemporalError> temporal::PlainDate::to_zoned_date_time(temporal::TimeZone time_zone, const temporal::PlainTime* time) const {
    auto result = temporal::capi::temporal_PlainDate_to_zoned_date_time(this->AsFFI(),
        time_zone.AsFFI(),
        time ? time->AsFFI() : nullptr);
    return result.is_ok ? temporal::diplomat::result<std::unique_ptr<temporal::ZonedDateTime>, temporal::TemporalError>(temporal::diplomat::Ok<std::unique_ptr<temporal::ZonedDateTime>>(std::unique_ptr<temporal::ZonedDateTime>(temporal::ZonedDateTime::FromFFI(result.ok)))) : temporal::diplomat::result<std::unique_ptr<temporal::ZonedDateTime>, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline temporal::diplomat::result<std::unique_ptr<temporal::ZonedDateTime>, temporal::TemporalError> temporal::PlainDate::to_zoned_date_time_with_provider(temporal::TimeZone time_zone, const temporal::PlainTime* time, const temporal::Provider& p) const {
    auto result = temporal::capi::temporal_PlainDate_to_zoned_date_time_with_provider(this->AsFFI(),
        time_zone.AsFFI(),
        time ? time->AsFFI() : nullptr,
        p.AsFFI());
    return result.is_ok ? temporal::diplomat::result<std::unique_ptr<temporal::ZonedDateTime>, temporal::TemporalError>(temporal::diplomat::Ok<std::unique_ptr<temporal::ZonedDateTime>>(std::unique_ptr<temporal::ZonedDateTime>(temporal::ZonedDateTime::FromFFI(result.ok)))) : temporal::diplomat::result<std::unique_ptr<temporal::ZonedDateTime>, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline temporal::diplomat::result<int64_t, temporal::TemporalError> temporal::PlainDate::epoch_ms_for_utc() const {
    auto result = temporal::capi::temporal_PlainDate_epoch_ms_for_utc(this->AsFFI());
    return result.is_ok ? temporal::diplomat::result<int64_t, temporal::TemporalError>(temporal::diplomat::Ok<int64_t>(result.ok)) : temporal::diplomat::result<int64_t, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline std::string temporal::PlainDate::to_ixdtf_string(temporal::DisplayCalendar display_calendar) const {
    std::string output;
    temporal::diplomat::capi::DiplomatWrite write = temporal::diplomat::WriteFromString(output);
    temporal::capi::temporal_PlainDate_to_ixdtf_string(this->AsFFI(),
        display_calendar.AsFFI(),
        &write);
    return output;
}
template<typename W>
inline void temporal::PlainDate::to_ixdtf_string_write(temporal::DisplayCalendar display_calendar, W& writeable) const {
    temporal::diplomat::capi::DiplomatWrite write = temporal::diplomat::WriteTrait<W>::Construct(writeable);
    temporal::capi::temporal_PlainDate_to_ixdtf_string(this->AsFFI(),
        display_calendar.AsFFI(),
        &write);
}

inline std::unique_ptr<temporal::PlainDate> temporal::PlainDate::clone() const {
    auto result = temporal::capi::temporal_PlainDate_clone(this->AsFFI());
    return std::unique_ptr<temporal::PlainDate>(temporal::PlainDate::FromFFI(result));
}

inline const temporal::capi::PlainDate* temporal::PlainDate::AsFFI() const {
    return reinterpret_cast<const temporal::capi::PlainDate*>(this);
}

inline temporal::capi::PlainDate* temporal::PlainDate::AsFFI() {
    return reinterpret_cast<temporal::capi::PlainDate*>(this);
}

inline const temporal::PlainDate* temporal::PlainDate::FromFFI(const temporal::capi::PlainDate* ptr) {
    return reinterpret_cast<const temporal::PlainDate*>(ptr);
}

inline temporal::PlainDate* temporal::PlainDate::FromFFI(temporal::capi::PlainDate* ptr) {
    return reinterpret_cast<temporal::PlainDate*>(ptr);
}

inline void temporal::PlainDate::operator delete(void* ptr) {
    temporal::capi::temporal_PlainDate_destroy(reinterpret_cast<temporal::capi::PlainDate*>(ptr));
}


#endif // TEMPORAL_PlainDate_HPP
