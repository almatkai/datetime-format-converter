# Parent integration review

Standalone schema 3 / Rust / process / protocol 1. SDK dependency is the immutable
HTTPS Git revision in Cargo.toml/Cargo.lock, version 0.1.1, MSRV 1.85. No app,
Tauri or local SDK path dependency exists. Dev `artifacts:{}` cannot be used as a
catalog release pin. This clone remains uncommitted and unstaged for the parent.

Read-only host evidence is extracted exclusively with `git show` from
`ea100cee3f69711de12485ba061950c5fc65258f`; `qa/host-pins.json` hashes every fixture.
The dirty host checkout was not read as source, built, changed, switched or stashed.
The independent SDK checkout was read with exact `git show` only.

`integration/host-prerequisites.patch` is **UNAPPLIED** and contains only generic
prerequisites used by these commands:

1. Keep active-version resource paths separate from canonical preference/uninstall
   roots in registry discovery, resolving both command icons and extension icons
   from the verified active package. It does not move preference files.
2. Resolve exact replacement IDs only through verified installed manifests and
   covered Rust commands on this platform. Ambiguous, unverified and lookalike
   aliases do not acquire execution authority. No datetime-specific branch is added.
3. Pass current List draft text, dirty state and view token through the shared row,
   keyboard and action-panel dispatch. Invalidate synchronously on local editing;
   A-B-A must trigger another server search. Preserve a newer local draft against
   an older search response. Hide search on a no-view recovery surface. Retain the
   server's own empty-view feedback rather than generic application-search text.
4. Dispose and leave a view after an action returns no-view. This is used after
   Copy/Paste and after an explicit recovery retry; it prevents stale visible actions.

No Form, controlled dropdown, Detail MouseEvent or unrelated host ACK patch is
bundled. These commands do not use Form/Detail/dropdown surfaces. Fixture tests
execute the actual used ListRuntime, ExtensionRuntimeSurface and
ExtensionRuntimeView functions before/after the patch; inert imports replace
external UI/OS boundaries only. Actual registry icon functions, alias resolver,
manifest validator, launcher validator and runtime hello predicate are compiled
as extracted Rust fixtures. Full host compilation is **not run**.

Adapter authority is bounded and separate from SDK's success-only cloned JSON
state. Fresh opaque view/action tokens invalidate all previous actions; the
renderer must supply the current token/text with `__draftDirty:false`. Actions
consume authority before any await. Late ACK or SDK cancellation cannot restore
consumed actions. Current prepared bytes/phase survive ordinary view cancellation
and can be explicitly retried; successful write plus failed hide retries only
visibility. The SDK owns session registration, disposal, scoped cancellation and
its bounded stdin pump; no detached domain work is created.

Reverse calls are limited to the declared methods:

| Method | Params | Accepted successful producer result |
| --- | --- | --- |
| clipboard.read | `{}` | string (including empty string) |
| clipboard.write | `{text}` | closed `{ok:true,message?:string}` |
| clipboard.pasteText | `{text}` | closed `{ok:true,message?:string}` |
| settings.open | `{surface:"settings-extensions"}` | closed `{ok:true,message?:string}` |
| window.visibility | `{visible:false}` | **null only** at this host pin |

`tests/host_extracts.rs` is generated from actual pinned clipboard/platform match
arms. Those arms execute against synthetic boundary implementations to produce
`.qa/host-producers.json`; the real JSONL mock host uses these defaults. Thus bool,
null or extra/contradictory fields cannot accidentally pass generic ACK tests.
There is no extra write/hide around pasteText and no selected-text call.

No persistence or migration paths are needed: the source has only host-owned
preferences. No extension-local settings, clipboard history or parser cache is
written. Host dataDir/tempDir remain profile scoped and QA isolates them under
the ignored clone directory. Native IANA/OS locale behavior, real GUI, real
clipboard/focus/paste, installed alias routing and Windows runtime acceptance
remain parent/platform review work. No patch is applied by any QA script.
