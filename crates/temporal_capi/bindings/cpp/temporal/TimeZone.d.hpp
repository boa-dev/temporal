#ifndef TEMPORAL_TimeZone_D_HPP
#define TEMPORAL_TimeZone_D_HPP

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
struct TemporalError;
struct TimeZone;
} // namespace temporal



namespace temporal {
namespace capi {
    struct TimeZone {
      int16_t offset_minutes;
      size_t resolved_id;
      size_t normalized_id;
      bool is_iana_id;
    };

    typedef struct TimeZone_option {union { TimeZone ok; }; bool is_ok; } TimeZone_option;
} // namespace capi
} // namespace


namespace temporal {
/**
 * A type representing a time zone over FFI.
 *
 * It is not recommended to directly manipulate the fields of this type.
 */
struct TimeZone {
    int16_t offset_minutes;
    size_t resolved_id;
    size_t normalized_id;
    bool is_iana_id;

  inline static temporal::diplomat::result<temporal::TimeZone, temporal::TemporalError> try_from_identifier_str(std::string_view ident);

  inline static temporal::diplomat::result<temporal::TimeZone, temporal::TemporalError> try_from_identifier_str_with_provider(std::string_view ident, const temporal::Provider& p);

  inline static temporal::diplomat::result<temporal::TimeZone, temporal::TemporalError> try_from_offset_str(std::string_view ident);

  inline static temporal::diplomat::result<temporal::TimeZone, temporal::TemporalError> try_from_str(std::string_view ident);

  inline static temporal::diplomat::result<temporal::TimeZone, temporal::TemporalError> try_from_str_with_provider(std::string_view ident, const temporal::Provider& p);

  inline std::string identifier() const;
  template<typename W>
  inline void identifier_write(W& writeable_output) const;

  inline temporal::diplomat::result<std::string, temporal::TemporalError> identifier_with_provider(const temporal::Provider& p) const;
  template<typename W>
  inline temporal::diplomat::result<std::monostate, temporal::TemporalError> identifier_with_provider_write(const temporal::Provider& p, W& writeable_output) const;

  inline static temporal::TimeZone utc();

  inline static temporal::diplomat::result<temporal::TimeZone, temporal::TemporalError> utc_with_provider(const temporal::Provider& p);

  /**
   * Create a TimeZone that represents +00:00
   *
   * This is the only way to infallibly make a TimeZone without compiled_data,
   * and can be used as a fallback.
   */
  inline static temporal::TimeZone zero();

  /**
   * Get the primary time zone identifier corresponding to this time zone
   */
  inline temporal::diplomat::result<temporal::TimeZone, temporal::TemporalError> primary_identifier() const;

  inline temporal::diplomat::result<temporal::TimeZone, temporal::TemporalError> primary_identifier_with_provider(const temporal::Provider& p) const;

    inline temporal::capi::TimeZone AsFFI() const;
    inline static temporal::TimeZone FromFFI(temporal::capi::TimeZone c_struct);
};

} // namespace
#endif // TEMPORAL_TimeZone_D_HPP
