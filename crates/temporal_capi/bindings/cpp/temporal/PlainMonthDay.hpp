#ifndef TEMPORAL_PlainMonthDay_HPP
#define TEMPORAL_PlainMonthDay_HPP

#include "PlainMonthDay.d.hpp"

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
#include "DisplayCalendar.hpp"
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

    typedef struct temporal_PlainMonthDay_try_new_with_overflow_result {union {temporal::capi::PlainMonthDay* ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_PlainMonthDay_try_new_with_overflow_result;
    temporal_PlainMonthDay_try_new_with_overflow_result temporal_PlainMonthDay_try_new_with_overflow(uint8_t month, uint8_t day, temporal::capi::AnyCalendarKind calendar, temporal::capi::ArithmeticOverflow overflow, temporal::diplomat::capi::OptionI32 ref_year);

    typedef struct temporal_PlainMonthDay_from_partial_result {union {temporal::capi::PlainMonthDay* ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_PlainMonthDay_from_partial_result;
    temporal_PlainMonthDay_from_partial_result temporal_PlainMonthDay_from_partial(temporal::capi::PartialDate partial, temporal::capi::ArithmeticOverflow_option overflow);

    typedef struct temporal_PlainMonthDay_from_parsed_result {union {temporal::capi::PlainMonthDay* ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_PlainMonthDay_from_parsed_result;
    temporal_PlainMonthDay_from_parsed_result temporal_PlainMonthDay_from_parsed(const temporal::capi::ParsedDate* parsed);

    typedef struct temporal_PlainMonthDay_with_result {union {temporal::capi::PlainMonthDay* ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_PlainMonthDay_with_result;
    temporal_PlainMonthDay_with_result temporal_PlainMonthDay_with(const temporal::capi::PlainMonthDay* self, temporal::capi::PartialDate partial, temporal::capi::ArithmeticOverflow_option overflow);

    bool temporal_PlainMonthDay_equals(const temporal::capi::PlainMonthDay* self, const temporal::capi::PlainMonthDay* other);

    typedef struct temporal_PlainMonthDay_from_utf8_result {union {temporal::capi::PlainMonthDay* ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_PlainMonthDay_from_utf8_result;
    temporal_PlainMonthDay_from_utf8_result temporal_PlainMonthDay_from_utf8(temporal::diplomat::capi::DiplomatStringView s);

    typedef struct temporal_PlainMonthDay_from_utf16_result {union {temporal::capi::PlainMonthDay* ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_PlainMonthDay_from_utf16_result;
    temporal_PlainMonthDay_from_utf16_result temporal_PlainMonthDay_from_utf16(temporal::diplomat::capi::DiplomatString16View s);

    uint8_t temporal_PlainMonthDay_day(const temporal::capi::PlainMonthDay* self);

    const temporal::capi::Calendar* temporal_PlainMonthDay_calendar(const temporal::capi::PlainMonthDay* self);

    uint8_t temporal_PlainMonthDay_month(const temporal::capi::PlainMonthDay* self);

    void temporal_PlainMonthDay_month_code(const temporal::capi::PlainMonthDay* self, temporal::diplomat::capi::DiplomatWrite* write);

    int32_t temporal_PlainMonthDay_reference_year(const temporal::capi::PlainMonthDay* self);

    typedef struct temporal_PlainMonthDay_to_plain_date_result {union {temporal::capi::PlainDate* ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_PlainMonthDay_to_plain_date_result;
    temporal_PlainMonthDay_to_plain_date_result temporal_PlainMonthDay_to_plain_date(const temporal::capi::PlainMonthDay* self, temporal::capi::PartialDate_option year);

    typedef struct temporal_PlainMonthDay_epoch_ms_for_utc_result {union {int64_t ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_PlainMonthDay_epoch_ms_for_utc_result;
    temporal_PlainMonthDay_epoch_ms_for_utc_result temporal_PlainMonthDay_epoch_ms_for_utc(const temporal::capi::PlainMonthDay* self);

    typedef struct temporal_PlainMonthDay_epoch_ms_for_result {union {int64_t ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_PlainMonthDay_epoch_ms_for_result;
    temporal_PlainMonthDay_epoch_ms_for_result temporal_PlainMonthDay_epoch_ms_for(const temporal::capi::PlainMonthDay* self, temporal::capi::TimeZone time_zone);

    typedef struct temporal_PlainMonthDay_epoch_ms_for_with_provider_result {union {int64_t ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_PlainMonthDay_epoch_ms_for_with_provider_result;
    temporal_PlainMonthDay_epoch_ms_for_with_provider_result temporal_PlainMonthDay_epoch_ms_for_with_provider(const temporal::capi::PlainMonthDay* self, temporal::capi::TimeZone time_zone, const temporal::capi::Provider* p);

    void temporal_PlainMonthDay_to_ixdtf_string(const temporal::capi::PlainMonthDay* self, temporal::capi::DisplayCalendar display_calendar, temporal::diplomat::capi::DiplomatWrite* write);

    temporal::capi::PlainMonthDay* temporal_PlainMonthDay_clone(const temporal::capi::PlainMonthDay* self);

    void temporal_PlainMonthDay_destroy(PlainMonthDay* self);

    } // extern "C"
} // namespace capi
} // namespace

inline temporal::diplomat::result<std::unique_ptr<temporal::PlainMonthDay>, temporal::TemporalError> temporal::PlainMonthDay::try_new_with_overflow(uint8_t month, uint8_t day, temporal::AnyCalendarKind calendar, temporal::ArithmeticOverflow overflow, std::optional<int32_t> ref_year) {
    auto result = temporal::capi::temporal_PlainMonthDay_try_new_with_overflow(month,
        day,
        calendar.AsFFI(),
        overflow.AsFFI(),
        ref_year.has_value() ? (temporal::diplomat::capi::OptionI32{ { ref_year.value() }, true }) : (temporal::diplomat::capi::OptionI32{ {}, false }));
    return result.is_ok ? temporal::diplomat::result<std::unique_ptr<temporal::PlainMonthDay>, temporal::TemporalError>(temporal::diplomat::Ok<std::unique_ptr<temporal::PlainMonthDay>>(std::unique_ptr<temporal::PlainMonthDay>(temporal::PlainMonthDay::FromFFI(result.ok)))) : temporal::diplomat::result<std::unique_ptr<temporal::PlainMonthDay>, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline temporal::diplomat::result<std::unique_ptr<temporal::PlainMonthDay>, temporal::TemporalError> temporal::PlainMonthDay::from_partial(temporal::PartialDate partial, std::optional<temporal::ArithmeticOverflow> overflow) {
    auto result = temporal::capi::temporal_PlainMonthDay_from_partial(partial.AsFFI(),
        overflow.has_value() ? (temporal::capi::ArithmeticOverflow_option{ { overflow.value().AsFFI() }, true }) : (temporal::capi::ArithmeticOverflow_option{ {}, false }));
    return result.is_ok ? temporal::diplomat::result<std::unique_ptr<temporal::PlainMonthDay>, temporal::TemporalError>(temporal::diplomat::Ok<std::unique_ptr<temporal::PlainMonthDay>>(std::unique_ptr<temporal::PlainMonthDay>(temporal::PlainMonthDay::FromFFI(result.ok)))) : temporal::diplomat::result<std::unique_ptr<temporal::PlainMonthDay>, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline temporal::diplomat::result<std::unique_ptr<temporal::PlainMonthDay>, temporal::TemporalError> temporal::PlainMonthDay::from_parsed(const temporal::ParsedDate& parsed) {
    auto result = temporal::capi::temporal_PlainMonthDay_from_parsed(parsed.AsFFI());
    return result.is_ok ? temporal::diplomat::result<std::unique_ptr<temporal::PlainMonthDay>, temporal::TemporalError>(temporal::diplomat::Ok<std::unique_ptr<temporal::PlainMonthDay>>(std::unique_ptr<temporal::PlainMonthDay>(temporal::PlainMonthDay::FromFFI(result.ok)))) : temporal::diplomat::result<std::unique_ptr<temporal::PlainMonthDay>, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline temporal::diplomat::result<std::unique_ptr<temporal::PlainMonthDay>, temporal::TemporalError> temporal::PlainMonthDay::with(temporal::PartialDate partial, std::optional<temporal::ArithmeticOverflow> overflow) const {
    auto result = temporal::capi::temporal_PlainMonthDay_with(this->AsFFI(),
        partial.AsFFI(),
        overflow.has_value() ? (temporal::capi::ArithmeticOverflow_option{ { overflow.value().AsFFI() }, true }) : (temporal::capi::ArithmeticOverflow_option{ {}, false }));
    return result.is_ok ? temporal::diplomat::result<std::unique_ptr<temporal::PlainMonthDay>, temporal::TemporalError>(temporal::diplomat::Ok<std::unique_ptr<temporal::PlainMonthDay>>(std::unique_ptr<temporal::PlainMonthDay>(temporal::PlainMonthDay::FromFFI(result.ok)))) : temporal::diplomat::result<std::unique_ptr<temporal::PlainMonthDay>, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline bool temporal::PlainMonthDay::equals(const temporal::PlainMonthDay& other) const {
    auto result = temporal::capi::temporal_PlainMonthDay_equals(this->AsFFI(),
        other.AsFFI());
    return result;
}

inline temporal::diplomat::result<std::unique_ptr<temporal::PlainMonthDay>, temporal::TemporalError> temporal::PlainMonthDay::from_utf8(std::string_view s) {
    auto result = temporal::capi::temporal_PlainMonthDay_from_utf8({s.data(), s.size()});
    return result.is_ok ? temporal::diplomat::result<std::unique_ptr<temporal::PlainMonthDay>, temporal::TemporalError>(temporal::diplomat::Ok<std::unique_ptr<temporal::PlainMonthDay>>(std::unique_ptr<temporal::PlainMonthDay>(temporal::PlainMonthDay::FromFFI(result.ok)))) : temporal::diplomat::result<std::unique_ptr<temporal::PlainMonthDay>, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline temporal::diplomat::result<std::unique_ptr<temporal::PlainMonthDay>, temporal::TemporalError> temporal::PlainMonthDay::from_utf16(std::u16string_view s) {
    auto result = temporal::capi::temporal_PlainMonthDay_from_utf16({s.data(), s.size()});
    return result.is_ok ? temporal::diplomat::result<std::unique_ptr<temporal::PlainMonthDay>, temporal::TemporalError>(temporal::diplomat::Ok<std::unique_ptr<temporal::PlainMonthDay>>(std::unique_ptr<temporal::PlainMonthDay>(temporal::PlainMonthDay::FromFFI(result.ok)))) : temporal::diplomat::result<std::unique_ptr<temporal::PlainMonthDay>, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline uint8_t temporal::PlainMonthDay::day() const {
    auto result = temporal::capi::temporal_PlainMonthDay_day(this->AsFFI());
    return result;
}

inline const temporal::Calendar& temporal::PlainMonthDay::calendar() const DIPLOMAT_LIFETIME_BOUND {
    auto result = temporal::capi::temporal_PlainMonthDay_calendar(this->AsFFI());
    return *temporal::Calendar::FromFFI(result);
}

inline uint8_t temporal::PlainMonthDay::month() const {
    auto result = temporal::capi::temporal_PlainMonthDay_month(this->AsFFI());
    return result;
}

inline std::string temporal::PlainMonthDay::month_code() const {
    std::string output;
    temporal::diplomat::capi::DiplomatWrite write = temporal::diplomat::WriteFromString(output);
    temporal::capi::temporal_PlainMonthDay_month_code(this->AsFFI(),
        &write);
    return output;
}
template<typename W>
inline void temporal::PlainMonthDay::month_code_write(W& writeable) const {
    temporal::diplomat::capi::DiplomatWrite write = temporal::diplomat::WriteTrait<W>::Construct(writeable);
    temporal::capi::temporal_PlainMonthDay_month_code(this->AsFFI(),
        &write);
}

inline int32_t temporal::PlainMonthDay::reference_year() const {
    auto result = temporal::capi::temporal_PlainMonthDay_reference_year(this->AsFFI());
    return result;
}

inline temporal::diplomat::result<std::unique_ptr<temporal::PlainDate>, temporal::TemporalError> temporal::PlainMonthDay::to_plain_date(std::optional<temporal::PartialDate> year) const {
    auto result = temporal::capi::temporal_PlainMonthDay_to_plain_date(this->AsFFI(),
        year.has_value() ? (temporal::capi::PartialDate_option{ { year.value().AsFFI() }, true }) : (temporal::capi::PartialDate_option{ {}, false }));
    return result.is_ok ? temporal::diplomat::result<std::unique_ptr<temporal::PlainDate>, temporal::TemporalError>(temporal::diplomat::Ok<std::unique_ptr<temporal::PlainDate>>(std::unique_ptr<temporal::PlainDate>(temporal::PlainDate::FromFFI(result.ok)))) : temporal::diplomat::result<std::unique_ptr<temporal::PlainDate>, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline temporal::diplomat::result<int64_t, temporal::TemporalError> temporal::PlainMonthDay::epoch_ms_for_utc() const {
    auto result = temporal::capi::temporal_PlainMonthDay_epoch_ms_for_utc(this->AsFFI());
    return result.is_ok ? temporal::diplomat::result<int64_t, temporal::TemporalError>(temporal::diplomat::Ok<int64_t>(result.ok)) : temporal::diplomat::result<int64_t, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline temporal::diplomat::result<int64_t, temporal::TemporalError> temporal::PlainMonthDay::epoch_ms_for(temporal::TimeZone time_zone) const {
    auto result = temporal::capi::temporal_PlainMonthDay_epoch_ms_for(this->AsFFI(),
        time_zone.AsFFI());
    return result.is_ok ? temporal::diplomat::result<int64_t, temporal::TemporalError>(temporal::diplomat::Ok<int64_t>(result.ok)) : temporal::diplomat::result<int64_t, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline temporal::diplomat::result<int64_t, temporal::TemporalError> temporal::PlainMonthDay::epoch_ms_for_with_provider(temporal::TimeZone time_zone, const temporal::Provider& p) const {
    auto result = temporal::capi::temporal_PlainMonthDay_epoch_ms_for_with_provider(this->AsFFI(),
        time_zone.AsFFI(),
        p.AsFFI());
    return result.is_ok ? temporal::diplomat::result<int64_t, temporal::TemporalError>(temporal::diplomat::Ok<int64_t>(result.ok)) : temporal::diplomat::result<int64_t, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline std::string temporal::PlainMonthDay::to_ixdtf_string(temporal::DisplayCalendar display_calendar) const {
    std::string output;
    temporal::diplomat::capi::DiplomatWrite write = temporal::diplomat::WriteFromString(output);
    temporal::capi::temporal_PlainMonthDay_to_ixdtf_string(this->AsFFI(),
        display_calendar.AsFFI(),
        &write);
    return output;
}
template<typename W>
inline void temporal::PlainMonthDay::to_ixdtf_string_write(temporal::DisplayCalendar display_calendar, W& writeable) const {
    temporal::diplomat::capi::DiplomatWrite write = temporal::diplomat::WriteTrait<W>::Construct(writeable);
    temporal::capi::temporal_PlainMonthDay_to_ixdtf_string(this->AsFFI(),
        display_calendar.AsFFI(),
        &write);
}

inline std::unique_ptr<temporal::PlainMonthDay> temporal::PlainMonthDay::clone() const {
    auto result = temporal::capi::temporal_PlainMonthDay_clone(this->AsFFI());
    return std::unique_ptr<temporal::PlainMonthDay>(temporal::PlainMonthDay::FromFFI(result));
}

inline const temporal::capi::PlainMonthDay* temporal::PlainMonthDay::AsFFI() const {
    return reinterpret_cast<const temporal::capi::PlainMonthDay*>(this);
}

inline temporal::capi::PlainMonthDay* temporal::PlainMonthDay::AsFFI() {
    return reinterpret_cast<temporal::capi::PlainMonthDay*>(this);
}

inline const temporal::PlainMonthDay* temporal::PlainMonthDay::FromFFI(const temporal::capi::PlainMonthDay* ptr) {
    return reinterpret_cast<const temporal::PlainMonthDay*>(ptr);
}

inline temporal::PlainMonthDay* temporal::PlainMonthDay::FromFFI(temporal::capi::PlainMonthDay* ptr) {
    return reinterpret_cast<temporal::PlainMonthDay*>(ptr);
}

inline void temporal::PlainMonthDay::operator delete(void* ptr) {
    temporal::capi::temporal_PlainMonthDay_destroy(reinterpret_cast<temporal::capi::PlainMonthDay*>(ptr));
}


#endif // TEMPORAL_PlainMonthDay_HPP
