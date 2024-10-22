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
