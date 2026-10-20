#ifndef TEMPORAL_RoundingMode_HPP
#define TEMPORAL_RoundingMode_HPP

#include "RoundingMode.d.hpp"

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

inline temporal::capi::RoundingMode temporal::RoundingMode::AsFFI() const {
    return static_cast<temporal::capi::RoundingMode>(value);
}

inline temporal::RoundingMode temporal::RoundingMode::FromFFI(temporal::capi::RoundingMode c_enum) {
    switch (c_enum) {
        case temporal::capi::RoundingMode_Ceil:
        case temporal::capi::RoundingMode_Floor:
        case temporal::capi::RoundingMode_Expand:
        case temporal::capi::RoundingMode_Trunc:
        case temporal::capi::RoundingMode_HalfCeil:
        case temporal::capi::RoundingMode_HalfFloor:
        case temporal::capi::RoundingMode_HalfExpand:
        case temporal::capi::RoundingMode_HalfTrunc:
        case temporal::capi::RoundingMode_HalfEven:
            return static_cast<temporal::RoundingMode::Value>(c_enum);
        default:
            std::abort();
    }
}
#endif // TEMPORAL_RoundingMode_HPP
