#ifndef TEMPORAL_Provider_HPP
#define TEMPORAL_Provider_HPP

#include "Provider.d.hpp"

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

    typedef struct temporal_Provider_new_zoneinfo64_result {union {temporal::capi::Provider* ok; }; bool is_ok;} temporal_Provider_new_zoneinfo64_result;
    temporal_Provider_new_zoneinfo64_result temporal_Provider_new_zoneinfo64(temporal::diplomat::capi::DiplomatU32View data);

    temporal::capi::Provider* temporal_Provider_new_compiled(void);

    temporal::capi::Provider* temporal_Provider_empty(void);

    void temporal_Provider_destroy(Provider* self);

    } // extern "C"
} // namespace capi
} // namespace

inline temporal::diplomat::result<std::unique_ptr<temporal::Provider>, std::monostate> temporal::Provider::new_zoneinfo64(temporal::diplomat::span<const uint32_t> data DIPLOMAT_LIFETIME_BOUND) {
    auto result = temporal::capi::temporal_Provider_new_zoneinfo64({data.data(), data.size()});
    return result.is_ok ? temporal::diplomat::result<std::unique_ptr<temporal::Provider>, std::monostate>(temporal::diplomat::Ok<std::unique_ptr<temporal::Provider>>(std::unique_ptr<temporal::Provider>(temporal::Provider::FromFFI(result.ok)))) : temporal::diplomat::result<std::unique_ptr<temporal::Provider>, std::monostate>(temporal::diplomat::Err<std::monostate>());
}

inline std::unique_ptr<temporal::Provider> temporal::Provider::new_compiled() {
    auto result = temporal::capi::temporal_Provider_new_compiled();
    return std::unique_ptr<temporal::Provider>(temporal::Provider::FromFFI(result));
}

inline std::unique_ptr<temporal::Provider> temporal::Provider::empty() {
    auto result = temporal::capi::temporal_Provider_empty();
    return std::unique_ptr<temporal::Provider>(temporal::Provider::FromFFI(result));
}

inline const temporal::capi::Provider* temporal::Provider::AsFFI() const {
    return reinterpret_cast<const temporal::capi::Provider*>(this);
}

inline temporal::capi::Provider* temporal::Provider::AsFFI() {
    return reinterpret_cast<temporal::capi::Provider*>(this);
}

inline const temporal::Provider* temporal::Provider::FromFFI(const temporal::capi::Provider* ptr) {
    return reinterpret_cast<const temporal::Provider*>(ptr);
}

inline temporal::Provider* temporal::Provider::FromFFI(temporal::capi::Provider* ptr) {
    return reinterpret_cast<temporal::Provider*>(ptr);
}

inline void temporal::Provider::operator delete(void* ptr) {
    temporal::capi::temporal_Provider_destroy(reinterpret_cast<temporal::capi::Provider*>(ptr));
}


#endif // TEMPORAL_Provider_HPP
