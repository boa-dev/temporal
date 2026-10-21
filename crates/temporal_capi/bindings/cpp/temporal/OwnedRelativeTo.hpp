#ifndef TEMPORAL_OwnedRelativeTo_HPP
#define TEMPORAL_OwnedRelativeTo_HPP

#include "OwnedRelativeTo.d.hpp"

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <functional>
#include <optional>
#include <cstdlib>
#include "PlainDate.hpp"
#include "Provider.hpp"
#include "TemporalError.hpp"
#include "ZonedDateTime.hpp"
#include "diplomat_runtime.hpp"


namespace temporal {
namespace capi {
    extern "C" {

    typedef struct temporal_OwnedRelativeTo_from_utf8_result {union {temporal::capi::OwnedRelativeTo ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_OwnedRelativeTo_from_utf8_result;
    temporal_OwnedRelativeTo_from_utf8_result temporal_OwnedRelativeTo_from_utf8(temporal::diplomat::capi::DiplomatStringView s);

    typedef struct temporal_OwnedRelativeTo_from_utf8_with_provider_result {union {temporal::capi::OwnedRelativeTo ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_OwnedRelativeTo_from_utf8_with_provider_result;
    temporal_OwnedRelativeTo_from_utf8_with_provider_result temporal_OwnedRelativeTo_from_utf8_with_provider(temporal::diplomat::capi::DiplomatStringView s, const temporal::capi::Provider* p);

    typedef struct temporal_OwnedRelativeTo_from_utf16_result {union {temporal::capi::OwnedRelativeTo ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_OwnedRelativeTo_from_utf16_result;
    temporal_OwnedRelativeTo_from_utf16_result temporal_OwnedRelativeTo_from_utf16(temporal::diplomat::capi::DiplomatString16View s);

    typedef struct temporal_OwnedRelativeTo_from_utf16_with_provider_result {union {temporal::capi::OwnedRelativeTo ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_OwnedRelativeTo_from_utf16_with_provider_result;
    temporal_OwnedRelativeTo_from_utf16_with_provider_result temporal_OwnedRelativeTo_from_utf16_with_provider(temporal::diplomat::capi::DiplomatString16View s, const temporal::capi::Provider* p);

    temporal::capi::OwnedRelativeTo temporal_OwnedRelativeTo_empty(void);

    } // extern "C"
} // namespace capi
} // namespace

inline temporal::diplomat::result<temporal::OwnedRelativeTo, temporal::TemporalError> temporal::OwnedRelativeTo::from_utf8(std::string_view s) {
    auto result = temporal::capi::temporal_OwnedRelativeTo_from_utf8({s.data(), s.size()});
    return result.is_ok ? temporal::diplomat::result<temporal::OwnedRelativeTo, temporal::TemporalError>(temporal::diplomat::Ok<temporal::OwnedRelativeTo>(temporal::OwnedRelativeTo::FromFFI(result.ok))) : temporal::diplomat::result<temporal::OwnedRelativeTo, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline temporal::diplomat::result<temporal::OwnedRelativeTo, temporal::TemporalError> temporal::OwnedRelativeTo::from_utf8_with_provider(std::string_view s, const temporal::Provider& p) {
    auto result = temporal::capi::temporal_OwnedRelativeTo_from_utf8_with_provider({s.data(), s.size()},
        p.AsFFI());
    return result.is_ok ? temporal::diplomat::result<temporal::OwnedRelativeTo, temporal::TemporalError>(temporal::diplomat::Ok<temporal::OwnedRelativeTo>(temporal::OwnedRelativeTo::FromFFI(result.ok))) : temporal::diplomat::result<temporal::OwnedRelativeTo, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline temporal::diplomat::result<temporal::OwnedRelativeTo, temporal::TemporalError> temporal::OwnedRelativeTo::from_utf16(std::u16string_view s) {
    auto result = temporal::capi::temporal_OwnedRelativeTo_from_utf16({s.data(), s.size()});
    return result.is_ok ? temporal::diplomat::result<temporal::OwnedRelativeTo, temporal::TemporalError>(temporal::diplomat::Ok<temporal::OwnedRelativeTo>(temporal::OwnedRelativeTo::FromFFI(result.ok))) : temporal::diplomat::result<temporal::OwnedRelativeTo, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline temporal::diplomat::result<temporal::OwnedRelativeTo, temporal::TemporalError> temporal::OwnedRelativeTo::from_utf16_with_provider(std::u16string_view s, const temporal::Provider& p) {
    auto result = temporal::capi::temporal_OwnedRelativeTo_from_utf16_with_provider({s.data(), s.size()},
        p.AsFFI());
    return result.is_ok ? temporal::diplomat::result<temporal::OwnedRelativeTo, temporal::TemporalError>(temporal::diplomat::Ok<temporal::OwnedRelativeTo>(temporal::OwnedRelativeTo::FromFFI(result.ok))) : temporal::diplomat::result<temporal::OwnedRelativeTo, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline temporal::OwnedRelativeTo temporal::OwnedRelativeTo::empty() {
    auto result = temporal::capi::temporal_OwnedRelativeTo_empty();
    return temporal::OwnedRelativeTo::FromFFI(result);
}


inline temporal::capi::OwnedRelativeTo temporal::OwnedRelativeTo::AsFFI() const {
    return temporal::capi::OwnedRelativeTo {
        /* .date = */ date ? date->AsFFI() : nullptr,
        /* .zoned = */ zoned ? zoned->AsFFI() : nullptr,
    };
}

inline temporal::OwnedRelativeTo temporal::OwnedRelativeTo::FromFFI(temporal::capi::OwnedRelativeTo c_struct) {
    return temporal::OwnedRelativeTo {
        /* .date = */ std::unique_ptr<temporal::PlainDate>(temporal::PlainDate::FromFFI(c_struct.date)),
        /* .zoned = */ std::unique_ptr<temporal::ZonedDateTime>(temporal::ZonedDateTime::FromFFI(c_struct.zoned)),
    };
}


#endif // TEMPORAL_OwnedRelativeTo_HPP
