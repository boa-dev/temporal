#ifndef TEMPORAL_RelativeTo_HPP
#define TEMPORAL_RelativeTo_HPP

#include "RelativeTo.d.hpp"

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <functional>
#include <optional>
#include <cstdlib>
#include "PlainDate.hpp"
#include "ZonedDateTime.hpp"
#include "diplomat_runtime.hpp"


namespace temporal {
namespace capi {

} // namespace capi
} // namespace


inline temporal::capi::RelativeTo temporal::RelativeTo::AsFFI() const {
    return temporal::capi::RelativeTo {
        /* .date = */ date ? date->AsFFI() : nullptr,
        /* .zoned = */ zoned ? zoned->AsFFI() : nullptr,
    };
}

inline temporal::RelativeTo temporal::RelativeTo::FromFFI(temporal::capi::RelativeTo c_struct) {
    return temporal::RelativeTo {
        /* .date = */ temporal::PlainDate::FromFFI(c_struct.date),
        /* .zoned = */ temporal::ZonedDateTime::FromFFI(c_struct.zoned),
    };
}


#endif // TEMPORAL_RelativeTo_HPP
