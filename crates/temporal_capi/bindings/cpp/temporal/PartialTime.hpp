#ifndef TEMPORAL_PartialTime_HPP
#define TEMPORAL_PartialTime_HPP

#include "PartialTime.d.hpp"

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


inline temporal::capi::PartialTime temporal::PartialTime::AsFFI() const {
    return temporal::capi::PartialTime {
        /* .hour = */ hour.has_value() ? (temporal::diplomat::capi::OptionU8{ { hour.value() }, true }) : (temporal::diplomat::capi::OptionU8{ {}, false }),
        /* .minute = */ minute.has_value() ? (temporal::diplomat::capi::OptionU8{ { minute.value() }, true }) : (temporal::diplomat::capi::OptionU8{ {}, false }),
        /* .second = */ second.has_value() ? (temporal::diplomat::capi::OptionU8{ { second.value() }, true }) : (temporal::diplomat::capi::OptionU8{ {}, false }),
        /* .millisecond = */ millisecond.has_value() ? (temporal::diplomat::capi::OptionU16{ { millisecond.value() }, true }) : (temporal::diplomat::capi::OptionU16{ {}, false }),
        /* .microsecond = */ microsecond.has_value() ? (temporal::diplomat::capi::OptionU16{ { microsecond.value() }, true }) : (temporal::diplomat::capi::OptionU16{ {}, false }),
        /* .nanosecond = */ nanosecond.has_value() ? (temporal::diplomat::capi::OptionU16{ { nanosecond.value() }, true }) : (temporal::diplomat::capi::OptionU16{ {}, false }),
    };
}

inline temporal::PartialTime temporal::PartialTime::FromFFI(temporal::capi::PartialTime c_struct) {
    return temporal::PartialTime {
        /* .hour = */ c_struct.hour.is_ok ? std::optional(c_struct.hour.ok) : std::nullopt,
        /* .minute = */ c_struct.minute.is_ok ? std::optional(c_struct.minute.ok) : std::nullopt,
        /* .second = */ c_struct.second.is_ok ? std::optional(c_struct.second.ok) : std::nullopt,
        /* .millisecond = */ c_struct.millisecond.is_ok ? std::optional(c_struct.millisecond.ok) : std::nullopt,
        /* .microsecond = */ c_struct.microsecond.is_ok ? std::optional(c_struct.microsecond.ok) : std::nullopt,
        /* .nanosecond = */ c_struct.nanosecond.is_ok ? std::optional(c_struct.nanosecond.ok) : std::nullopt,
    };
}


#endif // TEMPORAL_PartialTime_HPP
