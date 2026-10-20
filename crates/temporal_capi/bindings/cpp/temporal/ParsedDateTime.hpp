#ifndef TEMPORAL_ParsedDateTime_HPP
#define TEMPORAL_ParsedDateTime_HPP

#include "ParsedDateTime.d.hpp"

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

    typedef struct temporal_ParsedDateTime_from_utf8_result {union {temporal::capi::ParsedDateTime* ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_ParsedDateTime_from_utf8_result;
    temporal_ParsedDateTime_from_utf8_result temporal_ParsedDateTime_from_utf8(temporal::diplomat::capi::DiplomatStringView s);

    typedef struct temporal_ParsedDateTime_from_utf16_result {union {temporal::capi::ParsedDateTime* ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_ParsedDateTime_from_utf16_result;
    temporal_ParsedDateTime_from_utf16_result temporal_ParsedDateTime_from_utf16(temporal::diplomat::capi::DiplomatString16View s);

    void temporal_ParsedDateTime_destroy(ParsedDateTime* self);

    } // extern "C"
} // namespace capi
} // namespace

inline temporal::diplomat::result<std::unique_ptr<temporal::ParsedDateTime>, temporal::TemporalError> temporal::ParsedDateTime::from_utf8(std::string_view s) {
    auto result = temporal::capi::temporal_ParsedDateTime_from_utf8({s.data(), s.size()});
    return result.is_ok ? temporal::diplomat::result<std::unique_ptr<temporal::ParsedDateTime>, temporal::TemporalError>(temporal::diplomat::Ok<std::unique_ptr<temporal::ParsedDateTime>>(std::unique_ptr<temporal::ParsedDateTime>(temporal::ParsedDateTime::FromFFI(result.ok)))) : temporal::diplomat::result<std::unique_ptr<temporal::ParsedDateTime>, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline temporal::diplomat::result<std::unique_ptr<temporal::ParsedDateTime>, temporal::TemporalError> temporal::ParsedDateTime::from_utf16(std::u16string_view s) {
    auto result = temporal::capi::temporal_ParsedDateTime_from_utf16({s.data(), s.size()});
    return result.is_ok ? temporal::diplomat::result<std::unique_ptr<temporal::ParsedDateTime>, temporal::TemporalError>(temporal::diplomat::Ok<std::unique_ptr<temporal::ParsedDateTime>>(std::unique_ptr<temporal::ParsedDateTime>(temporal::ParsedDateTime::FromFFI(result.ok)))) : temporal::diplomat::result<std::unique_ptr<temporal::ParsedDateTime>, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline const temporal::capi::ParsedDateTime* temporal::ParsedDateTime::AsFFI() const {
    return reinterpret_cast<const temporal::capi::ParsedDateTime*>(this);
}

inline temporal::capi::ParsedDateTime* temporal::ParsedDateTime::AsFFI() {
    return reinterpret_cast<temporal::capi::ParsedDateTime*>(this);
}

inline const temporal::ParsedDateTime* temporal::ParsedDateTime::FromFFI(const temporal::capi::ParsedDateTime* ptr) {
    return reinterpret_cast<const temporal::ParsedDateTime*>(ptr);
}

inline temporal::ParsedDateTime* temporal::ParsedDateTime::FromFFI(temporal::capi::ParsedDateTime* ptr) {
    return reinterpret_cast<temporal::ParsedDateTime*>(ptr);
}

inline void temporal::ParsedDateTime::operator delete(void* ptr) {
    temporal::capi::temporal_ParsedDateTime_destroy(reinterpret_cast<temporal::capi::ParsedDateTime*>(ptr));
}


#endif // TEMPORAL_ParsedDateTime_HPP
