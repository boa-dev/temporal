#ifndef TEMPORAL_Calendar_HPP
#define TEMPORAL_Calendar_HPP

#include "Calendar.d.hpp"

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <functional>
#include <optional>
#include <cstdlib>
#include "AnyCalendarKind.hpp"
#include "TemporalError.hpp"
#include "diplomat_runtime.hpp"


namespace temporal {
namespace capi {
    extern "C" {

    temporal::capi::Calendar* temporal_Calendar_try_new_constrain(temporal::capi::AnyCalendarKind kind);

    typedef struct temporal_Calendar_from_utf8_result {union {temporal::capi::Calendar* ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_Calendar_from_utf8_result;
    temporal_Calendar_from_utf8_result temporal_Calendar_from_utf8(temporal::diplomat::capi::DiplomatStringView s);

    bool temporal_Calendar_is_iso(const temporal::capi::Calendar* self);

    temporal::diplomat::capi::DiplomatStringView temporal_Calendar_identifier(const temporal::capi::Calendar* self);

    temporal::capi::AnyCalendarKind temporal_Calendar_kind(const temporal::capi::Calendar* self);

    void temporal_Calendar_destroy(Calendar* self);

    } // extern "C"
} // namespace capi
} // namespace

inline std::unique_ptr<temporal::Calendar> temporal::Calendar::try_new_constrain(temporal::AnyCalendarKind kind) {
    auto result = temporal::capi::temporal_Calendar_try_new_constrain(kind.AsFFI());
    return std::unique_ptr<temporal::Calendar>(temporal::Calendar::FromFFI(result));
}

inline temporal::diplomat::result<std::unique_ptr<temporal::Calendar>, temporal::TemporalError> temporal::Calendar::from_utf8(std::string_view s) {
    auto result = temporal::capi::temporal_Calendar_from_utf8({s.data(), s.size()});
    return result.is_ok ? temporal::diplomat::result<std::unique_ptr<temporal::Calendar>, temporal::TemporalError>(temporal::diplomat::Ok<std::unique_ptr<temporal::Calendar>>(std::unique_ptr<temporal::Calendar>(temporal::Calendar::FromFFI(result.ok)))) : temporal::diplomat::result<std::unique_ptr<temporal::Calendar>, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline bool temporal::Calendar::is_iso() const {
    auto result = temporal::capi::temporal_Calendar_is_iso(this->AsFFI());
    return result;
}

inline std::string_view temporal::Calendar::identifier() const {
    auto result = temporal::capi::temporal_Calendar_identifier(this->AsFFI());
    return std::string_view(result.data, result.len);
}

inline temporal::AnyCalendarKind temporal::Calendar::kind() const {
    auto result = temporal::capi::temporal_Calendar_kind(this->AsFFI());
    return temporal::AnyCalendarKind::FromFFI(result);
}

inline const temporal::capi::Calendar* temporal::Calendar::AsFFI() const {
    return reinterpret_cast<const temporal::capi::Calendar*>(this);
}

inline temporal::capi::Calendar* temporal::Calendar::AsFFI() {
    return reinterpret_cast<temporal::capi::Calendar*>(this);
}

inline const temporal::Calendar* temporal::Calendar::FromFFI(const temporal::capi::Calendar* ptr) {
    return reinterpret_cast<const temporal::Calendar*>(ptr);
}

inline temporal::Calendar* temporal::Calendar::FromFFI(temporal::capi::Calendar* ptr) {
    return reinterpret_cast<temporal::Calendar*>(ptr);
}

inline void temporal::Calendar::operator delete(void* ptr) {
    temporal::capi::temporal_Calendar_destroy(reinterpret_cast<temporal::capi::Calendar*>(ptr));
}


#endif // TEMPORAL_Calendar_HPP
