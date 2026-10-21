#ifndef TEMPORAL_I128Nanoseconds_HPP
#define TEMPORAL_I128Nanoseconds_HPP

#include "I128Nanoseconds.d.hpp"

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
    extern "C" {

    bool temporal_I128Nanoseconds_is_valid(temporal::capi::I128Nanoseconds self);

    } // extern "C"
} // namespace capi
} // namespace

inline bool temporal::I128Nanoseconds::is_valid() const {
    auto result = temporal::capi::temporal_I128Nanoseconds_is_valid(this->AsFFI());
    return result;
}


inline temporal::capi::I128Nanoseconds temporal::I128Nanoseconds::AsFFI() const {
    return temporal::capi::I128Nanoseconds {
        /* .high = */ high,
        /* .low = */ low,
    };
}

inline temporal::I128Nanoseconds temporal::I128Nanoseconds::FromFFI(temporal::capi::I128Nanoseconds c_struct) {
    return temporal::I128Nanoseconds {
        /* .high = */ c_struct.high,
        /* .low = */ c_struct.low,
    };
}


#endif // TEMPORAL_I128Nanoseconds_HPP
