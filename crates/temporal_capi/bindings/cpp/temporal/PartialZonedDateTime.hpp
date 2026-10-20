#ifndef TEMPORAL_PartialZonedDateTime_HPP
#define TEMPORAL_PartialZonedDateTime_HPP

#include "PartialZonedDateTime.d.hpp"

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
#include "TimeZone.hpp"
#include "diplomat_runtime.hpp"


namespace temporal {
namespace capi {

} // namespace capi
} // namespace


inline temporal::capi::PartialZonedDateTime temporal::PartialZonedDateTime::AsFFI() const {
    return temporal::capi::PartialZonedDateTime {
        /* .date = */ date.AsFFI(),
        /* .time = */ time.AsFFI(),
        /* .offset = */ offset.has_value() ? (temporal::diplomat::capi::OptionStringView{ { {offset.value().data(), offset.value().size()} }, true }) : (temporal::diplomat::capi::OptionStringView{ {}, false }),
        /* .timezone = */ timezone.has_value() ? (temporal::capi::TimeZone_option{ { timezone.value().AsFFI() }, true }) : (temporal::capi::TimeZone_option{ {}, false }),
    };
}

inline temporal::PartialZonedDateTime temporal::PartialZonedDateTime::FromFFI(temporal::capi::PartialZonedDateTime c_struct) {
    return temporal::PartialZonedDateTime {
        /* .date = */ temporal::PartialDate::FromFFI(c_struct.date),
        /* .time = */ temporal::PartialTime::FromFFI(c_struct.time),
        /* .offset = */ c_struct.offset.is_ok ? std::optional(std::string_view(c_struct.offset.ok.data, c_struct.offset.ok.len)) : std::nullopt,
        /* .timezone = */ c_struct.timezone.is_ok ? std::optional(temporal::TimeZone::FromFFI(c_struct.timezone.ok)) : std::nullopt,
    };
}


#endif // TEMPORAL_PartialZonedDateTime_HPP
