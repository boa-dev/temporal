#ifndef TEMPORAL_PlainTime_HPP
#define TEMPORAL_PlainTime_HPP

#include "PlainTime.d.hpp"

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <functional>
#include <optional>
#include <cstdlib>
#include "ArithmeticOverflow.hpp"
#include "DifferenceSettings.hpp"
#include "Duration.hpp"
#include "I128Nanoseconds.hpp"
#include "PartialTime.hpp"
#include "Provider.hpp"
#include "RoundingOptions.hpp"
#include "TemporalError.hpp"
#include "TimeZone.hpp"
#include "ToStringRoundingOptions.hpp"
#include "diplomat_runtime.hpp"


namespace temporal {
namespace capi {
    extern "C" {

    typedef struct temporal_PlainTime_try_new_constrain_result {union {temporal::capi::PlainTime* ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_PlainTime_try_new_constrain_result;
    temporal_PlainTime_try_new_constrain_result temporal_PlainTime_try_new_constrain(uint8_t hour, uint8_t minute, uint8_t second, uint16_t millisecond, uint16_t microsecond, uint16_t nanosecond);

    typedef struct temporal_PlainTime_try_new_result {union {temporal::capi::PlainTime* ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_PlainTime_try_new_result;
    temporal_PlainTime_try_new_result temporal_PlainTime_try_new(uint8_t hour, uint8_t minute, uint8_t second, uint16_t millisecond, uint16_t microsecond, uint16_t nanosecond);

    typedef struct temporal_PlainTime_from_partial_result {union {temporal::capi::PlainTime* ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_PlainTime_from_partial_result;
    temporal_PlainTime_from_partial_result temporal_PlainTime_from_partial(temporal::capi::PartialTime partial, temporal::capi::ArithmeticOverflow_option overflow);

    typedef struct temporal_PlainTime_from_epoch_milliseconds_result {union {temporal::capi::PlainTime* ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_PlainTime_from_epoch_milliseconds_result;
    temporal_PlainTime_from_epoch_milliseconds_result temporal_PlainTime_from_epoch_milliseconds(int64_t ms, temporal::capi::TimeZone tz);

    typedef struct temporal_PlainTime_from_epoch_milliseconds_with_provider_result {union {temporal::capi::PlainTime* ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_PlainTime_from_epoch_milliseconds_with_provider_result;
    temporal_PlainTime_from_epoch_milliseconds_with_provider_result temporal_PlainTime_from_epoch_milliseconds_with_provider(int64_t ms, temporal::capi::TimeZone tz, const temporal::capi::Provider* p);

    typedef struct temporal_PlainTime_from_epoch_nanoseconds_result {union {temporal::capi::PlainTime* ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_PlainTime_from_epoch_nanoseconds_result;
    temporal_PlainTime_from_epoch_nanoseconds_result temporal_PlainTime_from_epoch_nanoseconds(temporal::capi::I128Nanoseconds ns, temporal::capi::TimeZone tz);

    typedef struct temporal_PlainTime_from_epoch_nanoseconds_with_provider_result {union {temporal::capi::PlainTime* ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_PlainTime_from_epoch_nanoseconds_with_provider_result;
    temporal_PlainTime_from_epoch_nanoseconds_with_provider_result temporal_PlainTime_from_epoch_nanoseconds_with_provider(temporal::capi::I128Nanoseconds ns, temporal::capi::TimeZone tz, const temporal::capi::Provider* p);

    typedef struct temporal_PlainTime_epoch_ms_for_utc_result {union {int64_t ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_PlainTime_epoch_ms_for_utc_result;
    temporal_PlainTime_epoch_ms_for_utc_result temporal_PlainTime_epoch_ms_for_utc(const temporal::capi::PlainTime* self);

    typedef struct temporal_PlainTime_with_result {union {temporal::capi::PlainTime* ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_PlainTime_with_result;
    temporal_PlainTime_with_result temporal_PlainTime_with(const temporal::capi::PlainTime* self, temporal::capi::PartialTime partial, temporal::capi::ArithmeticOverflow_option overflow);

    typedef struct temporal_PlainTime_from_utf8_result {union {temporal::capi::PlainTime* ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_PlainTime_from_utf8_result;
    temporal_PlainTime_from_utf8_result temporal_PlainTime_from_utf8(temporal::diplomat::capi::DiplomatStringView s);

    typedef struct temporal_PlainTime_from_utf16_result {union {temporal::capi::PlainTime* ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_PlainTime_from_utf16_result;
    temporal_PlainTime_from_utf16_result temporal_PlainTime_from_utf16(temporal::diplomat::capi::DiplomatString16View s);

    uint8_t temporal_PlainTime_hour(const temporal::capi::PlainTime* self);

    uint8_t temporal_PlainTime_minute(const temporal::capi::PlainTime* self);

    uint8_t temporal_PlainTime_second(const temporal::capi::PlainTime* self);

    uint16_t temporal_PlainTime_millisecond(const temporal::capi::PlainTime* self);

    uint16_t temporal_PlainTime_microsecond(const temporal::capi::PlainTime* self);

    uint16_t temporal_PlainTime_nanosecond(const temporal::capi::PlainTime* self);

    typedef struct temporal_PlainTime_add_result {union {temporal::capi::PlainTime* ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_PlainTime_add_result;
    temporal_PlainTime_add_result temporal_PlainTime_add(const temporal::capi::PlainTime* self, const temporal::capi::Duration* duration);

    typedef struct temporal_PlainTime_subtract_result {union {temporal::capi::PlainTime* ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_PlainTime_subtract_result;
    temporal_PlainTime_subtract_result temporal_PlainTime_subtract(const temporal::capi::PlainTime* self, const temporal::capi::Duration* duration);

    typedef struct temporal_PlainTime_until_result {union {temporal::capi::Duration* ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_PlainTime_until_result;
    temporal_PlainTime_until_result temporal_PlainTime_until(const temporal::capi::PlainTime* self, const temporal::capi::PlainTime* other, temporal::capi::DifferenceSettings settings);

    typedef struct temporal_PlainTime_since_result {union {temporal::capi::Duration* ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_PlainTime_since_result;
    temporal_PlainTime_since_result temporal_PlainTime_since(const temporal::capi::PlainTime* self, const temporal::capi::PlainTime* other, temporal::capi::DifferenceSettings settings);

    bool temporal_PlainTime_equals(const temporal::capi::PlainTime* self, const temporal::capi::PlainTime* other);

    int8_t temporal_PlainTime_compare(const temporal::capi::PlainTime* one, const temporal::capi::PlainTime* two);

    typedef struct temporal_PlainTime_round_result {union {temporal::capi::PlainTime* ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_PlainTime_round_result;
    temporal_PlainTime_round_result temporal_PlainTime_round(const temporal::capi::PlainTime* self, temporal::capi::RoundingOptions options);

    typedef struct temporal_PlainTime_to_ixdtf_string_result {union { temporal::capi::TemporalError err;}; bool is_ok;} temporal_PlainTime_to_ixdtf_string_result;
    temporal_PlainTime_to_ixdtf_string_result temporal_PlainTime_to_ixdtf_string(const temporal::capi::PlainTime* self, temporal::capi::ToStringRoundingOptions options, temporal::diplomat::capi::DiplomatWrite* write);

    temporal::capi::PlainTime* temporal_PlainTime_clone(const temporal::capi::PlainTime* self);

    void temporal_PlainTime_destroy(PlainTime* self);

    } // extern "C"
} // namespace capi
} // namespace

inline temporal::diplomat::result<std::unique_ptr<temporal::PlainTime>, temporal::TemporalError> temporal::PlainTime::try_new_constrain(uint8_t hour, uint8_t minute, uint8_t second, uint16_t millisecond, uint16_t microsecond, uint16_t nanosecond) {
    auto result = temporal::capi::temporal_PlainTime_try_new_constrain(hour,
        minute,
        second,
        millisecond,
        microsecond,
        nanosecond);
    return result.is_ok ? temporal::diplomat::result<std::unique_ptr<temporal::PlainTime>, temporal::TemporalError>(temporal::diplomat::Ok<std::unique_ptr<temporal::PlainTime>>(std::unique_ptr<temporal::PlainTime>(temporal::PlainTime::FromFFI(result.ok)))) : temporal::diplomat::result<std::unique_ptr<temporal::PlainTime>, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline temporal::diplomat::result<std::unique_ptr<temporal::PlainTime>, temporal::TemporalError> temporal::PlainTime::try_new(uint8_t hour, uint8_t minute, uint8_t second, uint16_t millisecond, uint16_t microsecond, uint16_t nanosecond) {
    auto result = temporal::capi::temporal_PlainTime_try_new(hour,
        minute,
        second,
        millisecond,
        microsecond,
        nanosecond);
    return result.is_ok ? temporal::diplomat::result<std::unique_ptr<temporal::PlainTime>, temporal::TemporalError>(temporal::diplomat::Ok<std::unique_ptr<temporal::PlainTime>>(std::unique_ptr<temporal::PlainTime>(temporal::PlainTime::FromFFI(result.ok)))) : temporal::diplomat::result<std::unique_ptr<temporal::PlainTime>, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline temporal::diplomat::result<std::unique_ptr<temporal::PlainTime>, temporal::TemporalError> temporal::PlainTime::from_partial(temporal::PartialTime partial, std::optional<temporal::ArithmeticOverflow> overflow) {
    auto result = temporal::capi::temporal_PlainTime_from_partial(partial.AsFFI(),
        overflow.has_value() ? (temporal::capi::ArithmeticOverflow_option{ { overflow.value().AsFFI() }, true }) : (temporal::capi::ArithmeticOverflow_option{ {}, false }));
    return result.is_ok ? temporal::diplomat::result<std::unique_ptr<temporal::PlainTime>, temporal::TemporalError>(temporal::diplomat::Ok<std::unique_ptr<temporal::PlainTime>>(std::unique_ptr<temporal::PlainTime>(temporal::PlainTime::FromFFI(result.ok)))) : temporal::diplomat::result<std::unique_ptr<temporal::PlainTime>, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline temporal::diplomat::result<std::unique_ptr<temporal::PlainTime>, temporal::TemporalError> temporal::PlainTime::from_epoch_milliseconds(int64_t ms, temporal::TimeZone tz) {
    auto result = temporal::capi::temporal_PlainTime_from_epoch_milliseconds(ms,
        tz.AsFFI());
    return result.is_ok ? temporal::diplomat::result<std::unique_ptr<temporal::PlainTime>, temporal::TemporalError>(temporal::diplomat::Ok<std::unique_ptr<temporal::PlainTime>>(std::unique_ptr<temporal::PlainTime>(temporal::PlainTime::FromFFI(result.ok)))) : temporal::diplomat::result<std::unique_ptr<temporal::PlainTime>, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline temporal::diplomat::result<std::unique_ptr<temporal::PlainTime>, temporal::TemporalError> temporal::PlainTime::from_epoch_milliseconds_with_provider(int64_t ms, temporal::TimeZone tz, const temporal::Provider& p) {
    auto result = temporal::capi::temporal_PlainTime_from_epoch_milliseconds_with_provider(ms,
        tz.AsFFI(),
        p.AsFFI());
    return result.is_ok ? temporal::diplomat::result<std::unique_ptr<temporal::PlainTime>, temporal::TemporalError>(temporal::diplomat::Ok<std::unique_ptr<temporal::PlainTime>>(std::unique_ptr<temporal::PlainTime>(temporal::PlainTime::FromFFI(result.ok)))) : temporal::diplomat::result<std::unique_ptr<temporal::PlainTime>, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline temporal::diplomat::result<std::unique_ptr<temporal::PlainTime>, temporal::TemporalError> temporal::PlainTime::from_epoch_nanoseconds(temporal::I128Nanoseconds ns, temporal::TimeZone tz) {
    auto result = temporal::capi::temporal_PlainTime_from_epoch_nanoseconds(ns.AsFFI(),
        tz.AsFFI());
    return result.is_ok ? temporal::diplomat::result<std::unique_ptr<temporal::PlainTime>, temporal::TemporalError>(temporal::diplomat::Ok<std::unique_ptr<temporal::PlainTime>>(std::unique_ptr<temporal::PlainTime>(temporal::PlainTime::FromFFI(result.ok)))) : temporal::diplomat::result<std::unique_ptr<temporal::PlainTime>, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline temporal::diplomat::result<std::unique_ptr<temporal::PlainTime>, temporal::TemporalError> temporal::PlainTime::from_epoch_nanoseconds_with_provider(temporal::I128Nanoseconds ns, temporal::TimeZone tz, const temporal::Provider& p) {
    auto result = temporal::capi::temporal_PlainTime_from_epoch_nanoseconds_with_provider(ns.AsFFI(),
        tz.AsFFI(),
        p.AsFFI());
    return result.is_ok ? temporal::diplomat::result<std::unique_ptr<temporal::PlainTime>, temporal::TemporalError>(temporal::diplomat::Ok<std::unique_ptr<temporal::PlainTime>>(std::unique_ptr<temporal::PlainTime>(temporal::PlainTime::FromFFI(result.ok)))) : temporal::diplomat::result<std::unique_ptr<temporal::PlainTime>, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline temporal::diplomat::result<int64_t, temporal::TemporalError> temporal::PlainTime::epoch_ms_for_utc() const {
    auto result = temporal::capi::temporal_PlainTime_epoch_ms_for_utc(this->AsFFI());
    return result.is_ok ? temporal::diplomat::result<int64_t, temporal::TemporalError>(temporal::diplomat::Ok<int64_t>(result.ok)) : temporal::diplomat::result<int64_t, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline temporal::diplomat::result<std::unique_ptr<temporal::PlainTime>, temporal::TemporalError> temporal::PlainTime::with(temporal::PartialTime partial, std::optional<temporal::ArithmeticOverflow> overflow) const {
    auto result = temporal::capi::temporal_PlainTime_with(this->AsFFI(),
        partial.AsFFI(),
        overflow.has_value() ? (temporal::capi::ArithmeticOverflow_option{ { overflow.value().AsFFI() }, true }) : (temporal::capi::ArithmeticOverflow_option{ {}, false }));
    return result.is_ok ? temporal::diplomat::result<std::unique_ptr<temporal::PlainTime>, temporal::TemporalError>(temporal::diplomat::Ok<std::unique_ptr<temporal::PlainTime>>(std::unique_ptr<temporal::PlainTime>(temporal::PlainTime::FromFFI(result.ok)))) : temporal::diplomat::result<std::unique_ptr<temporal::PlainTime>, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline temporal::diplomat::result<std::unique_ptr<temporal::PlainTime>, temporal::TemporalError> temporal::PlainTime::from_utf8(std::string_view s) {
    auto result = temporal::capi::temporal_PlainTime_from_utf8({s.data(), s.size()});
    return result.is_ok ? temporal::diplomat::result<std::unique_ptr<temporal::PlainTime>, temporal::TemporalError>(temporal::diplomat::Ok<std::unique_ptr<temporal::PlainTime>>(std::unique_ptr<temporal::PlainTime>(temporal::PlainTime::FromFFI(result.ok)))) : temporal::diplomat::result<std::unique_ptr<temporal::PlainTime>, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline temporal::diplomat::result<std::unique_ptr<temporal::PlainTime>, temporal::TemporalError> temporal::PlainTime::from_utf16(std::u16string_view s) {
    auto result = temporal::capi::temporal_PlainTime_from_utf16({s.data(), s.size()});
    return result.is_ok ? temporal::diplomat::result<std::unique_ptr<temporal::PlainTime>, temporal::TemporalError>(temporal::diplomat::Ok<std::unique_ptr<temporal::PlainTime>>(std::unique_ptr<temporal::PlainTime>(temporal::PlainTime::FromFFI(result.ok)))) : temporal::diplomat::result<std::unique_ptr<temporal::PlainTime>, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline uint8_t temporal::PlainTime::hour() const {
    auto result = temporal::capi::temporal_PlainTime_hour(this->AsFFI());
    return result;
}

inline uint8_t temporal::PlainTime::minute() const {
    auto result = temporal::capi::temporal_PlainTime_minute(this->AsFFI());
    return result;
}

inline uint8_t temporal::PlainTime::second() const {
    auto result = temporal::capi::temporal_PlainTime_second(this->AsFFI());
    return result;
}

inline uint16_t temporal::PlainTime::millisecond() const {
    auto result = temporal::capi::temporal_PlainTime_millisecond(this->AsFFI());
    return result;
}

inline uint16_t temporal::PlainTime::microsecond() const {
    auto result = temporal::capi::temporal_PlainTime_microsecond(this->AsFFI());
    return result;
}

inline uint16_t temporal::PlainTime::nanosecond() const {
    auto result = temporal::capi::temporal_PlainTime_nanosecond(this->AsFFI());
    return result;
}

inline temporal::diplomat::result<std::unique_ptr<temporal::PlainTime>, temporal::TemporalError> temporal::PlainTime::add(const temporal::Duration& duration) const {
    auto result = temporal::capi::temporal_PlainTime_add(this->AsFFI(),
        duration.AsFFI());
    return result.is_ok ? temporal::diplomat::result<std::unique_ptr<temporal::PlainTime>, temporal::TemporalError>(temporal::diplomat::Ok<std::unique_ptr<temporal::PlainTime>>(std::unique_ptr<temporal::PlainTime>(temporal::PlainTime::FromFFI(result.ok)))) : temporal::diplomat::result<std::unique_ptr<temporal::PlainTime>, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline temporal::diplomat::result<std::unique_ptr<temporal::PlainTime>, temporal::TemporalError> temporal::PlainTime::subtract(const temporal::Duration& duration) const {
    auto result = temporal::capi::temporal_PlainTime_subtract(this->AsFFI(),
        duration.AsFFI());
    return result.is_ok ? temporal::diplomat::result<std::unique_ptr<temporal::PlainTime>, temporal::TemporalError>(temporal::diplomat::Ok<std::unique_ptr<temporal::PlainTime>>(std::unique_ptr<temporal::PlainTime>(temporal::PlainTime::FromFFI(result.ok)))) : temporal::diplomat::result<std::unique_ptr<temporal::PlainTime>, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline temporal::diplomat::result<std::unique_ptr<temporal::Duration>, temporal::TemporalError> temporal::PlainTime::until(const temporal::PlainTime& other, temporal::DifferenceSettings settings) const {
    auto result = temporal::capi::temporal_PlainTime_until(this->AsFFI(),
        other.AsFFI(),
        settings.AsFFI());
    return result.is_ok ? temporal::diplomat::result<std::unique_ptr<temporal::Duration>, temporal::TemporalError>(temporal::diplomat::Ok<std::unique_ptr<temporal::Duration>>(std::unique_ptr<temporal::Duration>(temporal::Duration::FromFFI(result.ok)))) : temporal::diplomat::result<std::unique_ptr<temporal::Duration>, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline temporal::diplomat::result<std::unique_ptr<temporal::Duration>, temporal::TemporalError> temporal::PlainTime::since(const temporal::PlainTime& other, temporal::DifferenceSettings settings) const {
    auto result = temporal::capi::temporal_PlainTime_since(this->AsFFI(),
        other.AsFFI(),
        settings.AsFFI());
    return result.is_ok ? temporal::diplomat::result<std::unique_ptr<temporal::Duration>, temporal::TemporalError>(temporal::diplomat::Ok<std::unique_ptr<temporal::Duration>>(std::unique_ptr<temporal::Duration>(temporal::Duration::FromFFI(result.ok)))) : temporal::diplomat::result<std::unique_ptr<temporal::Duration>, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline bool temporal::PlainTime::equals(const temporal::PlainTime& other) const {
    auto result = temporal::capi::temporal_PlainTime_equals(this->AsFFI(),
        other.AsFFI());
    return result;
}

inline int8_t temporal::PlainTime::compare(const temporal::PlainTime& one, const temporal::PlainTime& two) {
    auto result = temporal::capi::temporal_PlainTime_compare(one.AsFFI(),
        two.AsFFI());
    return result;
}

inline temporal::diplomat::result<std::unique_ptr<temporal::PlainTime>, temporal::TemporalError> temporal::PlainTime::round(temporal::RoundingOptions options) const {
    auto result = temporal::capi::temporal_PlainTime_round(this->AsFFI(),
        options.AsFFI());
    return result.is_ok ? temporal::diplomat::result<std::unique_ptr<temporal::PlainTime>, temporal::TemporalError>(temporal::diplomat::Ok<std::unique_ptr<temporal::PlainTime>>(std::unique_ptr<temporal::PlainTime>(temporal::PlainTime::FromFFI(result.ok)))) : temporal::diplomat::result<std::unique_ptr<temporal::PlainTime>, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline temporal::diplomat::result<std::string, temporal::TemporalError> temporal::PlainTime::to_ixdtf_string(temporal::ToStringRoundingOptions options) const {
    std::string output;
    temporal::diplomat::capi::DiplomatWrite write = temporal::diplomat::WriteFromString(output);
    auto result = temporal::capi::temporal_PlainTime_to_ixdtf_string(this->AsFFI(),
        options.AsFFI(),
        &write);
    return result.is_ok ? temporal::diplomat::result<std::string, temporal::TemporalError>(temporal::diplomat::Ok<std::string>(std::move(output))) : temporal::diplomat::result<std::string, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}
template<typename W>
inline temporal::diplomat::result<std::monostate, temporal::TemporalError> temporal::PlainTime::to_ixdtf_string_write(temporal::ToStringRoundingOptions options, W& writeable) const {
    temporal::diplomat::capi::DiplomatWrite write = temporal::diplomat::WriteTrait<W>::Construct(writeable);
    auto result = temporal::capi::temporal_PlainTime_to_ixdtf_string(this->AsFFI(),
        options.AsFFI(),
        &write);
    return result.is_ok ? temporal::diplomat::result<std::monostate, temporal::TemporalError>(temporal::diplomat::Ok<std::monostate>()) : temporal::diplomat::result<std::monostate, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline std::unique_ptr<temporal::PlainTime> temporal::PlainTime::clone() const {
    auto result = temporal::capi::temporal_PlainTime_clone(this->AsFFI());
    return std::unique_ptr<temporal::PlainTime>(temporal::PlainTime::FromFFI(result));
}

inline const temporal::capi::PlainTime* temporal::PlainTime::AsFFI() const {
    return reinterpret_cast<const temporal::capi::PlainTime*>(this);
}

inline temporal::capi::PlainTime* temporal::PlainTime::AsFFI() {
    return reinterpret_cast<temporal::capi::PlainTime*>(this);
}

inline const temporal::PlainTime* temporal::PlainTime::FromFFI(const temporal::capi::PlainTime* ptr) {
    return reinterpret_cast<const temporal::PlainTime*>(ptr);
}

inline temporal::PlainTime* temporal::PlainTime::FromFFI(temporal::capi::PlainTime* ptr) {
    return reinterpret_cast<temporal::PlainTime*>(ptr);
}

inline void temporal::PlainTime::operator delete(void* ptr) {
    temporal::capi::temporal_PlainTime_destroy(reinterpret_cast<temporal::capi::PlainTime*>(ptr));
}


#endif // TEMPORAL_PlainTime_HPP
