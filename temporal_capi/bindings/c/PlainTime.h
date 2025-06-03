#ifndef PlainTime_H
#define PlainTime_H

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include "diplomat_runtime.h"

#include "ArithmeticOverflow.d.h"
#include "DifferenceSettings.d.h"
#include "Duration.d.h"
#include "PartialTime.d.h"
#include "RoundingMode.d.h"
#include "TemporalError.d.h"
#include "TimeDuration.d.h"
#include "ToStringRoundingOptions.d.h"
#include "Unit.d.h"

#include "PlainTime.d.h"






typedef struct temporal_rs_PlainTime_try_new_constrain_result {union {PlainTime* ok; TemporalError err;}; bool is_ok;} temporal_rs_PlainTime_try_new_constrain_result;
temporal_rs_PlainTime_try_new_constrain_result temporal_rs_PlainTime_try_new_constrain(uint8_t hour, uint8_t minute, uint8_t second, uint16_t millisecond, uint16_t microsecond, uint16_t nanosecond);

typedef struct temporal_rs_PlainTime_try_new_result {union {PlainTime* ok; TemporalError err;}; bool is_ok;} temporal_rs_PlainTime_try_new_result;
temporal_rs_PlainTime_try_new_result temporal_rs_PlainTime_try_new(uint8_t hour, uint8_t minute, uint8_t second, uint16_t millisecond, uint16_t microsecond, uint16_t nanosecond);

typedef struct temporal_rs_PlainTime_from_partial_result {union {PlainTime* ok; TemporalError err;}; bool is_ok;} temporal_rs_PlainTime_from_partial_result;
temporal_rs_PlainTime_from_partial_result temporal_rs_PlainTime_from_partial(PartialTime partial, ArithmeticOverflow_option overflow);

typedef struct temporal_rs_PlainTime_with_result {union {PlainTime* ok; TemporalError err;}; bool is_ok;} temporal_rs_PlainTime_with_result;
temporal_rs_PlainTime_with_result temporal_rs_PlainTime_with(const PlainTime* self, PartialTime partial, ArithmeticOverflow_option overflow);

typedef struct temporal_rs_PlainTime_from_utf8_result {union {PlainTime* ok; TemporalError err;}; bool is_ok;} temporal_rs_PlainTime_from_utf8_result;
temporal_rs_PlainTime_from_utf8_result temporal_rs_PlainTime_from_utf8(DiplomatStringView s);

typedef struct temporal_rs_PlainTime_from_utf16_result {union {PlainTime* ok; TemporalError err;}; bool is_ok;} temporal_rs_PlainTime_from_utf16_result;
temporal_rs_PlainTime_from_utf16_result temporal_rs_PlainTime_from_utf16(DiplomatString16View s);

uint8_t temporal_rs_PlainTime_hour(const PlainTime* self);

uint8_t temporal_rs_PlainTime_minute(const PlainTime* self);

uint8_t temporal_rs_PlainTime_second(const PlainTime* self);

uint16_t temporal_rs_PlainTime_millisecond(const PlainTime* self);

uint16_t temporal_rs_PlainTime_microsecond(const PlainTime* self);

uint16_t temporal_rs_PlainTime_nanosecond(const PlainTime* self);

typedef struct temporal_rs_PlainTime_add_result {union {PlainTime* ok; TemporalError err;}; bool is_ok;} temporal_rs_PlainTime_add_result;
temporal_rs_PlainTime_add_result temporal_rs_PlainTime_add(const PlainTime* self, const Duration* duration);

typedef struct temporal_rs_PlainTime_subtract_result {union {PlainTime* ok; TemporalError err;}; bool is_ok;} temporal_rs_PlainTime_subtract_result;
temporal_rs_PlainTime_subtract_result temporal_rs_PlainTime_subtract(const PlainTime* self, const Duration* duration);

typedef struct temporal_rs_PlainTime_add_time_duration_result {union {PlainTime* ok; TemporalError err;}; bool is_ok;} temporal_rs_PlainTime_add_time_duration_result;
temporal_rs_PlainTime_add_time_duration_result temporal_rs_PlainTime_add_time_duration(const PlainTime* self, const TimeDuration* duration);

typedef struct temporal_rs_PlainTime_subtract_time_duration_result {union {PlainTime* ok; TemporalError err;}; bool is_ok;} temporal_rs_PlainTime_subtract_time_duration_result;
temporal_rs_PlainTime_subtract_time_duration_result temporal_rs_PlainTime_subtract_time_duration(const PlainTime* self, const TimeDuration* duration);

typedef struct temporal_rs_PlainTime_until_result {union {Duration* ok; TemporalError err;}; bool is_ok;} temporal_rs_PlainTime_until_result;
temporal_rs_PlainTime_until_result temporal_rs_PlainTime_until(const PlainTime* self, const PlainTime* other, DifferenceSettings settings);

typedef struct temporal_rs_PlainTime_since_result {union {Duration* ok; TemporalError err;}; bool is_ok;} temporal_rs_PlainTime_since_result;
temporal_rs_PlainTime_since_result temporal_rs_PlainTime_since(const PlainTime* self, const PlainTime* other, DifferenceSettings settings);

bool temporal_rs_PlainTime_equals(const PlainTime* self, const PlainTime* other);

int8_t temporal_rs_PlainTime_compare(const PlainTime* one, const PlainTime* two);

typedef struct temporal_rs_PlainTime_round_result {union {PlainTime* ok; TemporalError err;}; bool is_ok;} temporal_rs_PlainTime_round_result;
temporal_rs_PlainTime_round_result temporal_rs_PlainTime_round(const PlainTime* self, Unit smallest_unit, OptionF64 rounding_increment, RoundingMode_option rounding_mode);

typedef struct temporal_rs_PlainTime_to_ixdtf_string_result {union { TemporalError err;}; bool is_ok;} temporal_rs_PlainTime_to_ixdtf_string_result;
temporal_rs_PlainTime_to_ixdtf_string_result temporal_rs_PlainTime_to_ixdtf_string(const PlainTime* self, ToStringRoundingOptions options, DiplomatWrite* write);

void temporal_rs_PlainTime_destroy(PlainTime* self);





#endif // PlainTime_H
