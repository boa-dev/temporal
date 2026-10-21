#ifndef TEMPORAL_TransitionDirection_HPP
#define TEMPORAL_TransitionDirection_HPP

#include "TransitionDirection.d.hpp"

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

inline temporal::capi::TransitionDirection temporal::TransitionDirection::AsFFI() const {
    return static_cast<temporal::capi::TransitionDirection>(value);
}

inline temporal::TransitionDirection temporal::TransitionDirection::FromFFI(temporal::capi::TransitionDirection c_enum) {
    switch (c_enum) {
        case temporal::capi::TransitionDirection_Next:
        case temporal::capi::TransitionDirection_Previous:
            return static_cast<temporal::TransitionDirection::Value>(c_enum);
        default:
            std::abort();
    }
}
#endif // TEMPORAL_TransitionDirection_HPP
