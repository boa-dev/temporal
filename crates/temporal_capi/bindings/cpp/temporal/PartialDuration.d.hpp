#ifndef TEMPORAL_PartialDuration_D_HPP
#define TEMPORAL_PartialDuration_D_HPP

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
    struct PartialDuration {
      temporal::diplomat::capi::OptionI64 years;
      temporal::diplomat::capi::OptionI64 months;
      temporal::diplomat::capi::OptionI64 weeks;
      temporal::diplomat::capi::OptionI64 days;
      temporal::diplomat::capi::OptionI64 hours;
      temporal::diplomat::capi::OptionI64 minutes;
      temporal::diplomat::capi::OptionI64 seconds;
      temporal::diplomat::capi::OptionI64 milliseconds;
      temporal::diplomat::capi::OptionF64 microseconds;
      temporal::diplomat::capi::OptionF64 nanoseconds;
    };

    typedef struct PartialDuration_option {union { PartialDuration ok; }; bool is_ok; } PartialDuration_option;
} // namespace capi
} // namespace


namespace temporal {
struct PartialDuration {
    std::optional<int64_t> years;
    std::optional<int64_t> months;
    std::optional<int64_t> weeks;
    std::optional<int64_t> days;
    std::optional<int64_t> hours;
    std::optional<int64_t> minutes;
    std::optional<int64_t> seconds;
    std::optional<int64_t> milliseconds;
    std::optional<double> microseconds;
    std::optional<double> nanoseconds;

  inline bool is_empty() const;

    inline temporal::capi::PartialDuration AsFFI() const;
    inline static temporal::PartialDuration FromFFI(temporal::capi::PartialDuration c_struct);
};

} // namespace
#endif // TEMPORAL_PartialDuration_D_HPP
