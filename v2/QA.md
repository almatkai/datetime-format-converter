# Validation evidence

Development feature branch `feat/native-v2-rust`, parent-owned review/publication.
Source HEAD remains `6633da332d70a4a34a2b317309ef61f2c0e4d74d`; there is no new
commit. The index is empty. Intended final status: ` M README.md`, `?? v2/`.
Everything else in V1 is verified byte-identical by `evidence/v1-hashes.json`.
No app/profile/SDK/sibling/catalog/ledger changes, installation or deletion occur.

## Executed

| Check | Result |
| --- | --- |
| Whole own TSX + actual locked dayjs/chrono source oracle | **2025/2025 byte-exact finite vectors** |
| Deterministic domain vectors in Rust tests | Passed on Rust 1.98.0 and MSRV 1.85.0 |
| Native adapter tests | Fixed-clock automatic natural-date paste; prepared bytes retained across edit/retry |
| Closed ACK validation | String-only read, closed object ACKs, null only for window visibility; malformed/contradictory negatives rejected |
| Real SDK production process pipes | 10 scenarios (listed below) |
| Actual pinned host producer extracts | Compiled/executed against inert OS boundaries; feed process mock defaults |
| Actual pinned host manifest/launcher validator + runtime hello | Passed positive and negative fixtures |
| Actual pinned registry functions | Active-version icon path regression before/after; verified exact covered aliases, ambiguity and lookalike rejection |
| Actual List/Surface/Runtime renderer functions | Before/after draft, A-B-A, no-view disposal/back and no-search recovery fixtures passed |
| Exact pinned extension-v3 schema | Manifest valid; 10 schema-negative fixtures rejected |
| Formatting / strict Clippy all targets | `cargo fmt --check`; `cargo clippy --locked --all-targets -- -D warnings` passed |
| Disposable exact V1 root-lock tests/typecheck | Both passed; no V1 dependency installation in root |
| Fresh isolated Cargo home HTTPS SDK build | Native ARM64 release executable built with immutable SDK revision; no sibling path dependency |
| Development ZIP | Closed file inventory, all member hashes, executable hash and native ARM64 architecture verified |
| Extracted production executable | Same 10 real SDK pipe scenarios passed from extracted ZIP |
| Windows GNU compilation check | `cargo check --target x86_64-pc-windows-gnu` passed |
| Windows GNU linked build attempt | Blocked: missing `x86_64-w64-mingw32-dlltool`; no tooling installed |
| Windows MSVC compilation attempt | Blocked: target `core`/`std` not installed; no target installed |

The 2025 vectors exercise all 14 output formats and each preference in isolation;
empty/now/invalid/numeric inputs; seconds/milliseconds length quirks, signs,
whitespace, fractions, scientific/radix coercion, rollover and leap years; ISO,
RFC and common English Date paths; both commands' argument/default validation;
relative reference time, natural month/day/weekday/year, next/last/this, relative
units and fractions, AM/PM, offsets/abbreviations and range starts. Six zones are
UTC, America/New_York, Europe/Berlin, Asia/Kathmandu, Australia/Lord_Howe and
Pacific/Apia. Fixed references include January end, leap day and spring/autumn DST.
Expected snapshots never use the current wall clock. Source actions' copy/paste
payloads are inspected from the whole original render tree. They are synthetic
strings, never user clipboard contents.

The process suite uses actual SDK JSONL pipes and production domain execution:

1. Search A-B-A, stale/consumed action IDs and current draft/token guards; Copy/Paste.
2. Preferences, no formats selected, settings action and invalid parse editing.
3. Automatic no-view natural-date paste and parse-before-format error ordering.
4. Clipboard STRING contract, invalid producer shape and edit/retry recovery.
5. Unknown/malformed ACKs and explicit exact-byte retries.
6. Cancellation, late ACK and refresh recovery without automatic replay.
7. 140 canceled opens followed by a successful open (SDK + adapter capacity cleanup).
8. 128-session capacity rejection followed by disposal/recovery.
9. EOF during a pending no-view paste; bounded SDK process exit.
10. Copy write followed by null visibility ACK; failed visibility retry does not
    rewrite clipboard text.

`evidence/process-checks.json`, `renderer-checks.json`,
`extracted-process-checks.json`, `development-package.json`, `rust-notices.json`,
`PROVENANCE.json` and `qa/host-pins.json` are inspectable evidence. The extracted
fixture tests are generated from the pinned files; limited Clippy allowances
preserve original host `return`/closure spelling and boolean expressions rather than
changing the extracts. Parent verification added the narrow `nonminimal_bool`
fixture allowance after Rust 1.85 Clippy rejected an unchanged pinned host expression.
All synthetic data/home/temp/package outputs remain under ignored `v2/.qa/` or
`v2/dist/`. The closed source VM exposes no process, filesystem, OS, browser,
network, real clipboard or settings producers. Tests make no live AI/product
network calls and request no credentials. Dependency/asset/license downloads are
build preparation only. No parser/clipboard values are logged by the product.

## Not run / review gates

Full Tezbar/Tauri build, applying the host patch, live GUI, live clipboard,
frontmost-app paste/focus, installed alias/icon/preferences behavior, native
Windows linking/runtime, OS-specific font/date behavior and clean-profile release
acceptance are **not run**. Manifest verification stays `not-run` on macOS and
Windows; source `artifacts:{}` has no fabricated URLs or hashes. The inactive
`qa/ci-example.yml` is an example, not a completed CI run or release.

No Form/Detail fixture or prerequisite is claimed: these two commands use List
and no-view surfaces. No source persistence/history exists, so there is no fake
migration or profile store fixture. Unknown no-view-open delivery cannot retain
its volatile checkpoint after cancellation/process loss; see [LIMITS.md](LIMITS.md).
Finite source vectors and mocked host boundaries do not establish universal
parser, full host-build, live OS or native Windows parity.

XERJ: initial canonical-prefix index and search completed with skipped non-code
assets (all four initial code files were indexed). Final indexing outcome and
coverage are recorded in `evidence/xerj.json`; no alternate prefix, global restart
or schema mutation is used. Source claims are verified through full pinned file
reads and exact rg/Git checks, not inferred from search absence.

Final XERJ refresh failed (exit 1): the original frozen dataset cannot accept
`v2/Cargo.toml` (family docs, unsupported schema evolution). V2 search coverage
is **incomplete**. No alternative prefix/restart/schema mutation was attempted.
