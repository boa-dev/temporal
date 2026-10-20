#ifndef TEMPORAL_PartialDate_HPP
#define TEMPORAL_PartialDate_HPP

#include "PartialDate.d.hpp"

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <functional>
#include <optional>
#include <cstdlib>
#include "AnyCalendarKind.hpp"
#include "diplomat_runtime.hpp"


namespace temporal {
namespace capi {

} // namespace capi
} // namespace


inline temporal::capi::PartialDate temporal::PartialDate::AsFFI() const {
    return temporal::capi::PartialDate {
        /* .year = */ year.has_value() ? (temporal::diplomat::capi::OptionI32{ { year.value() }, true }) : (temporal::diplomat::capi::OptionI32{ {}, false }),
        /* .month = */ month.has_value() ? (temporal::diplomat::capi::OptionU8{ { month.value() }, true }) : (temporal::diplomat::capi::OptionU8{ {}, false }),
        /* .month_code = */ {month_code.data(), month_code.size()},
        /* .day = */ day.has_value() ? (temporal::diplomat::capi::OptionU8{ { day.value() }, true }) : (temporal::diplomat::capi::OptionU8{ {}, false }),
        /* .era = */ {era.data(), era.size()},
        /* .era_year = */ era_year.has_value() ? (temporal::diplomat::capi::OptionI32{ { era_year.value() }, true }) : (temporal::diplomat::capi::OptionI32{ {}, false }),
        /* .calendar = */ calendar.AsFFI(),
    };
}

inline temporal::PartialDate temporal::PartialDate::FromFFI(temporal::capi::PartialDate c_struct) {
    return temporal::PartialDate {
        /* .year = */ c_struct.year.is_ok ? std::optional(c_struct.year.ok) : std::nullopt,
        /* .month = */ c_struct.month.is_ok ? std::optional(c_struct.month.ok) : std::nullopt,
        /* .month_code = */ std::string_view(c_struct.month_code.data, c_struct.month_code.len),
        /* .day = */ c_struct.day.is_ok ? std::optional(c_struct.day.ok) : std::nullopt,
        /* .era = */ std::string_view(c_struct.era.data, c_struct.era.len),
        /* .era_year = */ c_struct.era_year.is_ok ? std::optional(c_struct.era_year.ok) : std::nullopt,
        /* .calendar = */ temporal::AnyCalendarKind::FromFFI(c_struct.calendar),
    };
}


#endif // TEMPORAL_PartialDate_HPP
