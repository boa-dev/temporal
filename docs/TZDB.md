# General TZDB implementation notes

Below are some logs of the logic currently at play to find the local
time records based off user provided nanoseconds (seconds).

Important to note, that currently the logs only exist for notably +/-
time zones that observe DST.

Further testing still needs to be done for time zones without any DST
transition / potentially historically abnormal edge cases.

## Empty case (AKA Spring forward)

// New York STD -> DST transition (2017-3-12)
// Record direction: DST -> STD
Search time: Seconds(1489285800)
Unresolved under LocalTimeTypeRecord { utoff: Seconds(-18000), is_dst: false, idx: 8 }
Unresolved over LocalTimeTypeRecord { utoff: Seconds(-14400), is_dst: true, idx: 4 }
previous over Seconds(10873800)
current diff Seconds(-16200)
Resolved under LocalTimeTypeRecord { utoff: Seconds(-18000), is_dst: false, idx: 8 }
Resolved over LocalTimeTypeRecord { utoff: Seconds(-14400), is_dst: true, idx: 4 }
Range: Seconds(-18000)..Seconds(-14400)
Diff value: Seconds(-16200)
Contains Result: true
Result: []

// Sydney STD -> DST transition (2017-10-1)
// Record direction: STD -> DST
Search time: Seconds(1506825000)
Unresolved under LocalTimeTypeRecord { utoff: Seconds(39600), is_dst: true, idx: 4 }
Unresolved over LocalTimeTypeRecord { utoff: Seconds(36000), is_dst: false, idx: 9 }
previous over Seconds(15762600)
current diff Seconds(37800)
Resolved under LocalTimeTypeRecord { utoff: Seconds(36000), is_dst: false, idx: 9 }
Resolved over LocalTimeTypeRecord { utoff: Seconds(39600), is_dst: true, idx: 4 }
Range: Seconds(36000)..Seconds(39600)
Diff value: Seconds(37800)
Contains Result: true
Result: []


## Duplicate case (AKA Spring backward)

// New York DST -> STD transition (2017-11-5)
Search time: Seconds(1509845400)
Unresolved under LocalTimeTypeRecord { utoff: Seconds(-14400), is_dst: true, idx: 4 }
Unresolved over LocalTimeTypeRecord { utoff: Seconds(-18000), is_dst: false, idx: 8 }
previous over Seconds(20543400)
current diff Seconds(-16200)
Resolved under LocalTimeTypeRecord { utoff: Seconds(-14400), is_dst: true, idx: 4 }
Resolved over LocalTimeTypeRecord { utoff: Seconds(-18000), is_dst: false, idx: 8 }
Range: Seconds(-18000)..Seconds(-14400)
Diff value: Seconds(-16200)
Contains Result: true
Result: [LocalTimeTypeRecord { utoff: Seconds(-14400), is_dst: true, idx: 4 }, LocalTimeTypeRecord { utoff: Seconds(-18000), is_dst: false, idx: 8 }]

// Sydney DST -> STD transition (2017-4-2)
Search time: Seconds(1491100200)
Unresolved under LocalTimeTypeRecord { utoff: Seconds(36000), is_dst: false, idx: 9 }
Unresolved over LocalTimeTypeRecord { utoff: Seconds(39600), is_dst: true, idx: 4 }
previous over Seconds(15762600)
current diff Seconds(37800)
Resolved under LocalTimeTypeRecord { utoff: Seconds(39600), is_dst: true, idx: 4 }
Resolved over LocalTimeTypeRecord { utoff: Seconds(36000), is_dst: false, idx: 9 }
Range: Seconds(36000)..Seconds(39600)
Diff value: Seconds(37800)
Contains Result: true
Result: [LocalTimeTypeRecord { utoff: Seconds(39600), is_dst: true, idx: 4 }, LocalTimeTypeRecord { utoff: Seconds(36000), is_dst: false, idx: 9 }]

## Slim format testing

`jiff_tzdb` and potentially others use a different compiled `tzif`
than operating systems.

Where operating systems use the "-b fat", smaller embedded tzifs may
use "-b slim" (it's also worth noting that slim is the default setting)

What does slim actually do? The slim formats "slims" the size of a tzif
by calculating the transition times for a smaller range. Instead of calculating
the transition times in a larger range, the tzif (and thus user) differs to
the POSIX tz string.

So in order to support "slim" format `tzif`s, we need to be able to resolve the
[POSIX tz string](glibc-posix-docs).

### Running tests / logging

While using `jiff_tzdb`, the binary search will run the below:

Running a date from 2017 using jiff:

time array length: 175
Returned idx: Err(175)

This will panic, because we have gone past the supported transition times, so
we should default to parsing the POSIX tz string.


[glibc-posix-docs]:https://sourceware.org/glibc/manual/2.40/html_node/Proleptic-TZ.html
