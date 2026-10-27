# Compatibility and resource boundaries

The native implementation translates the locked dayjs parse/format paths and
chrono-node English casual parser/refiner pipeline. **2025 exact source vectors
are finite evidence, not universal JavaScript Date/chrono-node equivalence.**
The corpus executes both whole own-source TSX modules and their actual locked
libraries under Node 26.7.0 with fixed Date/Date.now, TZ and English module locale.
It does not run a live Raycast, Bun, browser, Tezbar app or operating-system paste.

Inputs and format strings are limited to 4096 UTF-8 bytes, relative amounts to
10000 units, matches to 128 per parser, combined candidates to 512, sessions to
128, and output civil years to -9999..9999. Fancy-regex additionally enforces its
bounded backtracking budget. These explicit resource rejections can differ from
V1. Oversized clipboard content is rejected before retaining it. Editing/retrying
ordinary bounded parse/load errors remains available. No full JS TimeClip range
or universal Date.parse fallback is claimed.

The Format Date command preserves V1's unusual *two-step* numeric path: validate
`dayjs(string)` first, then JS Number coercion and **UTF-16 string length** 10
(seconds) / 13 (milliseconds); other numeric lengths yield an empty selected-format
result. It does not use a magnitude threshold. Negative values, scientific/radix
coercion, whitespace, Date normalization and fractional seconds have explicit
regressions. The native implementation applies finite f64/TimeClip guards before
integer conversion, then the civil-year output bound.

Local dayjs ISO-like strings normalize overflowing month/day/time fields as the
pinned source does, including Date's constructor years 0..99 -> 1900..1999.
The explicit-offset ISO path validates fields and normalizes permissible day
rollover. RFC/English Date string fallback supports common pinned families in
`src/date.rs`; other engine-dependent Date.parse string grammars can be rejected
where V1's JS engine accepts them. Such fallback behavior is not a new natural
language parser in Format Date. NLP is provided by its separate original command.

All 14 Format Date preference outputs preserve order and actual loaded plugins.
Natural Date uses basic dayjs tokens and its original literal token-stripping
validation; ISO `T`, brackets and week-year tokens are rejected by that validation.
No week-year, timezone-name or ordinal plugins are silently enabled. The original
advanced-format week/year/name tokens would throw for missing plugins; the native
formatter returns an error if called with them. V1 exposes no custom Format Date
format argument, so these cannot appear among the 14 ordinary outputs.

Natural Date retains source defaults: `today` carries reference time, date-only
parsed dates/weekday defaults use noon, no `forwardDate` option is supplied, and
closest weekdays/years, explicit next/last week rules, month-end overflow,
relative-unit fractions, range-start interpretation and timezone abbreviation
quirks are exercised. One/double-digit fractional seconds are raw milliseconds
in the actual locked parsers, not conventional tenths/hundredths. In particular
ISO `-00:30` and following-time-range offsets have source quirks; corpus fixtures
preserve observed outputs instead of correcting them.

The source has no selected timezone preference. `TZ` can select a valid IANA zone
in tests; production uses the current system IANA zone when TZ is absent. Unknown
zones fail explicitly. Native IANA data is pinned to 2025b. JavaScript/system ICU
may have a different IANA revision outside the exercised zones/dates. Fold choice
is the earlier instant; the dayjs Date path advances gaps by transition size,
while chrono-node's date validity filter rejects gap-normalized components.
The corpus covers UTC, New York, Berlin, Kathmandu, Lord Howe and Apia, including
one-hour/half-hour/date-line transitions. Historical display offsets use dayjs's
15-minute rounding while instants retain IANA second-resolution offsets.

There is no clock-dependent cache or stored history. Each parse samples one clock
and zone; prepared effect bytes intentionally stay fixed for explicit retries.
Preference snapshots are immutable within an SDK session. Reopen after a settings
change. V1's no-dependency clipboard effect repeatedly reads on renders; V2 reads
once on open and offers explicit Retry on load failure, preventing a late read
from overwriting a user's edited search. Source debug logging of input is omitted.
Raycast's Copy HUD is represented by the no-view result message in protocol 1.

An acknowledged host effect is not rolled back by SDK state cancellation. Unknown
ACK/cancel retains the exact operation phase and bytes for a view session; no
implicit paste retry occurs. A canceled no-view **open**, disposed session, EOF,
process exit, profile switch or host teardown can lose that volatile checkpoint.
A later new command may repeat a prior paste. There is no persistent clipboard
journal, fabricated V1 migration, or promise of rollback/exactly-once OS effects.
An event-triggered bounded scope cleanup reclaims canceled opens even at the SDK
commit boundary; there are no periodic tasks, timers, leases or detached helpers.
