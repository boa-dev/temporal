#ifndef TEMPORAL_Provider_D_HPP
#define TEMPORAL_Provider_D_HPP

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
namespace capi { struct Provider; }
class Provider;
} // namespace temporal



namespace temporal {
namespace capi {
    struct Provider;
} // namespace capi
} // namespace

namespace temporal {
/**
 * A time zone data provider
 */
class Provider {
public:

  /**
   * Construct a provider backed by a zoneinfo64.res file
   *
   * This failing to construct is not a Temporal error, so it just returns ()
   */
  inline static temporal::diplomat::result<std::unique_ptr<temporal::Provider>, std::monostate> new_zoneinfo64(temporal::diplomat::span<const uint32_t> data DIPLOMAT_LIFETIME_BOUND);

  inline static std::unique_ptr<temporal::Provider> new_compiled();

  /**
   * Fallback type in case construction does not work.
   */
  inline static std::unique_ptr<temporal::Provider> empty();

    inline const temporal::capi::Provider* AsFFI() const;
    inline temporal::capi::Provider* AsFFI();
    inline static const temporal::Provider* FromFFI(const temporal::capi::Provider* ptr);
    inline static temporal::Provider* FromFFI(temporal::capi::Provider* ptr);
    inline static void operator delete(void* ptr);
private:
    Provider() = delete;
    Provider(const temporal::Provider&) = delete;
    Provider(temporal::Provider&&) noexcept = delete;
    Provider operator=(const temporal::Provider&) = delete;
    Provider operator=(temporal::Provider&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};

} // namespace
#endif // TEMPORAL_Provider_D_HPP
