#ifndef TEMPORAL_ArithmeticOverflow_HPP
#define TEMPORAL_ArithmeticOverflow_HPP

#include "ArithmeticOverflow.d.hpp"

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

inline temporal::capi::ArithmeticOverflow temporal::ArithmeticOverflow::AsFFI() const {
    return static_cast<temporal::capi::ArithmeticOverflow>(value);
}

inline temporal::ArithmeticOverflow temporal::ArithmeticOverflow::FromFFI(temporal::capi::ArithmeticOverflow c_enum) {
    switch (c_enum) {
        case temporal::capi::ArithmeticOverflow_Constrain:
        case temporal::capi::ArithmeticOverflow_Reject:
            return static_cast<temporal::ArithmeticOverflow::Value>(c_enum);
        default:
            std::abort();
    }
}
#endif // TEMPORAL_ArithmeticOverflow_HPP
