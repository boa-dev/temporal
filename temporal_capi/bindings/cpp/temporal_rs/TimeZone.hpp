#ifndef temporal_rs_TimeZone_HPP
#define temporal_rs_TimeZone_HPP

#include "TimeZone.d.hpp"

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <functional>
#include <optional>
#include <cstdlib>
#include "../diplomat_runtime.hpp"
#include "Provider.hpp"
#include "TemporalError.hpp"


namespace temporal_rs {
namespace capi {
    extern "C" {

    typedef struct temporal_rs_TimeZone_try_from_identifier_str_result {union {temporal_rs::capi::TimeZone ok; temporal_rs::capi::TemporalError err;}; bool is_ok;} temporal_rs_TimeZone_try_from_identifier_str_result;
    temporal_rs_TimeZone_try_from_identifier_str_result temporal_rs_TimeZone_try_from_identifier_str(diplomat::capi::DiplomatStringView ident);

    typedef struct temporal_rs_TimeZone_try_from_identifier_str_with_provider_result {union {temporal_rs::capi::TimeZone ok; temporal_rs::capi::TemporalError err;}; bool is_ok;} temporal_rs_TimeZone_try_from_identifier_str_with_provider_result;
    temporal_rs_TimeZone_try_from_identifier_str_with_provider_result temporal_rs_TimeZone_try_from_identifier_str_with_provider(diplomat::capi::DiplomatStringView ident, const temporal_rs::capi::Provider* p);

    typedef struct temporal_rs_TimeZone_try_from_offset_str_result {union {temporal_rs::capi::TimeZone ok; temporal_rs::capi::TemporalError err;}; bool is_ok;} temporal_rs_TimeZone_try_from_offset_str_result;
    temporal_rs_TimeZone_try_from_offset_str_result temporal_rs_TimeZone_try_from_offset_str(diplomat::capi::DiplomatStringView ident);

    typedef struct temporal_rs_TimeZone_try_from_str_result {union {temporal_rs::capi::TimeZone ok; temporal_rs::capi::TemporalError err;}; bool is_ok;} temporal_rs_TimeZone_try_from_str_result;
    temporal_rs_TimeZone_try_from_str_result temporal_rs_TimeZone_try_from_str(diplomat::capi::DiplomatStringView ident);

    typedef struct temporal_rs_TimeZone_try_from_str_with_provider_result {union {temporal_rs::capi::TimeZone ok; temporal_rs::capi::TemporalError err;}; bool is_ok;} temporal_rs_TimeZone_try_from_str_with_provider_result;
    temporal_rs_TimeZone_try_from_str_with_provider_result temporal_rs_TimeZone_try_from_str_with_provider(diplomat::capi::DiplomatStringView ident, const temporal_rs::capi::Provider* p);

    void temporal_rs_TimeZone_identifier(temporal_rs::capi::TimeZone self, diplomat::capi::DiplomatWrite* write);

    typedef struct temporal_rs_TimeZone_identifier_with_provider_result {union { temporal_rs::capi::TemporalError err;}; bool is_ok;} temporal_rs_TimeZone_identifier_with_provider_result;
    temporal_rs_TimeZone_identifier_with_provider_result temporal_rs_TimeZone_identifier_with_provider(temporal_rs::capi::TimeZone self, const temporal_rs::capi::Provider* p, diplomat::capi::DiplomatWrite* write);

    temporal_rs::capi::TimeZone temporal_rs_TimeZone_utc(void);

    typedef struct temporal_rs_TimeZone_utc_with_provider_result {union {temporal_rs::capi::TimeZone ok; temporal_rs::capi::TemporalError err;}; bool is_ok;} temporal_rs_TimeZone_utc_with_provider_result;
    temporal_rs_TimeZone_utc_with_provider_result temporal_rs_TimeZone_utc_with_provider(const temporal_rs::capi::Provider* p);

    temporal_rs::capi::TimeZone temporal_rs_TimeZone_zero(void);

    typedef struct temporal_rs_TimeZone_primary_identifier_result {union {temporal_rs::capi::TimeZone ok; temporal_rs::capi::TemporalError err;}; bool is_ok;} temporal_rs_TimeZone_primary_identifier_result;
    temporal_rs_TimeZone_primary_identifier_result temporal_rs_TimeZone_primary_identifier(temporal_rs::capi::TimeZone self);

    typedef struct temporal_rs_TimeZone_primary_identifier_with_provider_result {union {temporal_rs::capi::TimeZone ok; temporal_rs::capi::TemporalError err;}; bool is_ok;} temporal_rs_TimeZone_primary_identifier_with_provider_result;
    temporal_rs_TimeZone_primary_identifier_with_provider_result temporal_rs_TimeZone_primary_identifier_with_provider(temporal_rs::capi::TimeZone self, const temporal_rs::capi::Provider* p);

    } // extern "C"
} // namespace capi
} // namespace

inline diplomat::result<temporal_rs::TimeZone, temporal_rs::TemporalError> temporal_rs::TimeZone::try_from_identifier_str(std::string_view ident) {
  auto result = temporal_rs::capi::temporal_rs_TimeZone_try_from_identifier_str({ident.data(), ident.size()});
  return result.is_ok ? diplomat::result<temporal_rs::TimeZone, temporal_rs::TemporalError>(diplomat::Ok<temporal_rs::TimeZone>(temporal_rs::TimeZone::FromFFI(result.ok))) : diplomat::result<temporal_rs::TimeZone, temporal_rs::TemporalError>(diplomat::Err<temporal_rs::TemporalError>(temporal_rs::TemporalError::FromFFI(result.err)));
}

inline diplomat::result<temporal_rs::TimeZone, temporal_rs::TemporalError> temporal_rs::TimeZone::try_from_identifier_str_with_provider(std::string_view ident, const temporal_rs::Provider& p) {
  auto result = temporal_rs::capi::temporal_rs_TimeZone_try_from_identifier_str_with_provider({ident.data(), ident.size()},
    p.AsFFI());
  return result.is_ok ? diplomat::result<temporal_rs::TimeZone, temporal_rs::TemporalError>(diplomat::Ok<temporal_rs::TimeZone>(temporal_rs::TimeZone::FromFFI(result.ok))) : diplomat::result<temporal_rs::TimeZone, temporal_rs::TemporalError>(diplomat::Err<temporal_rs::TemporalError>(temporal_rs::TemporalError::FromFFI(result.err)));
}

inline diplomat::result<temporal_rs::TimeZone, temporal_rs::TemporalError> temporal_rs::TimeZone::try_from_offset_str(std::string_view ident) {
  auto result = temporal_rs::capi::temporal_rs_TimeZone_try_from_offset_str({ident.data(), ident.size()});
  return result.is_ok ? diplomat::result<temporal_rs::TimeZone, temporal_rs::TemporalError>(diplomat::Ok<temporal_rs::TimeZone>(temporal_rs::TimeZone::FromFFI(result.ok))) : diplomat::result<temporal_rs::TimeZone, temporal_rs::TemporalError>(diplomat::Err<temporal_rs::TemporalError>(temporal_rs::TemporalError::FromFFI(result.err)));
}

inline diplomat::result<temporal_rs::TimeZone, temporal_rs::TemporalError> temporal_rs::TimeZone::try_from_str(std::string_view ident) {
  auto result = temporal_rs::capi::temporal_rs_TimeZone_try_from_str({ident.data(), ident.size()});
  return result.is_ok ? diplomat::result<temporal_rs::TimeZone, temporal_rs::TemporalError>(diplomat::Ok<temporal_rs::TimeZone>(temporal_rs::TimeZone::FromFFI(result.ok))) : diplomat::result<temporal_rs::TimeZone, temporal_rs::TemporalError>(diplomat::Err<temporal_rs::TemporalError>(temporal_rs::TemporalError::FromFFI(result.err)));
}

inline diplomat::result<temporal_rs::TimeZone, temporal_rs::TemporalError> temporal_rs::TimeZone::try_from_str_with_provider(std::string_view ident, const temporal_rs::Provider& p) {
  auto result = temporal_rs::capi::temporal_rs_TimeZone_try_from_str_with_provider({ident.data(), ident.size()},
    p.AsFFI());
  return result.is_ok ? diplomat::result<temporal_rs::TimeZone, temporal_rs::TemporalError>(diplomat::Ok<temporal_rs::TimeZone>(temporal_rs::TimeZone::FromFFI(result.ok))) : diplomat::result<temporal_rs::TimeZone, temporal_rs::TemporalError>(diplomat::Err<temporal_rs::TemporalError>(temporal_rs::TemporalError::FromFFI(result.err)));
}

inline std::string temporal_rs::TimeZone::identifier() const {
  std::string output;
  diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
  temporal_rs::capi::temporal_rs_TimeZone_identifier(this->AsFFI(),
    &write);
  return output;
}
template<typename W>
inline void temporal_rs::TimeZone::identifier_write(W& writeable) const {
  diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
  temporal_rs::capi::temporal_rs_TimeZone_identifier(this->AsFFI(),
    &write);
}

inline diplomat::result<std::string, temporal_rs::TemporalError> temporal_rs::TimeZone::identifier_with_provider(const temporal_rs::Provider& p) const {
  std::string output;
  diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
  auto result = temporal_rs::capi::temporal_rs_TimeZone_identifier_with_provider(this->AsFFI(),
    p.AsFFI(),
    &write);
  return result.is_ok ? diplomat::result<std::string, temporal_rs::TemporalError>(diplomat::Ok<std::string>(std::move(output))) : diplomat::result<std::string, temporal_rs::TemporalError>(diplomat::Err<temporal_rs::TemporalError>(temporal_rs::TemporalError::FromFFI(result.err)));
}
template<typename W>
inline diplomat::result<std::monostate, temporal_rs::TemporalError> temporal_rs::TimeZone::identifier_with_provider_write(const temporal_rs::Provider& p, W& writeable) const {
  diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
  auto result = temporal_rs::capi::temporal_rs_TimeZone_identifier_with_provider(this->AsFFI(),
    p.AsFFI(),
    &write);
  return result.is_ok ? diplomat::result<std::monostate, temporal_rs::TemporalError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, temporal_rs::TemporalError>(diplomat::Err<temporal_rs::TemporalError>(temporal_rs::TemporalError::FromFFI(result.err)));
}

inline temporal_rs::TimeZone temporal_rs::TimeZone::utc() {
  auto result = temporal_rs::capi::temporal_rs_TimeZone_utc();
  return temporal_rs::TimeZone::FromFFI(result);
}

inline diplomat::result<temporal_rs::TimeZone, temporal_rs::TemporalError> temporal_rs::TimeZone::utc_with_provider(const temporal_rs::Provider& p) {
  auto result = temporal_rs::capi::temporal_rs_TimeZone_utc_with_provider(p.AsFFI());
  return result.is_ok ? diplomat::result<temporal_rs::TimeZone, temporal_rs::TemporalError>(diplomat::Ok<temporal_rs::TimeZone>(temporal_rs::TimeZone::FromFFI(result.ok))) : diplomat::result<temporal_rs::TimeZone, temporal_rs::TemporalError>(diplomat::Err<temporal_rs::TemporalError>(temporal_rs::TemporalError::FromFFI(result.err)));
}

inline temporal_rs::TimeZone temporal_rs::TimeZone::zero() {
  auto result = temporal_rs::capi::temporal_rs_TimeZone_zero();
  return temporal_rs::TimeZone::FromFFI(result);
}

inline diplomat::result<temporal_rs::TimeZone, temporal_rs::TemporalError> temporal_rs::TimeZone::primary_identifier() const {
  auto result = temporal_rs::capi::temporal_rs_TimeZone_primary_identifier(this->AsFFI());
  return result.is_ok ? diplomat::result<temporal_rs::TimeZone, temporal_rs::TemporalError>(diplomat::Ok<temporal_rs::TimeZone>(temporal_rs::TimeZone::FromFFI(result.ok))) : diplomat::result<temporal_rs::TimeZone, temporal_rs::TemporalError>(diplomat::Err<temporal_rs::TemporalError>(temporal_rs::TemporalError::FromFFI(result.err)));
}

inline diplomat::result<temporal_rs::TimeZone, temporal_rs::TemporalError> temporal_rs::TimeZone::primary_identifier_with_provider(const temporal_rs::Provider& p) const {
  auto result = temporal_rs::capi::temporal_rs_TimeZone_primary_identifier_with_provider(this->AsFFI(),
    p.AsFFI());
  return result.is_ok ? diplomat::result<temporal_rs::TimeZone, temporal_rs::TemporalError>(diplomat::Ok<temporal_rs::TimeZone>(temporal_rs::TimeZone::FromFFI(result.ok))) : diplomat::result<temporal_rs::TimeZone, temporal_rs::TemporalError>(diplomat::Err<temporal_rs::TemporalError>(temporal_rs::TemporalError::FromFFI(result.err)));
}


inline temporal_rs::capi::TimeZone temporal_rs::TimeZone::AsFFI() const {
  return temporal_rs::capi::TimeZone {
    /* .offset_minutes = */ offset_minutes,
    /* .resolved_id = */ resolved_id,
    /* .normalized_id = */ normalized_id,
    /* .is_iana_id = */ is_iana_id,
  };
}

inline temporal_rs::TimeZone temporal_rs::TimeZone::FromFFI(temporal_rs::capi::TimeZone c_struct) {
  return temporal_rs::TimeZone {
    /* .offset_minutes = */ c_struct.offset_minutes,
    /* .resolved_id = */ c_struct.resolved_id,
    /* .normalized_id = */ c_struct.normalized_id,
    /* .is_iana_id = */ c_struct.is_iana_id,
  };
}


#endif // temporal_rs_TimeZone_HPP
