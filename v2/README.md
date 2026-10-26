# Datetime Format Converter — standalone Rust V2

Feature branch for parent review. **Not installed, published, or released.**
Only `README.md` and `v2/` differ from the pinned V1 checkout. Source manifest
`artifacts` is intentionally `{}`. The parent owns publication and catalog pins.

Both original commands are implemented:

- **Format Date** (`index`): clipboard auto-fill, editable date/timestamp search,
  all 14 original preference-controlled formats in their original order, Copy,
  Paste, invalid-date feedback and extension preferences.
- **Insert Natural Language Date** (`insert-natural-language-date`): the actual
  offline English casual `chrono-node` workflow. Prompt defaults to `today`,
  parsing precedes format validation, explicit format overrides the original
  preference (`YYYY-MM-DD` by default), then text is pasted automatically.
  Parse/format errors remain no-view results; edit launch arguments and rerun.

No JS/Node runtime, AI, network service, new credentials, history, database,
selection read, timezone preference or additional confirmation is introduced.
The original source has no timezone selector and imports no non-English locale;
local dates use the OS IANA zone and dayjs's actual English default formatting.
NLP grammar, dictionaries and timezone abbreviations come from the locked library.
Read [LIMITS.md](LIMITS.md) before interpreting finite parity evidence.

Copy writes text, then hides the main window (the locked Raycast API documents
this behavior). Its two stages are checkpointed separately. Paste calls only
`host.clipboard.pasteText`, which already owns clipboard write/hide/focus/paste.
An uncertain effect retains its exact prepared bytes and requires explicit Retry;
Retry Paste warns it may repeat a previous paste. Editing does not silently
reparse/re-paste that checkpoint. No automatic effect replay occurs.

The pinned host needs the **unapplied generic prerequisites** in
[integration/host-prerequisites.patch](integration/host-prerequisites.patch).
Actions require the current renderer draft/token fields and safely reject an
unpatched renderer. See [INTEGRATION.md](INTEGRATION.md) for scope and evidence.
No host, SDK, profile, sibling extension or catalog working files were modified.

## Pins

| Input | Immutable identity |
| --- | --- |
| Own source | `6633da332d70a4a34a2b317309ef61f2c0e4d74d` |
| Original Raycast assets/upstream | `6734d85992290066fd31a7f5bee3cca3d3545a1a` |
| Independent SDK 0.1.1 | HTTPS Git `e955911f0bf5b558f9d3eb5183df0adc95c22af2` |
| Read-only host fixture | `ea100cee3f69711de12485ba061950c5fc65258f` |
| Locked JS libraries used by V1 | `chrono-node 2.8.3`, `dayjs 1.11.13` |
| Native timezone database | `chrono-tz 0.10.4`, IANA `2025b` |

Native ID is `com.tezbar.native.datetime-format-converter`. Exact replacements
are `com.tezbar.datetime-format-converter` and `raycast.datetime-format-converter`;
both original command names are retained. Host catalog identity/repository and
both Recommended commands are confirmed in the pinned catalog fixture.

Both commands use the **same original PNG**, without conversion:
`e5c2567099d4ce8aaee8f22a3a8e391e51619adb86897e15ee9094d26011f09f`.
[PROVENANCE.json](PROVENANCE.json) records URLs, source hashes and extraction data.
Authentic notices live in `notices/`; absent notice text is recorded explicitly.

## Reproduce

From this repository's root (Rust stable 1.98; MSRV 1.85):

```sh
python3 v2/qa/bootstrap.py
python3 v2/qa/setup-schema.py
node v2/qa/extract-grammar.cjs
python3 v2/qa/make-vectors.py
node v2/qa/source-oracle.cjs
python3 v2/qa/prepare-host-patch.py
python3 v2/qa/generate-host-tests.py
cargo fmt --manifest-path v2/Cargo.toml --check
cargo test --locked --manifest-path v2/Cargo.toml
cargo clippy --locked --manifest-path v2/Cargo.toml --all-targets -- -D warnings
cargo +1.85.0 test --locked --manifest-path v2/Cargo.toml
cargo build --locked --manifest-path v2/Cargo.toml
python3 v2/qa/compare.py
python3 v2/qa/process-tests.py
node v2/qa/renderer-tests.cjs
node v2/qa/validate-schema.cjs
npm --prefix v2/.qa/v1 test
npm --prefix v2/.qa/v1 run typecheck
```

Generated host tests are rustfmt-formatted after regeneration by
`cargo fmt --manifest-path v2/Cargo.toml`. Host tests derive exact producer replies
before process tests use them. All synthetic test homes/temp/data and disposable
V1 dependencies are beneath ignored `v2/.qa/`. Source JS is executed only in a
closed VM with synthetic producers; unmocked OS/network/browser access fails.

The development archive is reproducible without any sibling build dependency:

```sh
CARGO_HOME="$PWD/v2/.qa/isolated/cargo-home" \
CARGO_TARGET_DIR="$PWD/v2/.qa/isolated/target" \
cargo build --locked --release --manifest-path v2/Cargo.toml --bin datetime-format-converter
python3 v2/qa/package.py
```

The script validates every ZIP member/inventory hash, extracts beneath `.qa/`,
then executes the extracted production binary through the real SDK pipe suite.
It creates no artifact URL or release metadata. ZIP, target, node_modules and
other disposable outputs are ignored. [QA.md](QA.md) records results and gaps;
[INDEX.json](INDEX.json) inventories all Git-visible files except itself.
