#ifndef TEMPORAL_UnsignedRoundingMode_HPP
#define TEMPORAL_UnsignedRoundingMode_HPP

#include "UnsignedRoundingMode.d.hpp"

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <functional>
#include <optional>
#include <cstdlib>
#include "diplomat_runtime.hpp"


namespace temporal {
namespace capi {

} // namespace capi
} // namespace

inline temporal::capi::UnsignedRoundingMode temporal::UnsignedRoundingMode::AsFFI() const {
    return static_cast<temporal::capi::UnsignedRoundingMode>(value);
}

inline temporal::UnsignedRoundingMode temporal::UnsignedRoundingMode::FromFFI(temporal::capi::UnsignedRoundingMode c_enum) {
    switch (c_enum) {
        case temporal::capi::UnsignedRoundingMode_Infinity:
        case temporal::capi::UnsignedRoundingMode_Zero:
        case temporal::capi::UnsignedRoundingMode_HalfInfinity:
        case temporal::capi::UnsignedRoundingMode_HalfZero:
        case temporal::capi::UnsignedRoundingMode_HalfEven:
            return static_cast<temporal::UnsignedRoundingMode::Value>(c_enum);
        default:
            std::abort();
    }
}
#endif // TEMPORAL_UnsignedRoundingMode_HPP
