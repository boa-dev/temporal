#ifndef TEMPORAL_ParsedDate_HPP
#define TEMPORAL_ParsedDate_HPP

#include "ParsedDate.d.hpp"

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <functional>
#include <optional>
#include <cstdlib>
#include "TemporalError.hpp"
#include "diplomat_runtime.hpp"


namespace temporal {
namespace capi {
    extern "C" {

    typedef struct temporal_ParsedDate_from_utf8_result {union {temporal::capi::ParsedDate* ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_ParsedDate_from_utf8_result;
    temporal_ParsedDate_from_utf8_result temporal_ParsedDate_from_utf8(temporal::diplomat::capi::DiplomatStringView s);

    typedef struct temporal_ParsedDate_from_utf16_result {union {temporal::capi::ParsedDate* ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_ParsedDate_from_utf16_result;
    temporal_ParsedDate_from_utf16_result temporal_ParsedDate_from_utf16(temporal::diplomat::capi::DiplomatString16View s);

    typedef struct temporal_ParsedDate_year_month_from_utf8_result {union {temporal::capi::ParsedDate* ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_ParsedDate_year_month_from_utf8_result;
    temporal_ParsedDate_year_month_from_utf8_result temporal_ParsedDate_year_month_from_utf8(temporal::diplomat::capi::DiplomatStringView s);

    typedef struct temporal_ParsedDate_year_month_from_utf16_result {union {temporal::capi::ParsedDate* ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_ParsedDate_year_month_from_utf16_result;
    temporal_ParsedDate_year_month_from_utf16_result temporal_ParsedDate_year_month_from_utf16(temporal::diplomat::capi::DiplomatString16View s);

    typedef struct temporal_ParsedDate_month_day_from_utf8_result {union {temporal::capi::ParsedDate* ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_ParsedDate_month_day_from_utf8_result;
    temporal_ParsedDate_month_day_from_utf8_result temporal_ParsedDate_month_day_from_utf8(temporal::diplomat::capi::DiplomatStringView s);

    typedef struct temporal_ParsedDate_month_day_from_utf16_result {union {temporal::capi::ParsedDate* ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_ParsedDate_month_day_from_utf16_result;
    temporal_ParsedDate_month_day_from_utf16_result temporal_ParsedDate_month_day_from_utf16(temporal::diplomat::capi::DiplomatString16View s);

    void temporal_ParsedDate_destroy(ParsedDate* self);

    } // extern "C"
} // namespace capi
} // namespace

inline temporal::diplomat::result<std::unique_ptr<temporal::ParsedDate>, temporal::TemporalError> temporal::ParsedDate::from_utf8(std::string_view s) {
    auto result = temporal::capi::temporal_ParsedDate_from_utf8({s.data(), s.size()});
    return result.is_ok ? temporal::diplomat::result<std::unique_ptr<temporal::ParsedDate>, temporal::TemporalError>(temporal::diplomat::Ok<std::unique_ptr<temporal::ParsedDate>>(std::unique_ptr<temporal::ParsedDate>(temporal::ParsedDate::FromFFI(result.ok)))) : temporal::diplomat::result<std::unique_ptr<temporal::ParsedDate>, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline temporal::diplomat::result<std::unique_ptr<temporal::ParsedDate>, temporal::TemporalError> temporal::ParsedDate::from_utf16(std::u16string_view s) {
    auto result = temporal::capi::temporal_ParsedDate_from_utf16({s.data(), s.size()});
    return result.is_ok ? temporal::diplomat::result<std::unique_ptr<temporal::ParsedDate>, temporal::TemporalError>(temporal::diplomat::Ok<std::unique_ptr<temporal::ParsedDate>>(std::unique_ptr<temporal::ParsedDate>(temporal::ParsedDate::FromFFI(result.ok)))) : temporal::diplomat::result<std::unique_ptr<temporal::ParsedDate>, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline temporal::diplomat::result<std::unique_ptr<temporal::ParsedDate>, temporal::TemporalError> temporal::ParsedDate::year_month_from_utf8(std::string_view s) {
    auto result = temporal::capi::temporal_ParsedDate_year_month_from_utf8({s.data(), s.size()});
    return result.is_ok ? temporal::diplomat::result<std::unique_ptr<temporal::ParsedDate>, temporal::TemporalError>(temporal::diplomat::Ok<std::unique_ptr<temporal::ParsedDate>>(std::unique_ptr<temporal::ParsedDate>(temporal::ParsedDate::FromFFI(result.ok)))) : temporal::diplomat::result<std::unique_ptr<temporal::ParsedDate>, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline temporal::diplomat::result<std::unique_ptr<temporal::ParsedDate>, temporal::TemporalError> temporal::ParsedDate::year_month_from_utf16(std::u16string_view s) {
    auto result = temporal::capi::temporal_ParsedDate_year_month_from_utf16({s.data(), s.size()});
    return result.is_ok ? temporal::diplomat::result<std::unique_ptr<temporal::ParsedDate>, temporal::TemporalError>(temporal::diplomat::Ok<std::unique_ptr<temporal::ParsedDate>>(std::unique_ptr<temporal::ParsedDate>(temporal::ParsedDate::FromFFI(result.ok)))) : temporal::diplomat::result<std::unique_ptr<temporal::ParsedDate>, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline temporal::diplomat::result<std::unique_ptr<temporal::ParsedDate>, temporal::TemporalError> temporal::ParsedDate::month_day_from_utf8(std::string_view s) {
    auto result = temporal::capi::temporal_ParsedDate_month_day_from_utf8({s.data(), s.size()});
    return result.is_ok ? temporal::diplomat::result<std::unique_ptr<temporal::ParsedDate>, temporal::TemporalError>(temporal::diplomat::Ok<std::unique_ptr<temporal::ParsedDate>>(std::unique_ptr<temporal::ParsedDate>(temporal::ParsedDate::FromFFI(result.ok)))) : temporal::diplomat::result<std::unique_ptr<temporal::ParsedDate>, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline temporal::diplomat::result<std::unique_ptr<temporal::ParsedDate>, temporal::TemporalError> temporal::ParsedDate::month_day_from_utf16(std::u16string_view s) {
    auto result = temporal::capi::temporal_ParsedDate_month_day_from_utf16({s.data(), s.size()});
    return result.is_ok ? temporal::diplomat::result<std::unique_ptr<temporal::ParsedDate>, temporal::TemporalError>(temporal::diplomat::Ok<std::unique_ptr<temporal::ParsedDate>>(std::unique_ptr<temporal::ParsedDate>(temporal::ParsedDate::FromFFI(result.ok)))) : temporal::diplomat::result<std::unique_ptr<temporal::ParsedDate>, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline const temporal::capi::ParsedDate* temporal::ParsedDate::AsFFI() const {
    return reinterpret_cast<const temporal::capi::ParsedDate*>(this);
}

inline temporal::capi::ParsedDate* temporal::ParsedDate::AsFFI() {
    return reinterpret_cast<temporal::capi::ParsedDate*>(this);
}

inline const temporal::ParsedDate* temporal::ParsedDate::FromFFI(const temporal::capi::ParsedDate* ptr) {
    return reinterpret_cast<const temporal::ParsedDate*>(ptr);
}

inline temporal::ParsedDate* temporal::ParsedDate::FromFFI(temporal::capi::ParsedDate* ptr) {
    return reinterpret_cast<temporal::ParsedDate*>(ptr);
}

inline void temporal::ParsedDate::operator delete(void* ptr) {
    temporal::capi::temporal_ParsedDate_destroy(reinterpret_cast<temporal::capi::ParsedDate*>(ptr));
}


#endif // TEMPORAL_ParsedDate_HPP
