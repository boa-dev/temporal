#ifndef TEMPORAL_TimeZone_HPP
#define TEMPORAL_TimeZone_HPP

#include "TimeZone.d.hpp"

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

    typedef struct temporal_TimeZone_try_from_identifier_str_result {union {temporal::capi::TimeZone ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_TimeZone_try_from_identifier_str_result;
    temporal_TimeZone_try_from_identifier_str_result temporal_TimeZone_try_from_identifier_str(temporal::diplomat::capi::DiplomatStringView ident);

    typedef struct temporal_TimeZone_try_from_identifier_str_with_provider_result {union {temporal::capi::TimeZone ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_TimeZone_try_from_identifier_str_with_provider_result;
    temporal_TimeZone_try_from_identifier_str_with_provider_result temporal_TimeZone_try_from_identifier_str_with_provider(temporal::diplomat::capi::DiplomatStringView ident, const temporal::capi::Provider* p);

    typedef struct temporal_TimeZone_try_from_offset_str_result {union {temporal::capi::TimeZone ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_TimeZone_try_from_offset_str_result;
    temporal_TimeZone_try_from_offset_str_result temporal_TimeZone_try_from_offset_str(temporal::diplomat::capi::DiplomatStringView ident);

    typedef struct temporal_TimeZone_try_from_str_result {union {temporal::capi::TimeZone ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_TimeZone_try_from_str_result;
    temporal_TimeZone_try_from_str_result temporal_TimeZone_try_from_str(temporal::diplomat::capi::DiplomatStringView ident);

    typedef struct temporal_TimeZone_try_from_str_with_provider_result {union {temporal::capi::TimeZone ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_TimeZone_try_from_str_with_provider_result;
    temporal_TimeZone_try_from_str_with_provider_result temporal_TimeZone_try_from_str_with_provider(temporal::diplomat::capi::DiplomatStringView ident, const temporal::capi::Provider* p);

    void temporal_TimeZone_identifier(temporal::capi::TimeZone self, temporal::diplomat::capi::DiplomatWrite* write);

    typedef struct temporal_TimeZone_identifier_with_provider_result {union { temporal::capi::TemporalError err;}; bool is_ok;} temporal_TimeZone_identifier_with_provider_result;
    temporal_TimeZone_identifier_with_provider_result temporal_TimeZone_identifier_with_provider(temporal::capi::TimeZone self, const temporal::capi::Provider* p, temporal::diplomat::capi::DiplomatWrite* write);

    temporal::capi::TimeZone temporal_TimeZone_utc(void);

    typedef struct temporal_TimeZone_utc_with_provider_result {union {temporal::capi::TimeZone ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_TimeZone_utc_with_provider_result;
    temporal_TimeZone_utc_with_provider_result temporal_TimeZone_utc_with_provider(const temporal::capi::Provider* p);

    temporal::capi::TimeZone temporal_TimeZone_zero(void);

    typedef struct temporal_TimeZone_primary_identifier_result {union {temporal::capi::TimeZone ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_TimeZone_primary_identifier_result;
    temporal_TimeZone_primary_identifier_result temporal_TimeZone_primary_identifier(temporal::capi::TimeZone self);

    typedef struct temporal_TimeZone_primary_identifier_with_provider_result {union {temporal::capi::TimeZone ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_TimeZone_primary_identifier_with_provider_result;
    temporal_TimeZone_primary_identifier_with_provider_result temporal_TimeZone_primary_identifier_with_provider(temporal::capi::TimeZone self, const temporal::capi::Provider* p);

    } // extern "C"
} // namespace capi
} // namespace

inline temporal::diplomat::result<temporal::TimeZone, temporal::TemporalError> temporal::TimeZone::try_from_identifier_str(std::string_view ident) {
    auto result = temporal::capi::temporal_TimeZone_try_from_identifier_str({ident.data(), ident.size()});
    return result.is_ok ? temporal::diplomat::result<temporal::TimeZone, temporal::TemporalError>(temporal::diplomat::Ok<temporal::TimeZone>(temporal::TimeZone::FromFFI(result.ok))) : temporal::diplomat::result<temporal::TimeZone, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline temporal::diplomat::result<temporal::TimeZone, temporal::TemporalError> temporal::TimeZone::try_from_identifier_str_with_provider(std::string_view ident, const temporal::Provider& p) {
    auto result = temporal::capi::temporal_TimeZone_try_from_identifier_str_with_provider({ident.data(), ident.size()},
        p.AsFFI());
    return result.is_ok ? temporal::diplomat::result<temporal::TimeZone, temporal::TemporalError>(temporal::diplomat::Ok<temporal::TimeZone>(temporal::TimeZone::FromFFI(result.ok))) : temporal::diplomat::result<temporal::TimeZone, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline temporal::diplomat::result<temporal::TimeZone, temporal::TemporalError> temporal::TimeZone::try_from_offset_str(std::string_view ident) {
    auto result = temporal::capi::temporal_TimeZone_try_from_offset_str({ident.data(), ident.size()});
    return result.is_ok ? temporal::diplomat::result<temporal::TimeZone, temporal::TemporalError>(temporal::diplomat::Ok<temporal::TimeZone>(temporal::TimeZone::FromFFI(result.ok))) : temporal::diplomat::result<temporal::TimeZone, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline temporal::diplomat::result<temporal::TimeZone, temporal::TemporalError> temporal::TimeZone::try_from_str(std::string_view ident) {
    auto result = temporal::capi::temporal_TimeZone_try_from_str({ident.data(), ident.size()});
    return result.is_ok ? temporal::diplomat::result<temporal::TimeZone, temporal::TemporalError>(temporal::diplomat::Ok<temporal::TimeZone>(temporal::TimeZone::FromFFI(result.ok))) : temporal::diplomat::result<temporal::TimeZone, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline temporal::diplomat::result<temporal::TimeZone, temporal::TemporalError> temporal::TimeZone::try_from_str_with_provider(std::string_view ident, const temporal::Provider& p) {
    auto result = temporal::capi::temporal_TimeZone_try_from_str_with_provider({ident.data(), ident.size()},
        p.AsFFI());
    return result.is_ok ? temporal::diplomat::result<temporal::TimeZone, temporal::TemporalError>(temporal::diplomat::Ok<temporal::TimeZone>(temporal::TimeZone::FromFFI(result.ok))) : temporal::diplomat::result<temporal::TimeZone, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline std::string temporal::TimeZone::identifier() const {
    std::string output;
    temporal::diplomat::capi::DiplomatWrite write = temporal::diplomat::WriteFromString(output);
    temporal::capi::temporal_TimeZone_identifier(this->AsFFI(),
        &write);
    return output;
}
template<typename W>
inline void temporal::TimeZone::identifier_write(W& writeable) const {
    temporal::diplomat::capi::DiplomatWrite write = temporal::diplomat::WriteTrait<W>::Construct(writeable);
    temporal::capi::temporal_TimeZone_identifier(this->AsFFI(),
        &write);
}

inline temporal::diplomat::result<std::string, temporal::TemporalError> temporal::TimeZone::identifier_with_provider(const temporal::Provider& p) const {
    std::string output;
    temporal::diplomat::capi::DiplomatWrite write = temporal::diplomat::WriteFromString(output);
    auto result = temporal::capi::temporal_TimeZone_identifier_with_provider(this->AsFFI(),
        p.AsFFI(),
        &write);
    return result.is_ok ? temporal::diplomat::result<std::string, temporal::TemporalError>(temporal::diplomat::Ok<std::string>(std::move(output))) : temporal::diplomat::result<std::string, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}
template<typename W>
inline temporal::diplomat::result<std::monostate, temporal::TemporalError> temporal::TimeZone::identifier_with_provider_write(const temporal::Provider& p, W& writeable) const {
    temporal::diplomat::capi::DiplomatWrite write = temporal::diplomat::WriteTrait<W>::Construct(writeable);
    auto result = temporal::capi::temporal_TimeZone_identifier_with_provider(this->AsFFI(),
        p.AsFFI(),
        &write);
    return result.is_ok ? temporal::diplomat::result<std::monostate, temporal::TemporalError>(temporal::diplomat::Ok<std::monostate>()) : temporal::diplomat::result<std::monostate, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline temporal::TimeZone temporal::TimeZone::utc() {
    auto result = temporal::capi::temporal_TimeZone_utc();
    return temporal::TimeZone::FromFFI(result);
}

inline temporal::diplomat::result<temporal::TimeZone, temporal::TemporalError> temporal::TimeZone::utc_with_provider(const temporal::Provider& p) {
    auto result = temporal::capi::temporal_TimeZone_utc_with_provider(p.AsFFI());
    return result.is_ok ? temporal::diplomat::result<temporal::TimeZone, temporal::TemporalError>(temporal::diplomat::Ok<temporal::TimeZone>(temporal::TimeZone::FromFFI(result.ok))) : temporal::diplomat::result<temporal::TimeZone, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline temporal::TimeZone temporal::TimeZone::zero() {
    auto result = temporal::capi::temporal_TimeZone_zero();
    return temporal::TimeZone::FromFFI(result);
}

inline temporal::diplomat::result<temporal::TimeZone, temporal::TemporalError> temporal::TimeZone::primary_identifier() const {
    auto result = temporal::capi::temporal_TimeZone_primary_identifier(this->AsFFI());
    return result.is_ok ? temporal::diplomat::result<temporal::TimeZone, temporal::TemporalError>(temporal::diplomat::Ok<temporal::TimeZone>(temporal::TimeZone::FromFFI(result.ok))) : temporal::diplomat::result<temporal::TimeZone, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline temporal::diplomat::result<temporal::TimeZone, temporal::TemporalError> temporal::TimeZone::primary_identifier_with_provider(const temporal::Provider& p) const {
    auto result = temporal::capi::temporal_TimeZone_primary_identifier_with_provider(this->AsFFI(),
        p.AsFFI());
    return result.is_ok ? temporal::diplomat::result<temporal::TimeZone, temporal::TemporalError>(temporal::diplomat::Ok<temporal::TimeZone>(temporal::TimeZone::FromFFI(result.ok))) : temporal::diplomat::result<temporal::TimeZone, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}


inline temporal::capi::TimeZone temporal::TimeZone::AsFFI() const {
    return temporal::capi::TimeZone {
        /* .offset_minutes = */ offset_minutes,
        /* .resolved_id = */ resolved_id,
        /* .normalized_id = */ normalized_id,
        /* .is_iana_id = */ is_iana_id,
    };
}

inline temporal::TimeZone temporal::TimeZone::FromFFI(temporal::capi::TimeZone c_struct) {
    return temporal::TimeZone {
        /* .offset_minutes = */ c_struct.offset_minutes,
        /* .resolved_id = */ c_struct.resolved_id,
        /* .normalized_id = */ c_struct.normalized_id,
        /* .is_iana_id = */ c_struct.is_iana_id,
    };
}


#endif // TEMPORAL_TimeZone_HPP
