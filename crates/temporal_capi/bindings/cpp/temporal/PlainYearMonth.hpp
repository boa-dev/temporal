#ifndef TEMPORAL_PlainYearMonth_HPP
#define TEMPORAL_PlainYearMonth_HPP

#include "PlainYearMonth.d.hpp"

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
#include "ParsedDate.hpp"
#include "PartialDate.hpp"
#include "PlainDate.hpp"
#include "Provider.hpp"
#include "TemporalError.hpp"
#include "TimeZone.hpp"
#include "diplomat_runtime.hpp"


namespace temporal {
namespace capi {
    extern "C" {

    typedef struct temporal_PlainYearMonth_try_new_with_overflow_result {union {temporal::capi::PlainYearMonth* ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_PlainYearMonth_try_new_with_overflow_result;
    temporal_PlainYearMonth_try_new_with_overflow_result temporal_PlainYearMonth_try_new_with_overflow(int32_t year, uint8_t month, temporal::diplomat::capi::OptionU8 reference_day, temporal::capi::AnyCalendarKind calendar, temporal::capi::ArithmeticOverflow overflow);

    typedef struct temporal_PlainYearMonth_from_partial_result {union {temporal::capi::PlainYearMonth* ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_PlainYearMonth_from_partial_result;
    temporal_PlainYearMonth_from_partial_result temporal_PlainYearMonth_from_partial(temporal::capi::PartialDate partial, temporal::capi::ArithmeticOverflow_option overflow);

    typedef struct temporal_PlainYearMonth_from_parsed_result {union {temporal::capi::PlainYearMonth* ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_PlainYearMonth_from_parsed_result;
    temporal_PlainYearMonth_from_parsed_result temporal_PlainYearMonth_from_parsed(const temporal::capi::ParsedDate* parsed);

    typedef struct temporal_PlainYearMonth_with_result {union {temporal::capi::PlainYearMonth* ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_PlainYearMonth_with_result;
    temporal_PlainYearMonth_with_result temporal_PlainYearMonth_with(const temporal::capi::PlainYearMonth* self, temporal::capi::PartialDate partial, temporal::capi::ArithmeticOverflow_option overflow);

    typedef struct temporal_PlainYearMonth_from_utf8_result {union {temporal::capi::PlainYearMonth* ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_PlainYearMonth_from_utf8_result;
    temporal_PlainYearMonth_from_utf8_result temporal_PlainYearMonth_from_utf8(temporal::diplomat::capi::DiplomatStringView s);

    typedef struct temporal_PlainYearMonth_from_utf16_result {union {temporal::capi::PlainYearMonth* ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_PlainYearMonth_from_utf16_result;
    temporal_PlainYearMonth_from_utf16_result temporal_PlainYearMonth_from_utf16(temporal::diplomat::capi::DiplomatString16View s);

    int32_t temporal_PlainYearMonth_year(const temporal::capi::PlainYearMonth* self);

    uint8_t temporal_PlainYearMonth_month(const temporal::capi::PlainYearMonth* self);

    void temporal_PlainYearMonth_month_code(const temporal::capi::PlainYearMonth* self, temporal::diplomat::capi::DiplomatWrite* write);

    bool temporal_PlainYearMonth_in_leap_year(const temporal::capi::PlainYearMonth* self);

    uint16_t temporal_PlainYearMonth_days_in_month(const temporal::capi::PlainYearMonth* self);

    uint16_t temporal_PlainYearMonth_days_in_year(const temporal::capi::PlainYearMonth* self);

    uint16_t temporal_PlainYearMonth_months_in_year(const temporal::capi::PlainYearMonth* self);

    void temporal_PlainYearMonth_era(const temporal::capi::PlainYearMonth* self, temporal::diplomat::capi::DiplomatWrite* write);

    typedef struct temporal_PlainYearMonth_era_year_result {union {int32_t ok; }; bool is_ok;} temporal_PlainYearMonth_era_year_result;
    temporal_PlainYearMonth_era_year_result temporal_PlainYearMonth_era_year(const temporal::capi::PlainYearMonth* self);

    uint8_t temporal_PlainYearMonth_reference_day(const temporal::capi::PlainYearMonth* self);

    const temporal::capi::Calendar* temporal_PlainYearMonth_calendar(const temporal::capi::PlainYearMonth* self);

    typedef struct temporal_PlainYearMonth_add_result {union {temporal::capi::PlainYearMonth* ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_PlainYearMonth_add_result;
    temporal_PlainYearMonth_add_result temporal_PlainYearMonth_add(const temporal::capi::PlainYearMonth* self, const temporal::capi::Duration* duration, temporal::capi::ArithmeticOverflow overflow);

    typedef struct temporal_PlainYearMonth_subtract_result {union {temporal::capi::PlainYearMonth* ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_PlainYearMonth_subtract_result;
    temporal_PlainYearMonth_subtract_result temporal_PlainYearMonth_subtract(const temporal::capi::PlainYearMonth* self, const temporal::capi::Duration* duration, temporal::capi::ArithmeticOverflow overflow);

    typedef struct temporal_PlainYearMonth_until_result {union {temporal::capi::Duration* ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_PlainYearMonth_until_result;
    temporal_PlainYearMonth_until_result temporal_PlainYearMonth_until(const temporal::capi::PlainYearMonth* self, const temporal::capi::PlainYearMonth* other, temporal::capi::DifferenceSettings settings);

    typedef struct temporal_PlainYearMonth_since_result {union {temporal::capi::Duration* ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_PlainYearMonth_since_result;
    temporal_PlainYearMonth_since_result temporal_PlainYearMonth_since(const temporal::capi::PlainYearMonth* self, const temporal::capi::PlainYearMonth* other, temporal::capi::DifferenceSettings settings);

    bool temporal_PlainYearMonth_equals(const temporal::capi::PlainYearMonth* self, const temporal::capi::PlainYearMonth* other);

    int8_t temporal_PlainYearMonth_compare(const temporal::capi::PlainYearMonth* one, const temporal::capi::PlainYearMonth* two);

    typedef struct temporal_PlainYearMonth_to_plain_date_result {union {temporal::capi::PlainDate* ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_PlainYearMonth_to_plain_date_result;
    temporal_PlainYearMonth_to_plain_date_result temporal_PlainYearMonth_to_plain_date(const temporal::capi::PlainYearMonth* self, temporal::capi::PartialDate_option day);

    typedef struct temporal_PlainYearMonth_epoch_ms_for_utc_result {union {int64_t ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_PlainYearMonth_epoch_ms_for_utc_result;
    temporal_PlainYearMonth_epoch_ms_for_utc_result temporal_PlainYearMonth_epoch_ms_for_utc(const temporal::capi::PlainYearMonth* self);

    typedef struct temporal_PlainYearMonth_epoch_ms_for_result {union {int64_t ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_PlainYearMonth_epoch_ms_for_result;
    temporal_PlainYearMonth_epoch_ms_for_result temporal_PlainYearMonth_epoch_ms_for(const temporal::capi::PlainYearMonth* self, temporal::capi::TimeZone time_zone);

    typedef struct temporal_PlainYearMonth_epoch_ms_for_with_provider_result {union {int64_t ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_PlainYearMonth_epoch_ms_for_with_provider_result;
    temporal_PlainYearMonth_epoch_ms_for_with_provider_result temporal_PlainYearMonth_epoch_ms_for_with_provider(const temporal::capi::PlainYearMonth* self, temporal::capi::TimeZone time_zone, const temporal::capi::Provider* p);

    void temporal_PlainYearMonth_to_ixdtf_string(const temporal::capi::PlainYearMonth* self, temporal::capi::DisplayCalendar display_calendar, temporal::diplomat::capi::DiplomatWrite* write);

    temporal::capi::PlainYearMonth* temporal_PlainYearMonth_clone(const temporal::capi::PlainYearMonth* self);

    void temporal_PlainYearMonth_destroy(PlainYearMonth* self);

    } // extern "C"
} // namespace capi
} // namespace

inline temporal::diplomat::result<std::unique_ptr<temporal::PlainYearMonth>, temporal::TemporalError> temporal::PlainYearMonth::try_new_with_overflow(int32_t year, uint8_t month, std::optional<uint8_t> reference_day, temporal::AnyCalendarKind calendar, temporal::ArithmeticOverflow overflow) {
    auto result = temporal::capi::temporal_PlainYearMonth_try_new_with_overflow(year,
        month,
        reference_day.has_value() ? (temporal::diplomat::capi::OptionU8{ { reference_day.value() }, true }) : (temporal::diplomat::capi::OptionU8{ {}, false }),
        calendar.AsFFI(),
        overflow.AsFFI());
    return result.is_ok ? temporal::diplomat::result<std::unique_ptr<temporal::PlainYearMonth>, temporal::TemporalError>(temporal::diplomat::Ok<std::unique_ptr<temporal::PlainYearMonth>>(std::unique_ptr<temporal::PlainYearMonth>(temporal::PlainYearMonth::FromFFI(result.ok)))) : temporal::diplomat::result<std::unique_ptr<temporal::PlainYearMonth>, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline temporal::diplomat::result<std::unique_ptr<temporal::PlainYearMonth>, temporal::TemporalError> temporal::PlainYearMonth::from_partial(temporal::PartialDate partial, std::optional<temporal::ArithmeticOverflow> overflow) {
    auto result = temporal::capi::temporal_PlainYearMonth_from_partial(partial.AsFFI(),
        overflow.has_value() ? (temporal::capi::ArithmeticOverflow_option{ { overflow.value().AsFFI() }, true }) : (temporal::capi::ArithmeticOverflow_option{ {}, false }));
    return result.is_ok ? temporal::diplomat::result<std::unique_ptr<temporal::PlainYearMonth>, temporal::TemporalError>(temporal::diplomat::Ok<std::unique_ptr<temporal::PlainYearMonth>>(std::unique_ptr<temporal::PlainYearMonth>(temporal::PlainYearMonth::FromFFI(result.ok)))) : temporal::diplomat::result<std::unique_ptr<temporal::PlainYearMonth>, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline temporal::diplomat::result<std::unique_ptr<temporal::PlainYearMonth>, temporal::TemporalError> temporal::PlainYearMonth::from_parsed(const temporal::ParsedDate& parsed) {
    auto result = temporal::capi::temporal_PlainYearMonth_from_parsed(parsed.AsFFI());
    return result.is_ok ? temporal::diplomat::result<std::unique_ptr<temporal::PlainYearMonth>, temporal::TemporalError>(temporal::diplomat::Ok<std::unique_ptr<temporal::PlainYearMonth>>(std::unique_ptr<temporal::PlainYearMonth>(temporal::PlainYearMonth::FromFFI(result.ok)))) : temporal::diplomat::result<std::unique_ptr<temporal::PlainYearMonth>, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline temporal::diplomat::result<std::unique_ptr<temporal::PlainYearMonth>, temporal::TemporalError> temporal::PlainYearMonth::with(temporal::PartialDate partial, std::optional<temporal::ArithmeticOverflow> overflow) const {
    auto result = temporal::capi::temporal_PlainYearMonth_with(this->AsFFI(),
        partial.AsFFI(),
        overflow.has_value() ? (temporal::capi::ArithmeticOverflow_option{ { overflow.value().AsFFI() }, true }) : (temporal::capi::ArithmeticOverflow_option{ {}, false }));
    return result.is_ok ? temporal::diplomat::result<std::unique_ptr<temporal::PlainYearMonth>, temporal::TemporalError>(temporal::diplomat::Ok<std::unique_ptr<temporal::PlainYearMonth>>(std::unique_ptr<temporal::PlainYearMonth>(temporal::PlainYearMonth::FromFFI(result.ok)))) : temporal::diplomat::result<std::unique_ptr<temporal::PlainYearMonth>, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline temporal::diplomat::result<std::unique_ptr<temporal::PlainYearMonth>, temporal::TemporalError> temporal::PlainYearMonth::from_utf8(std::string_view s) {
    auto result = temporal::capi::temporal_PlainYearMonth_from_utf8({s.data(), s.size()});
    return result.is_ok ? temporal::diplomat::result<std::unique_ptr<temporal::PlainYearMonth>, temporal::TemporalError>(temporal::diplomat::Ok<std::unique_ptr<temporal::PlainYearMonth>>(std::unique_ptr<temporal::PlainYearMonth>(temporal::PlainYearMonth::FromFFI(result.ok)))) : temporal::diplomat::result<std::unique_ptr<temporal::PlainYearMonth>, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline temporal::diplomat::result<std::unique_ptr<temporal::PlainYearMonth>, temporal::TemporalError> temporal::PlainYearMonth::from_utf16(std::u16string_view s) {
    auto result = temporal::capi::temporal_PlainYearMonth_from_utf16({s.data(), s.size()});
    return result.is_ok ? temporal::diplomat::result<std::unique_ptr<temporal::PlainYearMonth>, temporal::TemporalError>(temporal::diplomat::Ok<std::unique_ptr<temporal::PlainYearMonth>>(std::unique_ptr<temporal::PlainYearMonth>(temporal::PlainYearMonth::FromFFI(result.ok)))) : temporal::diplomat::result<std::unique_ptr<temporal::PlainYearMonth>, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline int32_t temporal::PlainYearMonth::year() const {
    auto result = temporal::capi::temporal_PlainYearMonth_year(this->AsFFI());
    return result;
}

inline uint8_t temporal::PlainYearMonth::month() const {
    auto result = temporal::capi::temporal_PlainYearMonth_month(this->AsFFI());
    return result;
}

inline std::string temporal::PlainYearMonth::month_code() const {
    std::string output;
    temporal::diplomat::capi::DiplomatWrite write = temporal::diplomat::WriteFromString(output);
    temporal::capi::temporal_PlainYearMonth_month_code(this->AsFFI(),
        &write);
    return output;
}
template<typename W>
inline void temporal::PlainYearMonth::month_code_write(W& writeable) const {
    temporal::diplomat::capi::DiplomatWrite write = temporal::diplomat::WriteTrait<W>::Construct(writeable);
    temporal::capi::temporal_PlainYearMonth_month_code(this->AsFFI(),
        &write);
}

inline bool temporal::PlainYearMonth::in_leap_year() const {
    auto result = temporal::capi::temporal_PlainYearMonth_in_leap_year(this->AsFFI());
    return result;
}

inline uint16_t temporal::PlainYearMonth::days_in_month() const {
    auto result = temporal::capi::temporal_PlainYearMonth_days_in_month(this->AsFFI());
    return result;
}

inline uint16_t temporal::PlainYearMonth::days_in_year() const {
    auto result = temporal::capi::temporal_PlainYearMonth_days_in_year(this->AsFFI());
    return result;
}

inline uint16_t temporal::PlainYearMonth::months_in_year() const {
    auto result = temporal::capi::temporal_PlainYearMonth_months_in_year(this->AsFFI());
    return result;
}

inline std::string temporal::PlainYearMonth::era() const {
    std::string output;
    temporal::diplomat::capi::DiplomatWrite write = temporal::diplomat::WriteFromString(output);
    temporal::capi::temporal_PlainYearMonth_era(this->AsFFI(),
        &write);
    return output;
}
template<typename W>
inline void temporal::PlainYearMonth::era_write(W& writeable) const {
    temporal::diplomat::capi::DiplomatWrite write = temporal::diplomat::WriteTrait<W>::Construct(writeable);
    temporal::capi::temporal_PlainYearMonth_era(this->AsFFI(),
        &write);
}

inline std::optional<int32_t> temporal::PlainYearMonth::era_year() const {
    auto result = temporal::capi::temporal_PlainYearMonth_era_year(this->AsFFI());
    return result.is_ok ? std::optional<int32_t>(result.ok) : std::nullopt;
}

inline uint8_t temporal::PlainYearMonth::reference_day() const {
    auto result = temporal::capi::temporal_PlainYearMonth_reference_day(this->AsFFI());
    return result;
}

inline const temporal::Calendar& temporal::PlainYearMonth::calendar() const DIPLOMAT_LIFETIME_BOUND {
    auto result = temporal::capi::temporal_PlainYearMonth_calendar(this->AsFFI());
    return *temporal::Calendar::FromFFI(result);
}

inline temporal::diplomat::result<std::unique_ptr<temporal::PlainYearMonth>, temporal::TemporalError> temporal::PlainYearMonth::add(const temporal::Duration& duration, temporal::ArithmeticOverflow overflow) const {
    auto result = temporal::capi::temporal_PlainYearMonth_add(this->AsFFI(),
        duration.AsFFI(),
        overflow.AsFFI());
    return result.is_ok ? temporal::diplomat::result<std::unique_ptr<temporal::PlainYearMonth>, temporal::TemporalError>(temporal::diplomat::Ok<std::unique_ptr<temporal::PlainYearMonth>>(std::unique_ptr<temporal::PlainYearMonth>(temporal::PlainYearMonth::FromFFI(result.ok)))) : temporal::diplomat::result<std::unique_ptr<temporal::PlainYearMonth>, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline temporal::diplomat::result<std::unique_ptr<temporal::PlainYearMonth>, temporal::TemporalError> temporal::PlainYearMonth::subtract(const temporal::Duration& duration, temporal::ArithmeticOverflow overflow) const {
    auto result = temporal::capi::temporal_PlainYearMonth_subtract(this->AsFFI(),
        duration.AsFFI(),
        overflow.AsFFI());
    return result.is_ok ? temporal::diplomat::result<std::unique_ptr<temporal::PlainYearMonth>, temporal::TemporalError>(temporal::diplomat::Ok<std::unique_ptr<temporal::PlainYearMonth>>(std::unique_ptr<temporal::PlainYearMonth>(temporal::PlainYearMonth::FromFFI(result.ok)))) : temporal::diplomat::result<std::unique_ptr<temporal::PlainYearMonth>, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline temporal::diplomat::result<std::unique_ptr<temporal::Duration>, temporal::TemporalError> temporal::PlainYearMonth::until(const temporal::PlainYearMonth& other, temporal::DifferenceSettings settings) const {
    auto result = temporal::capi::temporal_PlainYearMonth_until(this->AsFFI(),
        other.AsFFI(),
        settings.AsFFI());
    return result.is_ok ? temporal::diplomat::result<std::unique_ptr<temporal::Duration>, temporal::TemporalError>(temporal::diplomat::Ok<std::unique_ptr<temporal::Duration>>(std::unique_ptr<temporal::Duration>(temporal::Duration::FromFFI(result.ok)))) : temporal::diplomat::result<std::unique_ptr<temporal::Duration>, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline temporal::diplomat::result<std::unique_ptr<temporal::Duration>, temporal::TemporalError> temporal::PlainYearMonth::since(const temporal::PlainYearMonth& other, temporal::DifferenceSettings settings) const {
    auto result = temporal::capi::temporal_PlainYearMonth_since(this->AsFFI(),
        other.AsFFI(),
        settings.AsFFI());
    return result.is_ok ? temporal::diplomat::result<std::unique_ptr<temporal::Duration>, temporal::TemporalError>(temporal::diplomat::Ok<std::unique_ptr<temporal::Duration>>(std::unique_ptr<temporal::Duration>(temporal::Duration::FromFFI(result.ok)))) : temporal::diplomat::result<std::unique_ptr<temporal::Duration>, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline bool temporal::PlainYearMonth::equals(const temporal::PlainYearMonth& other) const {
    auto result = temporal::capi::temporal_PlainYearMonth_equals(this->AsFFI(),
        other.AsFFI());
    return result;
}

inline int8_t temporal::PlainYearMonth::compare(const temporal::PlainYearMonth& one, const temporal::PlainYearMonth& two) {
    auto result = temporal::capi::temporal_PlainYearMonth_compare(one.AsFFI(),
        two.AsFFI());
    return result;
}

inline temporal::diplomat::result<std::unique_ptr<temporal::PlainDate>, temporal::TemporalError> temporal::PlainYearMonth::to_plain_date(std::optional<temporal::PartialDate> day) const {
    auto result = temporal::capi::temporal_PlainYearMonth_to_plain_date(this->AsFFI(),
        day.has_value() ? (temporal::capi::PartialDate_option{ { day.value().AsFFI() }, true }) : (temporal::capi::PartialDate_option{ {}, false }));
    return result.is_ok ? temporal::diplomat::result<std::unique_ptr<temporal::PlainDate>, temporal::TemporalError>(temporal::diplomat::Ok<std::unique_ptr<temporal::PlainDate>>(std::unique_ptr<temporal::PlainDate>(temporal::PlainDate::FromFFI(result.ok)))) : temporal::diplomat::result<std::unique_ptr<temporal::PlainDate>, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline temporal::diplomat::result<int64_t, temporal::TemporalError> temporal::PlainYearMonth::epoch_ms_for_utc() const {
    auto result = temporal::capi::temporal_PlainYearMonth_epoch_ms_for_utc(this->AsFFI());
    return result.is_ok ? temporal::diplomat::result<int64_t, temporal::TemporalError>(temporal::diplomat::Ok<int64_t>(result.ok)) : temporal::diplomat::result<int64_t, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline temporal::diplomat::result<int64_t, temporal::TemporalError> temporal::PlainYearMonth::epoch_ms_for(temporal::TimeZone time_zone) const {
    auto result = temporal::capi::temporal_PlainYearMonth_epoch_ms_for(this->AsFFI(),
        time_zone.AsFFI());
    return result.is_ok ? temporal::diplomat::result<int64_t, temporal::TemporalError>(temporal::diplomat::Ok<int64_t>(result.ok)) : temporal::diplomat::result<int64_t, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline temporal::diplomat::result<int64_t, temporal::TemporalError> temporal::PlainYearMonth::epoch_ms_for_with_provider(temporal::TimeZone time_zone, const temporal::Provider& p) const {
    auto result = temporal::capi::temporal_PlainYearMonth_epoch_ms_for_with_provider(this->AsFFI(),
        time_zone.AsFFI(),
        p.AsFFI());
    return result.is_ok ? temporal::diplomat::result<int64_t, temporal::TemporalError>(temporal::diplomat::Ok<int64_t>(result.ok)) : temporal::diplomat::result<int64_t, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline std::string temporal::PlainYearMonth::to_ixdtf_string(temporal::DisplayCalendar display_calendar) const {
    std::string output;
    temporal::diplomat::capi::DiplomatWrite write = temporal::diplomat::WriteFromString(output);
    temporal::capi::temporal_PlainYearMonth_to_ixdtf_string(this->AsFFI(),
        display_calendar.AsFFI(),
        &write);
    return output;
}
template<typename W>
inline void temporal::PlainYearMonth::to_ixdtf_string_write(temporal::DisplayCalendar display_calendar, W& writeable) const {
    temporal::diplomat::capi::DiplomatWrite write = temporal::diplomat::WriteTrait<W>::Construct(writeable);
    temporal::capi::temporal_PlainYearMonth_to_ixdtf_string(this->AsFFI(),
        display_calendar.AsFFI(),
        &write);
}

inline std::unique_ptr<temporal::PlainYearMonth> temporal::PlainYearMonth::clone() const {
    auto result = temporal::capi::temporal_PlainYearMonth_clone(this->AsFFI());
    return std::unique_ptr<temporal::PlainYearMonth>(temporal::PlainYearMonth::FromFFI(result));
}

inline const temporal::capi::PlainYearMonth* temporal::PlainYearMonth::AsFFI() const {
    return reinterpret_cast<const temporal::capi::PlainYearMonth*>(this);
}

inline temporal::capi::PlainYearMonth* temporal::PlainYearMonth::AsFFI() {
    return reinterpret_cast<temporal::capi::PlainYearMonth*>(this);
}

inline const temporal::PlainYearMonth* temporal::PlainYearMonth::FromFFI(const temporal::capi::PlainYearMonth* ptr) {
    return reinterpret_cast<const temporal::PlainYearMonth*>(ptr);
}

inline temporal::PlainYearMonth* temporal::PlainYearMonth::FromFFI(temporal::capi::PlainYearMonth* ptr) {
    return reinterpret_cast<temporal::PlainYearMonth*>(ptr);
}

inline void temporal::PlainYearMonth::operator delete(void* ptr) {
    temporal::capi::temporal_PlainYearMonth_destroy(reinterpret_cast<temporal::capi::PlainYearMonth*>(ptr));
}


#endif // TEMPORAL_PlainYearMonth_HPP
