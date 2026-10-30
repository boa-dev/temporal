#ifndef TEMPORAL_DisplayOffset_D_HPP
#define TEMPORAL_DisplayOffset_D_HPP

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
    enum DisplayOffset {
      DisplayOffset_Auto = 0,
      DisplayOffset_Never = 1,
    };

    typedef struct DisplayOffset_option {union { DisplayOffset ok; }; bool is_ok; } DisplayOffset_option;
} // namespace capi
} // namespace

namespace temporal {
class DisplayOffset {
public:
    enum Value {
        Auto = 0,
        Never = 1,
    };

    DisplayOffset(): value(Value::Auto) {}

    // Implicit conversions between enum and ::Value
    constexpr DisplayOffset(Value v) : value(v) {}
    constexpr operator Value() const { return value; }
    // Prevent usage as boolean value
    explicit operator bool() const = delete;

    inline temporal::capi::DisplayOffset AsFFI() const;
    inline static temporal::DisplayOffset FromFFI(temporal::capi::DisplayOffset c_enum);
private:
    Value value;
};

} // namespace
#endif // TEMPORAL_DisplayOffset_D_HPP
