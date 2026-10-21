#ifndef TEMPORAL_ErrorKind_HPP
#define TEMPORAL_ErrorKind_HPP

#include "ErrorKind.d.hpp"

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

inline temporal::capi::ErrorKind temporal::ErrorKind::AsFFI() const {
    return static_cast<temporal::capi::ErrorKind>(value);
}

inline temporal::ErrorKind temporal::ErrorKind::FromFFI(temporal::capi::ErrorKind c_enum) {
    switch (c_enum) {
        case temporal::capi::ErrorKind_Generic:
        case temporal::capi::ErrorKind_Type:
        case temporal::capi::ErrorKind_Range:
        case temporal::capi::ErrorKind_Syntax:
        case temporal::capi::ErrorKind_Assert:
            return static_cast<temporal::ErrorKind::Value>(c_enum);
        default:
            std::abort();
    }
}
#endif // TEMPORAL_ErrorKind_HPP
