#ifndef TEMPORAL_PlainDateTime_HPP
#define TEMPORAL_PlainDateTime_HPP

#include "PlainDateTime.d.hpp"

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
#include "Duration.hpp"
#include "I128Nanoseconds.hpp"
#include "ParsedDateTime.hpp"
#include "PartialDateTime.hpp"
#include "PlainDate.hpp"
#include "PlainTime.hpp"
#include "Provider.hpp"
#include "RoundingOptions.hpp"
#include "TemporalError.hpp"
#include "TimeZone.hpp"
#include "ToStringRoundingOptions.hpp"
#include "ZonedDateTime.hpp"
#include "diplomat_runtime.hpp"


namespace temporal {
namespace capi {
    extern "C" {

    typedef struct temporal_PlainDateTime_try_new_constrain_result {union {temporal::capi::PlainDateTime* ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_PlainDateTime_try_new_constrain_result;
    temporal_PlainDateTime_try_new_constrain_result temporal_PlainDateTime_try_new_constrain(int32_t year, uint8_t month, uint8_t day, uint8_t hour, uint8_t minute, uint8_t second, uint16_t millisecond, uint16_t microsecond, uint16_t nanosecond, temporal::capi::AnyCalendarKind calendar);

    typedef struct temporal_PlainDateTime_try_new_result {union {temporal::capi::PlainDateTime* ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_PlainDateTime_try_new_result;
    temporal_PlainDateTime_try_new_result temporal_PlainDateTime_try_new(int32_t year, uint8_t month, uint8_t day, uint8_t hour, uint8_t minute, uint8_t second, uint16_t millisecond, uint16_t microsecond, uint16_t nanosecond, temporal::capi::AnyCalendarKind calendar);

    typedef struct temporal_PlainDateTime_from_partial_result {union {temporal::capi::PlainDateTime* ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_PlainDateTime_from_partial_result;
    temporal_PlainDateTime_from_partial_result temporal_PlainDateTime_from_partial(temporal::capi::PartialDateTime partial, temporal::capi::ArithmeticOverflow_option overflow);

    typedef struct temporal_PlainDateTime_from_parsed_result {union {temporal::capi::PlainDateTime* ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_PlainDateTime_from_parsed_result;
    temporal_PlainDateTime_from_parsed_result temporal_PlainDateTime_from_parsed(const temporal::capi::ParsedDateTime* parsed);

    typedef struct temporal_PlainDateTime_from_epoch_milliseconds_result {union {temporal::capi::PlainDateTime* ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_PlainDateTime_from_epoch_milliseconds_result;
    temporal_PlainDateTime_from_epoch_milliseconds_result temporal_PlainDateTime_from_epoch_milliseconds(int64_t ms, temporal::capi::TimeZone tz);

    typedef struct temporal_PlainDateTime_from_epoch_milliseconds_with_provider_result {union {temporal::capi::PlainDateTime* ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_PlainDateTime_from_epoch_milliseconds_with_provider_result;
    temporal_PlainDateTime_from_epoch_milliseconds_with_provider_result temporal_PlainDateTime_from_epoch_milliseconds_with_provider(int64_t ms, temporal::capi::TimeZone tz, const temporal::capi::Provider* p);

    typedef struct temporal_PlainDateTime_from_epoch_nanoseconds_result {union {temporal::capi::PlainDateTime* ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_PlainDateTime_from_epoch_nanoseconds_result;
    temporal_PlainDateTime_from_epoch_nanoseconds_result temporal_PlainDateTime_from_epoch_nanoseconds(temporal::capi::I128Nanoseconds ns, temporal::capi::TimeZone tz);

    typedef struct temporal_PlainDateTime_from_epoch_nanoseconds_with_provider_result {union {temporal::capi::PlainDateTime* ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_PlainDateTime_from_epoch_nanoseconds_with_provider_result;
    temporal_PlainDateTime_from_epoch_nanoseconds_with_provider_result temporal_PlainDateTime_from_epoch_nanoseconds_with_provider(temporal::capi::I128Nanoseconds ns, temporal::capi::TimeZone tz, const temporal::capi::Provider* p);

    typedef struct temporal_PlainDateTime_with_result {union {temporal::capi::PlainDateTime* ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_PlainDateTime_with_result;
    temporal_PlainDateTime_with_result temporal_PlainDateTime_with(const temporal::capi::PlainDateTime* self, temporal::capi::PartialDateTime partial, temporal::capi::ArithmeticOverflow_option overflow);

    typedef struct temporal_PlainDateTime_with_time_result {union {temporal::capi::PlainDateTime* ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_PlainDateTime_with_time_result;
    temporal_PlainDateTime_with_time_result temporal_PlainDateTime_with_time(const temporal::capi::PlainDateTime* self, const temporal::capi::PlainTime* time);

    temporal::capi::PlainDateTime* temporal_PlainDateTime_with_calendar(const temporal::capi::PlainDateTime* self, temporal::capi::AnyCalendarKind calendar);

    typedef struct temporal_PlainDateTime_from_utf8_result {union {temporal::capi::PlainDateTime* ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_PlainDateTime_from_utf8_result;
    temporal_PlainDateTime_from_utf8_result temporal_PlainDateTime_from_utf8(temporal::diplomat::capi::DiplomatStringView s);

    typedef struct temporal_PlainDateTime_from_utf16_result {union {temporal::capi::PlainDateTime* ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_PlainDateTime_from_utf16_result;
    temporal_PlainDateTime_from_utf16_result temporal_PlainDateTime_from_utf16(temporal::diplomat::capi::DiplomatString16View s);

    uint8_t temporal_PlainDateTime_hour(const temporal::capi::PlainDateTime* self);

    uint8_t temporal_PlainDateTime_minute(const temporal::capi::PlainDateTime* self);

    uint8_t temporal_PlainDateTime_second(const temporal::capi::PlainDateTime* self);

    uint16_t temporal_PlainDateTime_millisecond(const temporal::capi::PlainDateTime* self);

    uint16_t temporal_PlainDateTime_microsecond(const temporal::capi::PlainDateTime* self);

    uint16_t temporal_PlainDateTime_nanosecond(const temporal::capi::PlainDateTime* self);

    const temporal::capi::Calendar* temporal_PlainDateTime_calendar(const temporal::capi::PlainDateTime* self);

    int32_t temporal_PlainDateTime_year(const temporal::capi::PlainDateTime* self);

    uint8_t temporal_PlainDateTime_month(const temporal::capi::PlainDateTime* self);

    void temporal_PlainDateTime_month_code(const temporal::capi::PlainDateTime* self, temporal::diplomat::capi::DiplomatWrite* write);

    uint8_t temporal_PlainDateTime_day(const temporal::capi::PlainDateTime* self);

    uint16_t temporal_PlainDateTime_day_of_week(const temporal::capi::PlainDateTime* self);

    uint16_t temporal_PlainDateTime_day_of_year(const temporal::capi::PlainDateTime* self);

    typedef struct temporal_PlainDateTime_week_of_year_result {union {uint8_t ok; }; bool is_ok;} temporal_PlainDateTime_week_of_year_result;
    temporal_PlainDateTime_week_of_year_result temporal_PlainDateTime_week_of_year(const temporal::capi::PlainDateTime* self);

    typedef struct temporal_PlainDateTime_year_of_week_result {union {int32_t ok; }; bool is_ok;} temporal_PlainDateTime_year_of_week_result;
    temporal_PlainDateTime_year_of_week_result temporal_PlainDateTime_year_of_week(const temporal::capi::PlainDateTime* self);

    uint16_t temporal_PlainDateTime_days_in_week(const temporal::capi::PlainDateTime* self);

    uint16_t temporal_PlainDateTime_days_in_month(const temporal::capi::PlainDateTime* self);

    uint16_t temporal_PlainDateTime_days_in_year(const temporal::capi::PlainDateTime* self);

    uint16_t temporal_PlainDateTime_months_in_year(const temporal::capi::PlainDateTime* self);

    bool temporal_PlainDateTime_in_leap_year(const temporal::capi::PlainDateTime* self);

    void temporal_PlainDateTime_era(const temporal::capi::PlainDateTime* self, temporal::diplomat::capi::DiplomatWrite* write);

    typedef struct temporal_PlainDateTime_era_year_result {union {int32_t ok; }; bool is_ok;} temporal_PlainDateTime_era_year_result;
    temporal_PlainDateTime_era_year_result temporal_PlainDateTime_era_year(const temporal::capi::PlainDateTime* self);

    typedef struct temporal_PlainDateTime_add_result {union {temporal::capi::PlainDateTime* ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_PlainDateTime_add_result;
    temporal_PlainDateTime_add_result temporal_PlainDateTime_add(const temporal::capi::PlainDateTime* self, const temporal::capi::Duration* duration, temporal::capi::ArithmeticOverflow_option overflow);

    typedef struct temporal_PlainDateTime_subtract_result {union {temporal::capi::PlainDateTime* ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_PlainDateTime_subtract_result;
    temporal_PlainDateTime_subtract_result temporal_PlainDateTime_subtract(const temporal::capi::PlainDateTime* self, const temporal::capi::Duration* duration, temporal::capi::ArithmeticOverflow_option overflow);

    typedef struct temporal_PlainDateTime_until_result {union {temporal::capi::Duration* ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_PlainDateTime_until_result;
    temporal_PlainDateTime_until_result temporal_PlainDateTime_until(const temporal::capi::PlainDateTime* self, const temporal::capi::PlainDateTime* other, temporal::capi::DifferenceSettings settings);

    typedef struct temporal_PlainDateTime_since_result {union {temporal::capi::Duration* ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_PlainDateTime_since_result;
    temporal_PlainDateTime_since_result temporal_PlainDateTime_since(const temporal::capi::PlainDateTime* self, const temporal::capi::PlainDateTime* other, temporal::capi::DifferenceSettings settings);

    bool temporal_PlainDateTime_equals(const temporal::capi::PlainDateTime* self, const temporal::capi::PlainDateTime* other);

    int8_t temporal_PlainDateTime_compare(const temporal::capi::PlainDateTime* one, const temporal::capi::PlainDateTime* two);

    typedef struct temporal_PlainDateTime_round_result {union {temporal::capi::PlainDateTime* ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_PlainDateTime_round_result;
    temporal_PlainDateTime_round_result temporal_PlainDateTime_round(const temporal::capi::PlainDateTime* self, temporal::capi::RoundingOptions options);

    temporal::capi::PlainDate* temporal_PlainDateTime_to_plain_date(const temporal::capi::PlainDateTime* self);

    temporal::capi::PlainTime* temporal_PlainDateTime_to_plain_time(const temporal::capi::PlainDateTime* self);

    typedef struct temporal_PlainDateTime_to_zoned_date_time_result {union {temporal::capi::ZonedDateTime* ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_PlainDateTime_to_zoned_date_time_result;
    temporal_PlainDateTime_to_zoned_date_time_result temporal_PlainDateTime_to_zoned_date_time(const temporal::capi::PlainDateTime* self, temporal::capi::TimeZone time_zone, temporal::capi::Disambiguation disambiguation);

    typedef struct temporal_PlainDateTime_to_zoned_date_time_with_provider_result {union {temporal::capi::ZonedDateTime* ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_PlainDateTime_to_zoned_date_time_with_provider_result;
    temporal_PlainDateTime_to_zoned_date_time_with_provider_result temporal_PlainDateTime_to_zoned_date_time_with_provider(const temporal::capi::PlainDateTime* self, temporal::capi::TimeZone time_zone, temporal::capi::Disambiguation disambiguation, const temporal::capi::Provider* p);

    typedef struct temporal_PlainDateTime_epoch_ms_for_utc_result {union {int64_t ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_PlainDateTime_epoch_ms_for_utc_result;
    temporal_PlainDateTime_epoch_ms_for_utc_result temporal_PlainDateTime_epoch_ms_for_utc(const temporal::capi::PlainDateTime* self);

    typedef struct temporal_PlainDateTime_to_ixdtf_string_result {union { temporal::capi::TemporalError err;}; bool is_ok;} temporal_PlainDateTime_to_ixdtf_string_result;
    temporal_PlainDateTime_to_ixdtf_string_result temporal_PlainDateTime_to_ixdtf_string(const temporal::capi::PlainDateTime* self, temporal::capi::ToStringRoundingOptions options, temporal::capi::DisplayCalendar display_calendar, temporal::diplomat::capi::DiplomatWrite* write);

    temporal::capi::PlainDateTime* temporal_PlainDateTime_clone(const temporal::capi::PlainDateTime* self);

    void temporal_PlainDateTime_destroy(PlainDateTime* self);

    } // extern "C"
} // namespace capi
} // namespace

inline temporal::diplomat::result<std::unique_ptr<temporal::PlainDateTime>, temporal::TemporalError> temporal::PlainDateTime::try_new_constrain(int32_t year, uint8_t month, uint8_t day, uint8_t hour, uint8_t minute, uint8_t second, uint16_t millisecond, uint16_t microsecond, uint16_t nanosecond, temporal::AnyCalendarKind calendar) {
    auto result = temporal::capi::temporal_PlainDateTime_try_new_constrain(year,
        month,
        day,
        hour,
        minute,
        second,
        millisecond,
        microsecond,
        nanosecond,
        calendar.AsFFI());
    return result.is_ok ? temporal::diplomat::result<std::unique_ptr<temporal::PlainDateTime>, temporal::TemporalError>(temporal::diplomat::Ok<std::unique_ptr<temporal::PlainDateTime>>(std::unique_ptr<temporal::PlainDateTime>(temporal::PlainDateTime::FromFFI(result.ok)))) : temporal::diplomat::result<std::unique_ptr<temporal::PlainDateTime>, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline temporal::diplomat::result<std::unique_ptr<temporal::PlainDateTime>, temporal::TemporalError> temporal::PlainDateTime::try_new(int32_t year, uint8_t month, uint8_t day, uint8_t hour, uint8_t minute, uint8_t second, uint16_t millisecond, uint16_t microsecond, uint16_t nanosecond, temporal::AnyCalendarKind calendar) {
    auto result = temporal::capi::temporal_PlainDateTime_try_new(year,
        month,
        day,
        hour,
        minute,
        second,
        millisecond,
        microsecond,
        nanosecond,
        calendar.AsFFI());
    return result.is_ok ? temporal::diplomat::result<std::unique_ptr<temporal::PlainDateTime>, temporal::TemporalError>(temporal::diplomat::Ok<std::unique_ptr<temporal::PlainDateTime>>(std::unique_ptr<temporal::PlainDateTime>(temporal::PlainDateTime::FromFFI(result.ok)))) : temporal::diplomat::result<std::unique_ptr<temporal::PlainDateTime>, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline temporal::diplomat::result<std::unique_ptr<temporal::PlainDateTime>, temporal::TemporalError> temporal::PlainDateTime::from_partial(temporal::PartialDateTime partial, std::optional<temporal::ArithmeticOverflow> overflow) {
    auto result = temporal::capi::temporal_PlainDateTime_from_partial(partial.AsFFI(),
        overflow.has_value() ? (temporal::capi::ArithmeticOverflow_option{ { overflow.value().AsFFI() }, true }) : (temporal::capi::ArithmeticOverflow_option{ {}, false }));
    return result.is_ok ? temporal::diplomat::result<std::unique_ptr<temporal::PlainDateTime>, temporal::TemporalError>(temporal::diplomat::Ok<std::unique_ptr<temporal::PlainDateTime>>(std::unique_ptr<temporal::PlainDateTime>(temporal::PlainDateTime::FromFFI(result.ok)))) : temporal::diplomat::result<std::unique_ptr<temporal::PlainDateTime>, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline temporal::diplomat::result<std::unique_ptr<temporal::PlainDateTime>, temporal::TemporalError> temporal::PlainDateTime::from_parsed(const temporal::ParsedDateTime& parsed) {
    auto result = temporal::capi::temporal_PlainDateTime_from_parsed(parsed.AsFFI());
    return result.is_ok ? temporal::diplomat::result<std::unique_ptr<temporal::PlainDateTime>, temporal::TemporalError>(temporal::diplomat::Ok<std::unique_ptr<temporal::PlainDateTime>>(std::unique_ptr<temporal::PlainDateTime>(temporal::PlainDateTime::FromFFI(result.ok)))) : temporal::diplomat::result<std::unique_ptr<temporal::PlainDateTime>, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline temporal::diplomat::result<std::unique_ptr<temporal::PlainDateTime>, temporal::TemporalError> temporal::PlainDateTime::from_epoch_milliseconds(int64_t ms, temporal::TimeZone tz) {
    auto result = temporal::capi::temporal_PlainDateTime_from_epoch_milliseconds(ms,
        tz.AsFFI());
    return result.is_ok ? temporal::diplomat::result<std::unique_ptr<temporal::PlainDateTime>, temporal::TemporalError>(temporal::diplomat::Ok<std::unique_ptr<temporal::PlainDateTime>>(std::unique_ptr<temporal::PlainDateTime>(temporal::PlainDateTime::FromFFI(result.ok)))) : temporal::diplomat::result<std::unique_ptr<temporal::PlainDateTime>, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline temporal::diplomat::result<std::unique_ptr<temporal::PlainDateTime>, temporal::TemporalError> temporal::PlainDateTime::from_epoch_milliseconds_with_provider(int64_t ms, temporal::TimeZone tz, const temporal::Provider& p) {
    auto result = temporal::capi::temporal_PlainDateTime_from_epoch_milliseconds_with_provider(ms,
        tz.AsFFI(),
        p.AsFFI());
    return result.is_ok ? temporal::diplomat::result<std::unique_ptr<temporal::PlainDateTime>, temporal::TemporalError>(temporal::diplomat::Ok<std::unique_ptr<temporal::PlainDateTime>>(std::unique_ptr<temporal::PlainDateTime>(temporal::PlainDateTime::FromFFI(result.ok)))) : temporal::diplomat::result<std::unique_ptr<temporal::PlainDateTime>, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline temporal::diplomat::result<std::unique_ptr<temporal::PlainDateTime>, temporal::TemporalError> temporal::PlainDateTime::from_epoch_nanoseconds(temporal::I128Nanoseconds ns, temporal::TimeZone tz) {
    auto result = temporal::capi::temporal_PlainDateTime_from_epoch_nanoseconds(ns.AsFFI(),
        tz.AsFFI());
    return result.is_ok ? temporal::diplomat::result<std::unique_ptr<temporal::PlainDateTime>, temporal::TemporalError>(temporal::diplomat::Ok<std::unique_ptr<temporal::PlainDateTime>>(std::unique_ptr<temporal::PlainDateTime>(temporal::PlainDateTime::FromFFI(result.ok)))) : temporal::diplomat::result<std::unique_ptr<temporal::PlainDateTime>, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline temporal::diplomat::result<std::unique_ptr<temporal::PlainDateTime>, temporal::TemporalError> temporal::PlainDateTime::from_epoch_nanoseconds_with_provider(temporal::I128Nanoseconds ns, temporal::TimeZone tz, const temporal::Provider& p) {
    auto result = temporal::capi::temporal_PlainDateTime_from_epoch_nanoseconds_with_provider(ns.AsFFI(),
        tz.AsFFI(),
        p.AsFFI());
    return result.is_ok ? temporal::diplomat::result<std::unique_ptr<temporal::PlainDateTime>, temporal::TemporalError>(temporal::diplomat::Ok<std::unique_ptr<temporal::PlainDateTime>>(std::unique_ptr<temporal::PlainDateTime>(temporal::PlainDateTime::FromFFI(result.ok)))) : temporal::diplomat::result<std::unique_ptr<temporal::PlainDateTime>, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline temporal::diplomat::result<std::unique_ptr<temporal::PlainDateTime>, temporal::TemporalError> temporal::PlainDateTime::with(temporal::PartialDateTime partial, std::optional<temporal::ArithmeticOverflow> overflow) const {
    auto result = temporal::capi::temporal_PlainDateTime_with(this->AsFFI(),
        partial.AsFFI(),
        overflow.has_value() ? (temporal::capi::ArithmeticOverflow_option{ { overflow.value().AsFFI() }, true }) : (temporal::capi::ArithmeticOverflow_option{ {}, false }));
    return result.is_ok ? temporal::diplomat::result<std::unique_ptr<temporal::PlainDateTime>, temporal::TemporalError>(temporal::diplomat::Ok<std::unique_ptr<temporal::PlainDateTime>>(std::unique_ptr<temporal::PlainDateTime>(temporal::PlainDateTime::FromFFI(result.ok)))) : temporal::diplomat::result<std::unique_ptr<temporal::PlainDateTime>, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline temporal::diplomat::result<std::unique_ptr<temporal::PlainDateTime>, temporal::TemporalError> temporal::PlainDateTime::with_time(const temporal::PlainTime* time) const {
    auto result = temporal::capi::temporal_PlainDateTime_with_time(this->AsFFI(),
        time ? time->AsFFI() : nullptr);
    return result.is_ok ? temporal::diplomat::result<std::unique_ptr<temporal::PlainDateTime>, temporal::TemporalError>(temporal::diplomat::Ok<std::unique_ptr<temporal::PlainDateTime>>(std::unique_ptr<temporal::PlainDateTime>(temporal::PlainDateTime::FromFFI(result.ok)))) : temporal::diplomat::result<std::unique_ptr<temporal::PlainDateTime>, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline std::unique_ptr<temporal::PlainDateTime> temporal::PlainDateTime::with_calendar(temporal::AnyCalendarKind calendar) const {
    auto result = temporal::capi::temporal_PlainDateTime_with_calendar(this->AsFFI(),
        calendar.AsFFI());
    return std::unique_ptr<temporal::PlainDateTime>(temporal::PlainDateTime::FromFFI(result));
}

inline temporal::diplomat::result<std::unique_ptr<temporal::PlainDateTime>, temporal::TemporalError> temporal::PlainDateTime::from_utf8(std::string_view s) {
    auto result = temporal::capi::temporal_PlainDateTime_from_utf8({s.data(), s.size()});
    return result.is_ok ? temporal::diplomat::result<std::unique_ptr<temporal::PlainDateTime>, temporal::TemporalError>(temporal::diplomat::Ok<std::unique_ptr<temporal::PlainDateTime>>(std::unique_ptr<temporal::PlainDateTime>(temporal::PlainDateTime::FromFFI(result.ok)))) : temporal::diplomat::result<std::unique_ptr<temporal::PlainDateTime>, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline temporal::diplomat::result<std::unique_ptr<temporal::PlainDateTime>, temporal::TemporalError> temporal::PlainDateTime::from_utf16(std::u16string_view s) {
    auto result = temporal::capi::temporal_PlainDateTime_from_utf16({s.data(), s.size()});
    return result.is_ok ? temporal::diplomat::result<std::unique_ptr<temporal::PlainDateTime>, temporal::TemporalError>(temporal::diplomat::Ok<std::unique_ptr<temporal::PlainDateTime>>(std::unique_ptr<temporal::PlainDateTime>(temporal::PlainDateTime::FromFFI(result.ok)))) : temporal::diplomat::result<std::unique_ptr<temporal::PlainDateTime>, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline uint8_t temporal::PlainDateTime::hour() const {
    auto result = temporal::capi::temporal_PlainDateTime_hour(this->AsFFI());
    return result;
}

inline uint8_t temporal::PlainDateTime::minute() const {
    auto result = temporal::capi::temporal_PlainDateTime_minute(this->AsFFI());
    return result;
}

inline uint8_t temporal::PlainDateTime::second() const {
    auto result = temporal::capi::temporal_PlainDateTime_second(this->AsFFI());
    return result;
}

inline uint16_t temporal::PlainDateTime::millisecond() const {
    auto result = temporal::capi::temporal_PlainDateTime_millisecond(this->AsFFI());
    return result;
}

inline uint16_t temporal::PlainDateTime::microsecond() const {
    auto result = temporal::capi::temporal_PlainDateTime_microsecond(this->AsFFI());
    return result;
}

inline uint16_t temporal::PlainDateTime::nanosecond() const {
    auto result = temporal::capi::temporal_PlainDateTime_nanosecond(this->AsFFI());
    return result;
}

inline const temporal::Calendar& temporal::PlainDateTime::calendar() const DIPLOMAT_LIFETIME_BOUND {
    auto result = temporal::capi::temporal_PlainDateTime_calendar(this->AsFFI());
    return *temporal::Calendar::FromFFI(result);
}

inline int32_t temporal::PlainDateTime::year() const {
    auto result = temporal::capi::temporal_PlainDateTime_year(this->AsFFI());
    return result;
}

inline uint8_t temporal::PlainDateTime::month() const {
    auto result = temporal::capi::temporal_PlainDateTime_month(this->AsFFI());
    return result;
}

inline std::string temporal::PlainDateTime::month_code() const {
    std::string output;
    temporal::diplomat::capi::DiplomatWrite write = temporal::diplomat::WriteFromString(output);
    temporal::capi::temporal_PlainDateTime_month_code(this->AsFFI(),
        &write);
    return output;
}
template<typename W>
inline void temporal::PlainDateTime::month_code_write(W& writeable) const {
    temporal::diplomat::capi::DiplomatWrite write = temporal::diplomat::WriteTrait<W>::Construct(writeable);
    temporal::capi::temporal_PlainDateTime_month_code(this->AsFFI(),
        &write);
}

inline uint8_t temporal::PlainDateTime::day() const {
    auto result = temporal::capi::temporal_PlainDateTime_day(this->AsFFI());
    return result;
}

inline uint16_t temporal::PlainDateTime::day_of_week() const {
    auto result = temporal::capi::temporal_PlainDateTime_day_of_week(this->AsFFI());
    return result;
}

inline uint16_t temporal::PlainDateTime::day_of_year() const {
    auto result = temporal::capi::temporal_PlainDateTime_day_of_year(this->AsFFI());
    return result;
}

inline std::optional<uint8_t> temporal::PlainDateTime::week_of_year() const {
    auto result = temporal::capi::temporal_PlainDateTime_week_of_year(this->AsFFI());
    return result.is_ok ? std::optional<uint8_t>(result.ok) : std::nullopt;
}

inline std::optional<int32_t> temporal::PlainDateTime::year_of_week() const {
    auto result = temporal::capi::temporal_PlainDateTime_year_of_week(this->AsFFI());
    return result.is_ok ? std::optional<int32_t>(result.ok) : std::nullopt;
}

inline uint16_t temporal::PlainDateTime::days_in_week() const {
    auto result = temporal::capi::temporal_PlainDateTime_days_in_week(this->AsFFI());
    return result;
}

inline uint16_t temporal::PlainDateTime::days_in_month() const {
    auto result = temporal::capi::temporal_PlainDateTime_days_in_month(this->AsFFI());
    return result;
}

inline uint16_t temporal::PlainDateTime::days_in_year() const {
    auto result = temporal::capi::temporal_PlainDateTime_days_in_year(this->AsFFI());
    return result;
}

inline uint16_t temporal::PlainDateTime::months_in_year() const {
    auto result = temporal::capi::temporal_PlainDateTime_months_in_year(this->AsFFI());
    return result;
}

inline bool temporal::PlainDateTime::in_leap_year() const {
    auto result = temporal::capi::temporal_PlainDateTime_in_leap_year(this->AsFFI());
    return result;
}

inline std::string temporal::PlainDateTime::era() const {
    std::string output;
    temporal::diplomat::capi::DiplomatWrite write = temporal::diplomat::WriteFromString(output);
    temporal::capi::temporal_PlainDateTime_era(this->AsFFI(),
        &write);
    return output;
}
template<typename W>
inline void temporal::PlainDateTime::era_write(W& writeable) const {
    temporal::diplomat::capi::DiplomatWrite write = temporal::diplomat::WriteTrait<W>::Construct(writeable);
    temporal::capi::temporal_PlainDateTime_era(this->AsFFI(),
        &write);
}

inline std::optional<int32_t> temporal::PlainDateTime::era_year() const {
    auto result = temporal::capi::temporal_PlainDateTime_era_year(this->AsFFI());
    return result.is_ok ? std::optional<int32_t>(result.ok) : std::nullopt;
}

inline temporal::diplomat::result<std::unique_ptr<temporal::PlainDateTime>, temporal::TemporalError> temporal::PlainDateTime::add(const temporal::Duration& duration, std::optional<temporal::ArithmeticOverflow> overflow) const {
    auto result = temporal::capi::temporal_PlainDateTime_add(this->AsFFI(),
        duration.AsFFI(),
        overflow.has_value() ? (temporal::capi::ArithmeticOverflow_option{ { overflow.value().AsFFI() }, true }) : (temporal::capi::ArithmeticOverflow_option{ {}, false }));
    return result.is_ok ? temporal::diplomat::result<std::unique_ptr<temporal::PlainDateTime>, temporal::TemporalError>(temporal::diplomat::Ok<std::unique_ptr<temporal::PlainDateTime>>(std::unique_ptr<temporal::PlainDateTime>(temporal::PlainDateTime::FromFFI(result.ok)))) : temporal::diplomat::result<std::unique_ptr<temporal::PlainDateTime>, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline temporal::diplomat::result<std::unique_ptr<temporal::PlainDateTime>, temporal::TemporalError> temporal::PlainDateTime::subtract(const temporal::Duration& duration, std::optional<temporal::ArithmeticOverflow> overflow) const {
    auto result = temporal::capi::temporal_PlainDateTime_subtract(this->AsFFI(),
        duration.AsFFI(),
        overflow.has_value() ? (temporal::capi::ArithmeticOverflow_option{ { overflow.value().AsFFI() }, true }) : (temporal::capi::ArithmeticOverflow_option{ {}, false }));
    return result.is_ok ? temporal::diplomat::result<std::unique_ptr<temporal::PlainDateTime>, temporal::TemporalError>(temporal::diplomat::Ok<std::unique_ptr<temporal::PlainDateTime>>(std::unique_ptr<temporal::PlainDateTime>(temporal::PlainDateTime::FromFFI(result.ok)))) : temporal::diplomat::result<std::unique_ptr<temporal::PlainDateTime>, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline temporal::diplomat::result<std::unique_ptr<temporal::Duration>, temporal::TemporalError> temporal::PlainDateTime::until(const temporal::PlainDateTime& other, temporal::DifferenceSettings settings) const {
    auto result = temporal::capi::temporal_PlainDateTime_until(this->AsFFI(),
        other.AsFFI(),
        settings.AsFFI());
    return result.is_ok ? temporal::diplomat::result<std::unique_ptr<temporal::Duration>, temporal::TemporalError>(temporal::diplomat::Ok<std::unique_ptr<temporal::Duration>>(std::unique_ptr<temporal::Duration>(temporal::Duration::FromFFI(result.ok)))) : temporal::diplomat::result<std::unique_ptr<temporal::Duration>, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline temporal::diplomat::result<std::unique_ptr<temporal::Duration>, temporal::TemporalError> temporal::PlainDateTime::since(const temporal::PlainDateTime& other, temporal::DifferenceSettings settings) const {
    auto result = temporal::capi::temporal_PlainDateTime_since(this->AsFFI(),
        other.AsFFI(),
        settings.AsFFI());
    return result.is_ok ? temporal::diplomat::result<std::unique_ptr<temporal::Duration>, temporal::TemporalError>(temporal::diplomat::Ok<std::unique_ptr<temporal::Duration>>(std::unique_ptr<temporal::Duration>(temporal::Duration::FromFFI(result.ok)))) : temporal::diplomat::result<std::unique_ptr<temporal::Duration>, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline bool temporal::PlainDateTime::equals(const temporal::PlainDateTime& other) const {
    auto result = temporal::capi::temporal_PlainDateTime_equals(this->AsFFI(),
        other.AsFFI());
    return result;
}

inline int8_t temporal::PlainDateTime::compare(const temporal::PlainDateTime& one, const temporal::PlainDateTime& two) {
    auto result = temporal::capi::temporal_PlainDateTime_compare(one.AsFFI(),
        two.AsFFI());
    return result;
}

inline temporal::diplomat::result<std::unique_ptr<temporal::PlainDateTime>, temporal::TemporalError> temporal::PlainDateTime::round(temporal::RoundingOptions options) const {
    auto result = temporal::capi::temporal_PlainDateTime_round(this->AsFFI(),
        options.AsFFI());
    return result.is_ok ? temporal::diplomat::result<std::unique_ptr<temporal::PlainDateTime>, temporal::TemporalError>(temporal::diplomat::Ok<std::unique_ptr<temporal::PlainDateTime>>(std::unique_ptr<temporal::PlainDateTime>(temporal::PlainDateTime::FromFFI(result.ok)))) : temporal::diplomat::result<std::unique_ptr<temporal::PlainDateTime>, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline std::unique_ptr<temporal::PlainDate> temporal::PlainDateTime::to_plain_date() const {
    auto result = temporal::capi::temporal_PlainDateTime_to_plain_date(this->AsFFI());
    return std::unique_ptr<temporal::PlainDate>(temporal::PlainDate::FromFFI(result));
}

inline std::unique_ptr<temporal::PlainTime> temporal::PlainDateTime::to_plain_time() const {
    auto result = temporal::capi::temporal_PlainDateTime_to_plain_time(this->AsFFI());
    return std::unique_ptr<temporal::PlainTime>(temporal::PlainTime::FromFFI(result));
}

inline temporal::diplomat::result<std::unique_ptr<temporal::ZonedDateTime>, temporal::TemporalError> temporal::PlainDateTime::to_zoned_date_time(temporal::TimeZone time_zone, temporal::Disambiguation disambiguation) const {
    auto result = temporal::capi::temporal_PlainDateTime_to_zoned_date_time(this->AsFFI(),
        time_zone.AsFFI(),
        disambiguation.AsFFI());
    return result.is_ok ? temporal::diplomat::result<std::unique_ptr<temporal::ZonedDateTime>, temporal::TemporalError>(temporal::diplomat::Ok<std::unique_ptr<temporal::ZonedDateTime>>(std::unique_ptr<temporal::ZonedDateTime>(temporal::ZonedDateTime::FromFFI(result.ok)))) : temporal::diplomat::result<std::unique_ptr<temporal::ZonedDateTime>, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline temporal::diplomat::result<std::unique_ptr<temporal::ZonedDateTime>, temporal::TemporalError> temporal::PlainDateTime::to_zoned_date_time_with_provider(temporal::TimeZone time_zone, temporal::Disambiguation disambiguation, const temporal::Provider& p) const {
    auto result = temporal::capi::temporal_PlainDateTime_to_zoned_date_time_with_provider(this->AsFFI(),
        time_zone.AsFFI(),
        disambiguation.AsFFI(),
        p.AsFFI());
    return result.is_ok ? temporal::diplomat::result<std::unique_ptr<temporal::ZonedDateTime>, temporal::TemporalError>(temporal::diplomat::Ok<std::unique_ptr<temporal::ZonedDateTime>>(std::unique_ptr<temporal::ZonedDateTime>(temporal::ZonedDateTime::FromFFI(result.ok)))) : temporal::diplomat::result<std::unique_ptr<temporal::ZonedDateTime>, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline temporal::diplomat::result<int64_t, temporal::TemporalError> temporal::PlainDateTime::epoch_ms_for_utc() const {
    auto result = temporal::capi::temporal_PlainDateTime_epoch_ms_for_utc(this->AsFFI());
    return result.is_ok ? temporal::diplomat::result<int64_t, temporal::TemporalError>(temporal::diplomat::Ok<int64_t>(result.ok)) : temporal::diplomat::result<int64_t, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline temporal::diplomat::result<std::string, temporal::TemporalError> temporal::PlainDateTime::to_ixdtf_string(temporal::ToStringRoundingOptions options, temporal::DisplayCalendar display_calendar) const {
    std::string output;
    temporal::diplomat::capi::DiplomatWrite write = temporal::diplomat::WriteFromString(output);
    auto result = temporal::capi::temporal_PlainDateTime_to_ixdtf_string(this->AsFFI(),
        options.AsFFI(),
        display_calendar.AsFFI(),
        &write);
    return result.is_ok ? temporal::diplomat::result<std::string, temporal::TemporalError>(temporal::diplomat::Ok<std::string>(std::move(output))) : temporal::diplomat::result<std::string, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}
template<typename W>
inline temporal::diplomat::result<std::monostate, temporal::TemporalError> temporal::PlainDateTime::to_ixdtf_string_write(temporal::ToStringRoundingOptions options, temporal::DisplayCalendar display_calendar, W& writeable) const {
    temporal::diplomat::capi::DiplomatWrite write = temporal::diplomat::WriteTrait<W>::Construct(writeable);
    auto result = temporal::capi::temporal_PlainDateTime_to_ixdtf_string(this->AsFFI(),
        options.AsFFI(),
        display_calendar.AsFFI(),
        &write);
    return result.is_ok ? temporal::diplomat::result<std::monostate, temporal::TemporalError>(temporal::diplomat::Ok<std::monostate>()) : temporal::diplomat::result<std::monostate, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline std::unique_ptr<temporal::PlainDateTime> temporal::PlainDateTime::clone() const {
    auto result = temporal::capi::temporal_PlainDateTime_clone(this->AsFFI());
    return std::unique_ptr<temporal::PlainDateTime>(temporal::PlainDateTime::FromFFI(result));
}

inline const temporal::capi::PlainDateTime* temporal::PlainDateTime::AsFFI() const {
    return reinterpret_cast<const temporal::capi::PlainDateTime*>(this);
}

inline temporal::capi::PlainDateTime* temporal::PlainDateTime::AsFFI() {
    return reinterpret_cast<temporal::capi::PlainDateTime*>(this);
}

inline const temporal::PlainDateTime* temporal::PlainDateTime::FromFFI(const temporal::capi::PlainDateTime* ptr) {
    return reinterpret_cast<const temporal::PlainDateTime*>(ptr);
}

inline temporal::PlainDateTime* temporal::PlainDateTime::FromFFI(temporal::capi::PlainDateTime* ptr) {
    return reinterpret_cast<temporal::PlainDateTime*>(ptr);
}

inline void temporal::PlainDateTime::operator delete(void* ptr) {
    temporal::capi::temporal_PlainDateTime_destroy(reinterpret_cast<temporal::capi::PlainDateTime*>(ptr));
}


#endif // TEMPORAL_PlainDateTime_HPP
