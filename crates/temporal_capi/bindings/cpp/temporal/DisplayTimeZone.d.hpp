#ifndef TEMPORAL_DisplayTimeZone_D_HPP
#define TEMPORAL_DisplayTimeZone_D_HPP

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
    enum DisplayTimeZone {
      DisplayTimeZone_Auto = 0,
      DisplayTimeZone_Never = 1,
      DisplayTimeZone_Critical = 2,
    };

    typedef struct DisplayTimeZone_option {union { DisplayTimeZone ok; }; bool is_ok; } DisplayTimeZone_option;
} // namespace capi
} // namespace

namespace temporal {
class DisplayTimeZone {
public:
    enum Value {
        Auto = 0,
        Never = 1,
        Critical = 2,
    };

    DisplayTimeZone(): value(Value::Auto) {}

    // Implicit conversions between enum and ::Value
    constexpr DisplayTimeZone(Value v) : value(v) {}
    constexpr operator Value() const { return value; }
    // Prevent usage as boolean value
    explicit operator bool() const = delete;

    inline temporal::capi::DisplayTimeZone AsFFI() const;
    inline static temporal::DisplayTimeZone FromFFI(temporal::capi::DisplayTimeZone c_enum);
private:
    Value value;
};

} // namespace
#endif // TEMPORAL_DisplayTimeZone_D_HPP
