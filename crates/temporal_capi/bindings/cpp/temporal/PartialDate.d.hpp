#ifndef TEMPORAL_PartialDate_D_HPP
#define TEMPORAL_PartialDate_D_HPP

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <functional>
#include <optional>
#include <cstdlib>
#include "AnyCalendarKind.d.hpp"
#include "diplomat_runtime.hpp"
namespace temporal {
class AnyCalendarKind;
} // namespace temporal



namespace temporal {
namespace capi {
    struct PartialDate {
      temporal::diplomat::capi::OptionI32 year;
      temporal::diplomat::capi::OptionU8 month;
      temporal::diplomat::capi::DiplomatStringView month_code;
      temporal::diplomat::capi::OptionU8 day;
      temporal::diplomat::capi::DiplomatStringView era;
      temporal::diplomat::capi::OptionI32 era_year;
      temporal::capi::AnyCalendarKind calendar;
    };

    typedef struct PartialDate_option {union { PartialDate ok; }; bool is_ok; } PartialDate_option;
} // namespace capi
} // namespace


namespace temporal {
struct PartialDate {
    std::optional<int32_t> year;
    std::optional<uint8_t> month;
    std::string_view month_code;
    std::optional<uint8_t> day;
    std::string_view era;
    std::optional<int32_t> era_year;
    temporal::AnyCalendarKind calendar;

    inline temporal::capi::PartialDate AsFFI() const;
    inline static temporal::PartialDate FromFFI(temporal::capi::PartialDate c_struct);
};

} // namespace
#endif // TEMPORAL_PartialDate_D_HPP
