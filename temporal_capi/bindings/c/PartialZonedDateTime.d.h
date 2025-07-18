#ifndef PartialZonedDateTime_D_H
#define PartialZonedDateTime_D_H

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include "diplomat_runtime.h"

#include "PartialDate.d.h"
#include "PartialTime.d.h"
#include "TimeZone.d.h"




typedef struct PartialZonedDateTime {
  PartialDate date;
  PartialTime time;
  bool has_utc_designator;
  OptionStringView offset;
  const TimeZone* timezone;
} PartialZonedDateTime;

typedef struct PartialZonedDateTime_option {union { PartialZonedDateTime ok; }; bool is_ok; } PartialZonedDateTime_option;



#endif // PartialZonedDateTime_D_H
