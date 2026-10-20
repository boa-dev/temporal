#ifndef TEMPORAL_Sign_D_HPP
#define TEMPORAL_Sign_D_HPP

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
    enum Sign {
      Sign_Positive = 1,
      Sign_Zero = 0,
      Sign_Negative = -1,
    };

    typedef struct Sign_option {union { Sign ok; }; bool is_ok; } Sign_option;
} // namespace capi
} // namespace

namespace temporal {
class Sign {
public:
    enum Value {
        Positive = 1,
        Zero = 0,
        Negative = -1,
    };

    Sign(): value(Value::Zero) {}

    // Implicit conversions between enum and ::Value
    constexpr Sign(Value v) : value(v) {}
    constexpr operator Value() const { return value; }
    // Prevent usage as boolean value
    explicit operator bool() const = delete;

    inline temporal::capi::Sign AsFFI() const;
    inline static temporal::Sign FromFFI(temporal::capi::Sign c_enum);
private:
    Value value;
};

} // namespace
#endif // TEMPORAL_Sign_D_HPP
