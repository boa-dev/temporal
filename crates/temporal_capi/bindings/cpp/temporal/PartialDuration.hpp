#ifndef TEMPORAL_PartialDuration_HPP
#define TEMPORAL_PartialDuration_HPP

#include "PartialDuration.d.hpp"

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

    bool temporal_PartialDuration_is_empty(temporal::capi::PartialDuration self);

    } // extern "C"
} // namespace capi
} // namespace

inline bool temporal::PartialDuration::is_empty() const {
    auto result = temporal::capi::temporal_PartialDuration_is_empty(this->AsFFI());
    return result;
}


inline temporal::capi::PartialDuration temporal::PartialDuration::AsFFI() const {
    return temporal::capi::PartialDuration {
        /* .years = */ years.has_value() ? (temporal::diplomat::capi::OptionI64{ { years.value() }, true }) : (temporal::diplomat::capi::OptionI64{ {}, false }),
        /* .months = */ months.has_value() ? (temporal::diplomat::capi::OptionI64{ { months.value() }, true }) : (temporal::diplomat::capi::OptionI64{ {}, false }),
        /* .weeks = */ weeks.has_value() ? (temporal::diplomat::capi::OptionI64{ { weeks.value() }, true }) : (temporal::diplomat::capi::OptionI64{ {}, false }),
        /* .days = */ days.has_value() ? (temporal::diplomat::capi::OptionI64{ { days.value() }, true }) : (temporal::diplomat::capi::OptionI64{ {}, false }),
        /* .hours = */ hours.has_value() ? (temporal::diplomat::capi::OptionI64{ { hours.value() }, true }) : (temporal::diplomat::capi::OptionI64{ {}, false }),
        /* .minutes = */ minutes.has_value() ? (temporal::diplomat::capi::OptionI64{ { minutes.value() }, true }) : (temporal::diplomat::capi::OptionI64{ {}, false }),
        /* .seconds = */ seconds.has_value() ? (temporal::diplomat::capi::OptionI64{ { seconds.value() }, true }) : (temporal::diplomat::capi::OptionI64{ {}, false }),
        /* .milliseconds = */ milliseconds.has_value() ? (temporal::diplomat::capi::OptionI64{ { milliseconds.value() }, true }) : (temporal::diplomat::capi::OptionI64{ {}, false }),
        /* .microseconds = */ microseconds.has_value() ? (temporal::diplomat::capi::OptionF64{ { microseconds.value() }, true }) : (temporal::diplomat::capi::OptionF64{ {}, false }),
        /* .nanoseconds = */ nanoseconds.has_value() ? (temporal::diplomat::capi::OptionF64{ { nanoseconds.value() }, true }) : (temporal::diplomat::capi::OptionF64{ {}, false }),
    };
}

inline temporal::PartialDuration temporal::PartialDuration::FromFFI(temporal::capi::PartialDuration c_struct) {
    return temporal::PartialDuration {
        /* .years = */ c_struct.years.is_ok ? std::optional(c_struct.years.ok) : std::nullopt,
        /* .months = */ c_struct.months.is_ok ? std::optional(c_struct.months.ok) : std::nullopt,
        /* .weeks = */ c_struct.weeks.is_ok ? std::optional(c_struct.weeks.ok) : std::nullopt,
        /* .days = */ c_struct.days.is_ok ? std::optional(c_struct.days.ok) : std::nullopt,
        /* .hours = */ c_struct.hours.is_ok ? std::optional(c_struct.hours.ok) : std::nullopt,
        /* .minutes = */ c_struct.minutes.is_ok ? std::optional(c_struct.minutes.ok) : std::nullopt,
        /* .seconds = */ c_struct.seconds.is_ok ? std::optional(c_struct.seconds.ok) : std::nullopt,
        /* .milliseconds = */ c_struct.milliseconds.is_ok ? std::optional(c_struct.milliseconds.ok) : std::nullopt,
        /* .microseconds = */ c_struct.microseconds.is_ok ? std::optional(c_struct.microseconds.ok) : std::nullopt,
        /* .nanoseconds = */ c_struct.nanoseconds.is_ok ? std::optional(c_struct.nanoseconds.ok) : std::nullopt,
    };
}


#endif // TEMPORAL_PartialDuration_HPP
