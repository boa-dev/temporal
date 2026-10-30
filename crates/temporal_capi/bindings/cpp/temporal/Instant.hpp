#ifndef TEMPORAL_Instant_HPP
#define TEMPORAL_Instant_HPP

#include "Instant.d.hpp"

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <functional>
#include <optional>
#include <cstdlib>
#include "DifferenceSettings.hpp"
#include "Duration.hpp"
#include "I128Nanoseconds.hpp"
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

    typedef struct temporal_Instant_try_new_result {union {temporal::capi::Instant* ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_Instant_try_new_result;
    temporal_Instant_try_new_result temporal_Instant_try_new(temporal::capi::I128Nanoseconds ns);

    typedef struct temporal_Instant_from_epoch_milliseconds_result {union {temporal::capi::Instant* ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_Instant_from_epoch_milliseconds_result;
    temporal_Instant_from_epoch_milliseconds_result temporal_Instant_from_epoch_milliseconds(int64_t epoch_milliseconds);

    typedef struct temporal_Instant_from_utf8_result {union {temporal::capi::Instant* ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_Instant_from_utf8_result;
    temporal_Instant_from_utf8_result temporal_Instant_from_utf8(temporal::diplomat::capi::DiplomatStringView s);

    typedef struct temporal_Instant_from_utf16_result {union {temporal::capi::Instant* ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_Instant_from_utf16_result;
    temporal_Instant_from_utf16_result temporal_Instant_from_utf16(temporal::diplomat::capi::DiplomatString16View s);

    typedef struct temporal_Instant_add_result {union {temporal::capi::Instant* ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_Instant_add_result;
    temporal_Instant_add_result temporal_Instant_add(const temporal::capi::Instant* self, const temporal::capi::Duration* duration);

    typedef struct temporal_Instant_subtract_result {union {temporal::capi::Instant* ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_Instant_subtract_result;
    temporal_Instant_subtract_result temporal_Instant_subtract(const temporal::capi::Instant* self, const temporal::capi::Duration* duration);

    typedef struct temporal_Instant_since_result {union {temporal::capi::Duration* ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_Instant_since_result;
    temporal_Instant_since_result temporal_Instant_since(const temporal::capi::Instant* self, const temporal::capi::Instant* other, temporal::capi::DifferenceSettings settings);

    typedef struct temporal_Instant_until_result {union {temporal::capi::Duration* ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_Instant_until_result;
    temporal_Instant_until_result temporal_Instant_until(const temporal::capi::Instant* self, const temporal::capi::Instant* other, temporal::capi::DifferenceSettings settings);

    typedef struct temporal_Instant_round_result {union {temporal::capi::Instant* ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_Instant_round_result;
    temporal_Instant_round_result temporal_Instant_round(const temporal::capi::Instant* self, temporal::capi::RoundingOptions options);

    int8_t temporal_Instant_compare(const temporal::capi::Instant* self, const temporal::capi::Instant* other);

    bool temporal_Instant_equals(const temporal::capi::Instant* self, const temporal::capi::Instant* other);

    int64_t temporal_Instant_epoch_milliseconds(const temporal::capi::Instant* self);

    temporal::capi::I128Nanoseconds temporal_Instant_epoch_nanoseconds(const temporal::capi::Instant* self);

    typedef struct temporal_Instant_to_ixdtf_string_with_compiled_data_result {union { temporal::capi::TemporalError err;}; bool is_ok;} temporal_Instant_to_ixdtf_string_with_compiled_data_result;
    temporal_Instant_to_ixdtf_string_with_compiled_data_result temporal_Instant_to_ixdtf_string_with_compiled_data(const temporal::capi::Instant* self, temporal::capi::TimeZone_option zone, temporal::capi::ToStringRoundingOptions options, temporal::diplomat::capi::DiplomatWrite* write);

    typedef struct temporal_Instant_to_ixdtf_string_with_provider_result {union { temporal::capi::TemporalError err;}; bool is_ok;} temporal_Instant_to_ixdtf_string_with_provider_result;
    temporal_Instant_to_ixdtf_string_with_provider_result temporal_Instant_to_ixdtf_string_with_provider(const temporal::capi::Instant* self, temporal::capi::TimeZone_option zone, temporal::capi::ToStringRoundingOptions options, const temporal::capi::Provider* p, temporal::diplomat::capi::DiplomatWrite* write);

    typedef struct temporal_Instant_to_zoned_date_time_iso_result {union {temporal::capi::ZonedDateTime* ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_Instant_to_zoned_date_time_iso_result;
    temporal_Instant_to_zoned_date_time_iso_result temporal_Instant_to_zoned_date_time_iso(const temporal::capi::Instant* self, temporal::capi::TimeZone zone);

    typedef struct temporal_Instant_to_zoned_date_time_iso_with_provider_result {union {temporal::capi::ZonedDateTime* ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_Instant_to_zoned_date_time_iso_with_provider_result;
    temporal_Instant_to_zoned_date_time_iso_with_provider_result temporal_Instant_to_zoned_date_time_iso_with_provider(const temporal::capi::Instant* self, temporal::capi::TimeZone zone, const temporal::capi::Provider* p);

    temporal::capi::Instant* temporal_Instant_clone(const temporal::capi::Instant* self);

    void temporal_Instant_destroy(Instant* self);

    } // extern "C"
} // namespace capi
} // namespace

inline temporal::diplomat::result<std::unique_ptr<temporal::Instant>, temporal::TemporalError> temporal::Instant::try_new(temporal::I128Nanoseconds ns) {
    auto result = temporal::capi::temporal_Instant_try_new(ns.AsFFI());
    return result.is_ok ? temporal::diplomat::result<std::unique_ptr<temporal::Instant>, temporal::TemporalError>(temporal::diplomat::Ok<std::unique_ptr<temporal::Instant>>(std::unique_ptr<temporal::Instant>(temporal::Instant::FromFFI(result.ok)))) : temporal::diplomat::result<std::unique_ptr<temporal::Instant>, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline temporal::diplomat::result<std::unique_ptr<temporal::Instant>, temporal::TemporalError> temporal::Instant::from_epoch_milliseconds(int64_t epoch_milliseconds) {
    auto result = temporal::capi::temporal_Instant_from_epoch_milliseconds(epoch_milliseconds);
    return result.is_ok ? temporal::diplomat::result<std::unique_ptr<temporal::Instant>, temporal::TemporalError>(temporal::diplomat::Ok<std::unique_ptr<temporal::Instant>>(std::unique_ptr<temporal::Instant>(temporal::Instant::FromFFI(result.ok)))) : temporal::diplomat::result<std::unique_ptr<temporal::Instant>, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline temporal::diplomat::result<std::unique_ptr<temporal::Instant>, temporal::TemporalError> temporal::Instant::from_utf8(std::string_view s) {
    auto result = temporal::capi::temporal_Instant_from_utf8({s.data(), s.size()});
    return result.is_ok ? temporal::diplomat::result<std::unique_ptr<temporal::Instant>, temporal::TemporalError>(temporal::diplomat::Ok<std::unique_ptr<temporal::Instant>>(std::unique_ptr<temporal::Instant>(temporal::Instant::FromFFI(result.ok)))) : temporal::diplomat::result<std::unique_ptr<temporal::Instant>, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline temporal::diplomat::result<std::unique_ptr<temporal::Instant>, temporal::TemporalError> temporal::Instant::from_utf16(std::u16string_view s) {
    auto result = temporal::capi::temporal_Instant_from_utf16({s.data(), s.size()});
    return result.is_ok ? temporal::diplomat::result<std::unique_ptr<temporal::Instant>, temporal::TemporalError>(temporal::diplomat::Ok<std::unique_ptr<temporal::Instant>>(std::unique_ptr<temporal::Instant>(temporal::Instant::FromFFI(result.ok)))) : temporal::diplomat::result<std::unique_ptr<temporal::Instant>, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline temporal::diplomat::result<std::unique_ptr<temporal::Instant>, temporal::TemporalError> temporal::Instant::add(const temporal::Duration& duration) const {
    auto result = temporal::capi::temporal_Instant_add(this->AsFFI(),
        duration.AsFFI());
    return result.is_ok ? temporal::diplomat::result<std::unique_ptr<temporal::Instant>, temporal::TemporalError>(temporal::diplomat::Ok<std::unique_ptr<temporal::Instant>>(std::unique_ptr<temporal::Instant>(temporal::Instant::FromFFI(result.ok)))) : temporal::diplomat::result<std::unique_ptr<temporal::Instant>, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline temporal::diplomat::result<std::unique_ptr<temporal::Instant>, temporal::TemporalError> temporal::Instant::subtract(const temporal::Duration& duration) const {
    auto result = temporal::capi::temporal_Instant_subtract(this->AsFFI(),
        duration.AsFFI());
    return result.is_ok ? temporal::diplomat::result<std::unique_ptr<temporal::Instant>, temporal::TemporalError>(temporal::diplomat::Ok<std::unique_ptr<temporal::Instant>>(std::unique_ptr<temporal::Instant>(temporal::Instant::FromFFI(result.ok)))) : temporal::diplomat::result<std::unique_ptr<temporal::Instant>, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline temporal::diplomat::result<std::unique_ptr<temporal::Duration>, temporal::TemporalError> temporal::Instant::since(const temporal::Instant& other, temporal::DifferenceSettings settings) const {
    auto result = temporal::capi::temporal_Instant_since(this->AsFFI(),
        other.AsFFI(),
        settings.AsFFI());
    return result.is_ok ? temporal::diplomat::result<std::unique_ptr<temporal::Duration>, temporal::TemporalError>(temporal::diplomat::Ok<std::unique_ptr<temporal::Duration>>(std::unique_ptr<temporal::Duration>(temporal::Duration::FromFFI(result.ok)))) : temporal::diplomat::result<std::unique_ptr<temporal::Duration>, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline temporal::diplomat::result<std::unique_ptr<temporal::Duration>, temporal::TemporalError> temporal::Instant::until(const temporal::Instant& other, temporal::DifferenceSettings settings) const {
    auto result = temporal::capi::temporal_Instant_until(this->AsFFI(),
        other.AsFFI(),
        settings.AsFFI());
    return result.is_ok ? temporal::diplomat::result<std::unique_ptr<temporal::Duration>, temporal::TemporalError>(temporal::diplomat::Ok<std::unique_ptr<temporal::Duration>>(std::unique_ptr<temporal::Duration>(temporal::Duration::FromFFI(result.ok)))) : temporal::diplomat::result<std::unique_ptr<temporal::Duration>, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline temporal::diplomat::result<std::unique_ptr<temporal::Instant>, temporal::TemporalError> temporal::Instant::round(temporal::RoundingOptions options) const {
    auto result = temporal::capi::temporal_Instant_round(this->AsFFI(),
        options.AsFFI());
    return result.is_ok ? temporal::diplomat::result<std::unique_ptr<temporal::Instant>, temporal::TemporalError>(temporal::diplomat::Ok<std::unique_ptr<temporal::Instant>>(std::unique_ptr<temporal::Instant>(temporal::Instant::FromFFI(result.ok)))) : temporal::diplomat::result<std::unique_ptr<temporal::Instant>, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline int8_t temporal::Instant::compare(const temporal::Instant& other) const {
    auto result = temporal::capi::temporal_Instant_compare(this->AsFFI(),
        other.AsFFI());
    return result;
}

inline bool temporal::Instant::equals(const temporal::Instant& other) const {
    auto result = temporal::capi::temporal_Instant_equals(this->AsFFI(),
        other.AsFFI());
    return result;
}

inline int64_t temporal::Instant::epoch_milliseconds() const {
    auto result = temporal::capi::temporal_Instant_epoch_milliseconds(this->AsFFI());
    return result;
}

inline temporal::I128Nanoseconds temporal::Instant::epoch_nanoseconds() const {
    auto result = temporal::capi::temporal_Instant_epoch_nanoseconds(this->AsFFI());
    return temporal::I128Nanoseconds::FromFFI(result);
}

inline temporal::diplomat::result<std::string, temporal::TemporalError> temporal::Instant::to_ixdtf_string_with_compiled_data(std::optional<temporal::TimeZone> zone, temporal::ToStringRoundingOptions options) const {
    std::string output;
    temporal::diplomat::capi::DiplomatWrite write = temporal::diplomat::WriteFromString(output);
    auto result = temporal::capi::temporal_Instant_to_ixdtf_string_with_compiled_data(this->AsFFI(),
        zone.has_value() ? (temporal::capi::TimeZone_option{ { zone.value().AsFFI() }, true }) : (temporal::capi::TimeZone_option{ {}, false }),
        options.AsFFI(),
        &write);
    return result.is_ok ? temporal::diplomat::result<std::string, temporal::TemporalError>(temporal::diplomat::Ok<std::string>(std::move(output))) : temporal::diplomat::result<std::string, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}
template<typename W>
inline temporal::diplomat::result<std::monostate, temporal::TemporalError> temporal::Instant::to_ixdtf_string_with_compiled_data_write(std::optional<temporal::TimeZone> zone, temporal::ToStringRoundingOptions options, W& writeable) const {
    temporal::diplomat::capi::DiplomatWrite write = temporal::diplomat::WriteTrait<W>::Construct(writeable);
    auto result = temporal::capi::temporal_Instant_to_ixdtf_string_with_compiled_data(this->AsFFI(),
        zone.has_value() ? (temporal::capi::TimeZone_option{ { zone.value().AsFFI() }, true }) : (temporal::capi::TimeZone_option{ {}, false }),
        options.AsFFI(),
        &write);
    return result.is_ok ? temporal::diplomat::result<std::monostate, temporal::TemporalError>(temporal::diplomat::Ok<std::monostate>()) : temporal::diplomat::result<std::monostate, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline temporal::diplomat::result<std::string, temporal::TemporalError> temporal::Instant::to_ixdtf_string_with_provider(std::optional<temporal::TimeZone> zone, temporal::ToStringRoundingOptions options, const temporal::Provider& p) const {
    std::string output;
    temporal::diplomat::capi::DiplomatWrite write = temporal::diplomat::WriteFromString(output);
    auto result = temporal::capi::temporal_Instant_to_ixdtf_string_with_provider(this->AsFFI(),
        zone.has_value() ? (temporal::capi::TimeZone_option{ { zone.value().AsFFI() }, true }) : (temporal::capi::TimeZone_option{ {}, false }),
        options.AsFFI(),
        p.AsFFI(),
        &write);
    return result.is_ok ? temporal::diplomat::result<std::string, temporal::TemporalError>(temporal::diplomat::Ok<std::string>(std::move(output))) : temporal::diplomat::result<std::string, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}
template<typename W>
inline temporal::diplomat::result<std::monostate, temporal::TemporalError> temporal::Instant::to_ixdtf_string_with_provider_write(std::optional<temporal::TimeZone> zone, temporal::ToStringRoundingOptions options, const temporal::Provider& p, W& writeable) const {
    temporal::diplomat::capi::DiplomatWrite write = temporal::diplomat::WriteTrait<W>::Construct(writeable);
    auto result = temporal::capi::temporal_Instant_to_ixdtf_string_with_provider(this->AsFFI(),
        zone.has_value() ? (temporal::capi::TimeZone_option{ { zone.value().AsFFI() }, true }) : (temporal::capi::TimeZone_option{ {}, false }),
        options.AsFFI(),
        p.AsFFI(),
        &write);
    return result.is_ok ? temporal::diplomat::result<std::monostate, temporal::TemporalError>(temporal::diplomat::Ok<std::monostate>()) : temporal::diplomat::result<std::monostate, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline temporal::diplomat::result<std::unique_ptr<temporal::ZonedDateTime>, temporal::TemporalError> temporal::Instant::to_zoned_date_time_iso(temporal::TimeZone zone) const {
    auto result = temporal::capi::temporal_Instant_to_zoned_date_time_iso(this->AsFFI(),
        zone.AsFFI());
    return result.is_ok ? temporal::diplomat::result<std::unique_ptr<temporal::ZonedDateTime>, temporal::TemporalError>(temporal::diplomat::Ok<std::unique_ptr<temporal::ZonedDateTime>>(std::unique_ptr<temporal::ZonedDateTime>(temporal::ZonedDateTime::FromFFI(result.ok)))) : temporal::diplomat::result<std::unique_ptr<temporal::ZonedDateTime>, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline temporal::diplomat::result<std::unique_ptr<temporal::ZonedDateTime>, temporal::TemporalError> temporal::Instant::to_zoned_date_time_iso_with_provider(temporal::TimeZone zone, const temporal::Provider& p) const {
    auto result = temporal::capi::temporal_Instant_to_zoned_date_time_iso_with_provider(this->AsFFI(),
        zone.AsFFI(),
        p.AsFFI());
    return result.is_ok ? temporal::diplomat::result<std::unique_ptr<temporal::ZonedDateTime>, temporal::TemporalError>(temporal::diplomat::Ok<std::unique_ptr<temporal::ZonedDateTime>>(std::unique_ptr<temporal::ZonedDateTime>(temporal::ZonedDateTime::FromFFI(result.ok)))) : temporal::diplomat::result<std::unique_ptr<temporal::ZonedDateTime>, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline std::unique_ptr<temporal::Instant> temporal::Instant::clone() const {
    auto result = temporal::capi::temporal_Instant_clone(this->AsFFI());
    return std::unique_ptr<temporal::Instant>(temporal::Instant::FromFFI(result));
}

inline const temporal::capi::Instant* temporal::Instant::AsFFI() const {
    return reinterpret_cast<const temporal::capi::Instant*>(this);
}

inline temporal::capi::Instant* temporal::Instant::AsFFI() {
    return reinterpret_cast<temporal::capi::Instant*>(this);
}

inline const temporal::Instant* temporal::Instant::FromFFI(const temporal::capi::Instant* ptr) {
    return reinterpret_cast<const temporal::Instant*>(ptr);
}

inline temporal::Instant* temporal::Instant::FromFFI(temporal::capi::Instant* ptr) {
    return reinterpret_cast<temporal::Instant*>(ptr);
}

inline void temporal::Instant::operator delete(void* ptr) {
    temporal::capi::temporal_Instant_destroy(reinterpret_cast<temporal::capi::Instant*>(ptr));
}


#endif // TEMPORAL_Instant_HPP
