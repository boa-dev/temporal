#ifndef TEMPORAL_DateDuration_HPP
#define TEMPORAL_DateDuration_HPP

#include "DateDuration.d.hpp"

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <functional>
#include <optional>
#include <cstdlib>
#include "Sign.hpp"
#include "TemporalError.hpp"
#include "diplomat_runtime.hpp"


namespace temporal {
namespace capi {
    extern "C" {

    typedef struct temporal_DateDuration_try_new_result {union {temporal::capi::DateDuration* ok; temporal::capi::TemporalError err;}; bool is_ok;} temporal_DateDuration_try_new_result;
    temporal_DateDuration_try_new_result temporal_DateDuration_try_new(int64_t years, int64_t months, int64_t weeks, int64_t days);

    temporal::capi::DateDuration* temporal_DateDuration_abs(const temporal::capi::DateDuration* self);

    temporal::capi::DateDuration* temporal_DateDuration_negated(const temporal::capi::DateDuration* self);

    temporal::capi::Sign temporal_DateDuration_sign(const temporal::capi::DateDuration* self);

    void temporal_DateDuration_destroy(DateDuration* self);

    } // extern "C"
} // namespace capi
} // namespace

inline temporal::diplomat::result<std::unique_ptr<temporal::DateDuration>, temporal::TemporalError> temporal::DateDuration::try_new(int64_t years, int64_t months, int64_t weeks, int64_t days) {
    auto result = temporal::capi::temporal_DateDuration_try_new(years,
        months,
        weeks,
        days);
    return result.is_ok ? temporal::diplomat::result<std::unique_ptr<temporal::DateDuration>, temporal::TemporalError>(temporal::diplomat::Ok<std::unique_ptr<temporal::DateDuration>>(std::unique_ptr<temporal::DateDuration>(temporal::DateDuration::FromFFI(result.ok)))) : temporal::diplomat::result<std::unique_ptr<temporal::DateDuration>, temporal::TemporalError>(temporal::diplomat::Err<temporal::TemporalError>(temporal::TemporalError::FromFFI(result.err)));
}

inline std::unique_ptr<temporal::DateDuration> temporal::DateDuration::abs() const {
    auto result = temporal::capi::temporal_DateDuration_abs(this->AsFFI());
    return std::unique_ptr<temporal::DateDuration>(temporal::DateDuration::FromFFI(result));
}

inline std::unique_ptr<temporal::DateDuration> temporal::DateDuration::negated() const {
    auto result = temporal::capi::temporal_DateDuration_negated(this->AsFFI());
    return std::unique_ptr<temporal::DateDuration>(temporal::DateDuration::FromFFI(result));
}

inline temporal::Sign temporal::DateDuration::sign() const {
    auto result = temporal::capi::temporal_DateDuration_sign(this->AsFFI());
    return temporal::Sign::FromFFI(result);
}

inline const temporal::capi::DateDuration* temporal::DateDuration::AsFFI() const {
    return reinterpret_cast<const temporal::capi::DateDuration*>(this);
}

inline temporal::capi::DateDuration* temporal::DateDuration::AsFFI() {
    return reinterpret_cast<temporal::capi::DateDuration*>(this);
}

inline const temporal::DateDuration* temporal::DateDuration::FromFFI(const temporal::capi::DateDuration* ptr) {
    return reinterpret_cast<const temporal::DateDuration*>(ptr);
}

inline temporal::DateDuration* temporal::DateDuration::FromFFI(temporal::capi::DateDuration* ptr) {
    return reinterpret_cast<temporal::DateDuration*>(ptr);
}

inline void temporal::DateDuration::operator delete(void* ptr) {
    temporal::capi::temporal_DateDuration_destroy(reinterpret_cast<temporal::capi::DateDuration*>(ptr));
}


#endif // TEMPORAL_DateDuration_HPP
