#ifndef TEMPORAL_PartialZonedDateTime_D_HPP
#define TEMPORAL_PartialZonedDateTime_D_HPP

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <functional>
#include <optional>
#include <cstdlib>
#include "PartialDate.d.hpp"
#include "PartialTime.d.hpp"
#include "TimeZone.d.hpp"
#include "diplomat_runtime.hpp"
namespace temporal {
struct PartialDate;
struct PartialTime;
struct TimeZone;
} // namespace temporal



namespace temporal {
namespace capi {
    struct PartialZonedDateTime {
      temporal::capi::PartialDate date;
      temporal::capi::PartialTime time;
      temporal::diplomat::capi::OptionStringView offset;
      temporal::capi::TimeZone_option timezone;
    };

    typedef struct PartialZonedDateTime_option {union { PartialZonedDateTime ok; }; bool is_ok; } PartialZonedDateTime_option;
} // namespace capi
} // namespace


namespace temporal {
struct PartialZonedDateTime {
    temporal::PartialDate date;
    temporal::PartialTime time;
    std::optional<std::string_view> offset;
    std::optional<temporal::TimeZone> timezone;

    inline temporal::capi::PartialZonedDateTime AsFFI() const;
    inline static temporal::PartialZonedDateTime FromFFI(temporal::capi::PartialZonedDateTime c_struct);
};

} // namespace
#endif // TEMPORAL_PartialZonedDateTime_D_HPP
