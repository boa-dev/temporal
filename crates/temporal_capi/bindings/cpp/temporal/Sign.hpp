#ifndef TEMPORAL_Sign_HPP
#define TEMPORAL_Sign_HPP

#include "Sign.d.hpp"

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

inline temporal::capi::Sign temporal::Sign::AsFFI() const {
    return static_cast<temporal::capi::Sign>(value);
}

inline temporal::Sign temporal::Sign::FromFFI(temporal::capi::Sign c_enum) {
    switch (c_enum) {
        case temporal::capi::Sign_Positive:
        case temporal::capi::Sign_Zero:
        case temporal::capi::Sign_Negative:
            return static_cast<temporal::Sign::Value>(c_enum);
        default:
            std::abort();
    }
}
#endif // TEMPORAL_Sign_HPP
