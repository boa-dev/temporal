#ifndef TEMPORAL_PartialDateTime_HPP
#define TEMPORAL_PartialDateTime_HPP

#include "PartialDateTime.d.hpp"

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <functional>
#include <optional>
#include <cstdlib>
#include "PartialDate.hpp"
#include "PartialTime.hpp"
#include "diplomat_runtime.hpp"


namespace temporal {
namespace capi {

} // namespace capi
} // namespace


inline temporal::capi::PartialDateTime temporal::PartialDateTime::AsFFI() const {
    return temporal::capi::PartialDateTime {
        /* .date = */ date.AsFFI(),
        /* .time = */ time.AsFFI(),
    };
}

inline temporal::PartialDateTime temporal::PartialDateTime::FromFFI(temporal::capi::PartialDateTime c_struct) {
    return temporal::PartialDateTime {
        /* .date = */ temporal::PartialDate::FromFFI(c_struct.date),
        /* .time = */ temporal::PartialTime::FromFFI(c_struct.time),
    };
}


#endif // TEMPORAL_PartialDateTime_HPP
