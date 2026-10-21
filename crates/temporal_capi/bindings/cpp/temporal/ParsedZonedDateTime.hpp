#ifndef TEMPORAL_ParsedZonedDateTime_HPP
#define TEMPORAL_ParsedZonedDateTime_HPP

#include "ParsedZonedDateTime.d.hpp"

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <functional>
#include <optional>
#include <cstdlib>
#include "Provider.hpp"
#include "TemporalError.hpp"
#include "diplomat_runtime.hpp"


namespace temporal {
namespace capi {
    extern "C" {

    typedef struct temporal_ParsedZonedDateTime_from_utf8_result {union {temporal::capi::ParsedZonedDateTime* ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_ParsedZonedDateTime_from_utf8_result;
    temporal_ParsedZonedDateTime_from_utf8_result temporal_ParsedZonedDateTime_from_utf8(temporal::diplomat::capi::DiplomatStringView s);

    typedef struct temporal_ParsedZonedDateTime_from_utf8_with_provider_result {union {temporal::capi::ParsedZonedDateTime* ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_ParsedZonedDateTime_from_utf8_with_provider_result;
    temporal_ParsedZonedDateTime_from_utf8_with_provider_result temporal_ParsedZonedDateTime_from_utf8_with_provider(temporal::diplomat::capi::DiplomatStringView s, const temporal::capi::Provider* p);

    typedef struct temporal_ParsedZonedDateTime_from_utf16_result {union {temporal::capi::ParsedZonedDateTime* ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_ParsedZonedDateTime_from_utf16_result;
    temporal_ParsedZonedDateTime_from_utf16_result temporal_ParsedZonedDateTime_from_utf16(temporal::diplomat::capi::DiplomatString16View s);

    typedef struct temporal_ParsedZonedDateTime_from_utf16_with_provider_result {union {temporal::capi::ParsedZonedDateTime* ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_ParsedZonedDateTime_from_utf16_with_provider_result;
    temporal_ParsedZonedDateTime_from_utf16_with_provider_result temporal_ParsedZonedDateTime_from_utf16_with_provider(temporal::diplomat::capi::DiplomatString16View s, const temporal::capi::Provider* p);

    void temporal_ParsedZonedDateTime_destroy(ParsedZonedDateTime* self);

    } // extern "C"
} // namespace capi
} // namespace

inline temporal::diplomat::result<std::unique_ptr<temporal::ParsedZonedDateTime>, temporal::TemporalError> temporal::ParsedZonedDateTime::from_utf8(std::string_view s) {
    auto result = temporal::capi::temporal_ParsedZonedDateTime_from_utf8({s.data(), s.size()});
    return result.is_ok ? temporal::diplomat::result<std::unique_ptr<temporal::ParsedZonedDateTime>, temporal::TemporalError>(temporal::diplomat::Ok<std::unique_ptr<temporal::ParsedZonedDateTime>>(std::unique_ptr<temporal::ParsedZonedDateTime>(temporal::ParsedZonedDateTime::FromFFI(result.ok)))) : temporal::diplomat::result<std::unique_ptr<temporal::ParsedZonedDateTime>, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline temporal::diplomat::result<std::unique_ptr<temporal::ParsedZonedDateTime>, temporal::TemporalError> temporal::ParsedZonedDateTime::from_utf8_with_provider(std::string_view s, const temporal::Provider& p) {
    auto result = temporal::capi::temporal_ParsedZonedDateTime_from_utf8_with_provider({s.data(), s.size()},
        p.AsFFI());
    return result.is_ok ? temporal::diplomat::result<std::unique_ptr<temporal::ParsedZonedDateTime>, temporal::TemporalError>(temporal::diplomat::Ok<std::unique_ptr<temporal::ParsedZonedDateTime>>(std::unique_ptr<temporal::ParsedZonedDateTime>(temporal::ParsedZonedDateTime::FromFFI(result.ok)))) : temporal::diplomat::result<std::unique_ptr<temporal::ParsedZonedDateTime>, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline temporal::diplomat::result<std::unique_ptr<temporal::ParsedZonedDateTime>, temporal::TemporalError> temporal::ParsedZonedDateTime::from_utf16(std::u16string_view s) {
    auto result = temporal::capi::temporal_ParsedZonedDateTime_from_utf16({s.data(), s.size()});
    return result.is_ok ? temporal::diplomat::result<std::unique_ptr<temporal::ParsedZonedDateTime>, temporal::TemporalError>(temporal::diplomat::Ok<std::unique_ptr<temporal::ParsedZonedDateTime>>(std::unique_ptr<temporal::ParsedZonedDateTime>(temporal::ParsedZonedDateTime::FromFFI(result.ok)))) : temporal::diplomat::result<std::unique_ptr<temporal::ParsedZonedDateTime>, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline temporal::diplomat::result<std::unique_ptr<temporal::ParsedZonedDateTime>, temporal::TemporalError> temporal::ParsedZonedDateTime::from_utf16_with_provider(std::u16string_view s, const temporal::Provider& p) {
    auto result = temporal::capi::temporal_ParsedZonedDateTime_from_utf16_with_provider({s.data(), s.size()},
        p.AsFFI());
    return result.is_ok ? temporal::diplomat::result<std::unique_ptr<temporal::ParsedZonedDateTime>, temporal::TemporalError>(temporal::diplomat::Ok<std::unique_ptr<temporal::ParsedZonedDateTime>>(std::unique_ptr<temporal::ParsedZonedDateTime>(temporal::ParsedZonedDateTime::FromFFI(result.ok)))) : temporal::diplomat::result<std::unique_ptr<temporal::ParsedZonedDateTime>, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline const temporal::capi::ParsedZonedDateTime* temporal::ParsedZonedDateTime::AsFFI() const {
    return reinterpret_cast<const temporal::capi::ParsedZonedDateTime*>(this);
}

inline temporal::capi::ParsedZonedDateTime* temporal::ParsedZonedDateTime::AsFFI() {
    return reinterpret_cast<temporal::capi::ParsedZonedDateTime*>(this);
}

inline const temporal::ParsedZonedDateTime* temporal::ParsedZonedDateTime::FromFFI(const temporal::capi::ParsedZonedDateTime* ptr) {
    return reinterpret_cast<const temporal::ParsedZonedDateTime*>(ptr);
}

inline temporal::ParsedZonedDateTime* temporal::ParsedZonedDateTime::FromFFI(temporal::capi::ParsedZonedDateTime* ptr) {
    return reinterpret_cast<temporal::ParsedZonedDateTime*>(ptr);
}

inline void temporal::ParsedZonedDateTime::operator delete(void* ptr) {
    temporal::capi::temporal_ParsedZonedDateTime_destroy(reinterpret_cast<temporal::capi::ParsedZonedDateTime*>(ptr));
}


#endif // TEMPORAL_ParsedZonedDateTime_HPP
