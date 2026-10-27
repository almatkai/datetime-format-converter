"""Produces an UNAPPLIED generic host patch; never reads the app working files."""
import pathlib,difflib,shutil
base=pathlib.Path(__file__).parent/'host';out=pathlib.Path(__file__).parents[1]/'.qa/host-after'
shutil.copytree(base,out,dirs_exist_ok=True)
def edit(p,old,new):
 f=out/p;s=f.read_text();assert old in s,(p,old);f.write_text(s.replace(old,new))
reg='src-tauri/src/native_core/extensions/registry.rs'
# Retain canonical preference/uninstall path, keep the verified active resource path separate.
edit(reg,'    path: PathBuf,\n    manifest:','    path: PathBuf,\n    resource_path: PathBuf,\n    manifest:')
edit(reg,'            let mut manifest = if native {','            let mut resource_path = path.clone();\n            let mut manifest = if native {')
edit(reg,'Ok(package) => package.manifest,','Ok(package) => { resource_path = package.dir; package.manifest },')
edit(reg,'                path,\n                manifest,','                path,\n                resource_path,\n                manifest,')
edit(reg,'        path: root.clone(),','        path: root.clone(),\n        resource_path: root.clone(),') if '        path: root.clone(),' in (out/reg).read_text() else None
edit(reg,'        path,\n        manifest: Value::Null,','        resource_path: path.clone(),\n        path,\n        manifest: Value::Null,')
# All direct Installed struct constructions must be reviewed by the parent on application.
s=(out/reg).read_text();start=s.index('fn icon_path(');end=s.index('fn command_icon_data(',start)
s=s[:start]+s[start:end].replace('extension.path.join(', 'extension.resource_path.join(')+s[end:];(out/reg).write_text(s)
# Verified aliases are exact, case-sensitive, command-covered and ambiguity rejecting.
alias='''pub(super) fn verified_alias(ctx: &CoreContext, requested: &str, command: &str) -> Result<Option<String>, String> {
    if requested.starts_with("com.tezbar.native.") || requested.trim() != requested { return Ok(None); }
    let mut matches = Vec::new();
    for e in collect(ctx) {
        if !e.id.starts_with("com.tezbar.native.") || !e.manifest["healthError"].is_null() { continue; }
        if !e.manifest["replaces"].as_array().is_some_and(|a|a.iter().any(|v|v.as_str()==Some(requested))) { continue; }
        let Ok(package) = super::process_install::resolve(ctx, &e.id) else { continue; };
        if !package.manifest["replaces"].as_array().is_some_and(|a|a.iter().any(|v|v.as_str()==Some(requested))) { continue; }
        let covered = package.manifest["commands"].as_array().is_some_and(|a|a.iter().any(|c|c["name"]==command && c["runtime"]=="rust" && compatible(c,true)));
        matches.push((e.id,covered));
    }
    if matches.len()>1 { return Err("Ambiguous verified extension alias".into()); }
    Ok(matches.pop().and_then(|(id,covered)|covered.then_some(id)))
}

'''
edit(reg,'pub(super) fn installed_command(',alias+'pub(super) fn installed_command(')
# installed_command canonical is invoked by native.launch after alias resolution.
native='src-tauri/src/native_core/extensions/native.rs'
edit(native,'if requested.starts_with("com.tezbar.native.") || v2_runtime::slug(requested).is_some()', 'if requested.starts_with("com.tezbar.native.") || v2_runtime::slug(requested).is_some() || match registry::verified_alias(ctx, requested, body["commandName"].as_str().unwrap_or("")) { Ok(v) => v.is_some(), Err(e) => return Some(Ok(json!({"ok":false,"message":e}))) }')
edit(native,'        } else {\n            v2_runtime::canonical_id(v2_runtime::slug(requested).ok_or("Unknown extension")?)\n        };','        } else if let Some(id) = registry::verified_alias(ctx, requested, command)? {\n            id\n        } else {\n            v2_runtime::canonical_id(v2_runtime::slug(requested).ok_or("Unknown extension")?)\n        };')
# List drafts: immediate local invalidation, A-B-A must trigger another parse.
lst='src/renderer/src/raycast-api/list-runtime.tsx'
edit(lst,'  onLoadMore,\n}: {','  onLoadMore,\n  onDraftChange,\n}: {')
edit(lst,'  onLoadMore: () => Promise<void> | void\n','  onLoadMore: () => Promise<void> | void\n  onDraftChange?: (value: { searchText: string; dirty: boolean; token: unknown }) => void\n')
edit(lst,'  const [selected, setSelected] = useState(0)','  const [draftRevision, setDraftRevision] = useState(0)\n  const draftDirty = useRef(false)\n  const draftText = useRef(query)\n  const [selected, setSelected] = useState(0)')
edit(lst,"      setQuery(root.props.searchText)","      if (root.props?.__draftGuard && draftDirty.current && root.props.searchText !== draftText.current) return\n      setQuery(root.props.searchText)\n      draftText.current = root.props.searchText\n      draftDirty.current = false\n      onDraftChange?.({ searchText: root.props.searchText, dirty: false, token: root.props?.__draftGuard })")
edit(lst,'  }, [root.props?.searchText])','  }, [root.props?.searchText, root.props?.__draftGuard])')
edit(lst,'    if (query === lastSentQuery.current) return','    if (query === lastSentQuery.current && !draftDirty.current) return')
edit(lst,'  }, [query, onSearchTextChanged])','  }, [query, onSearchTextChanged, draftRevision])')
edit(lst,'  const shouldShowSearch =\n    isColorConversionSurface ||','  const shouldShowSearch = root.props?.__disableSearch !== true && (\n    isColorConversionSurface ||')
edit(lst,'    !root.props?.navigationTitle\n','    !root.props?.navigationTitle)\n')
# Exact current source's search input handler, all actions route through Surface wrapper.
s=(out/lst).read_text();needle='onChange={(event) => {\n                  setQuery(event.target.value)';assert needle in s
s=s.replace(needle,'''onChange={(event) => {
                  const text = event.target.value
                  draftText.current = text
                  draftDirty.current = true
                  onDraftChange?.({ searchText: text, dirty: true, token: root.props?.__draftGuard })
                  setDraftRevision((value) => value + 1)
                  setQuery(text)''');(out/lst).write_text(s)
edit(lst,"    : hasQuery\n      ? 'No matching applications'", "    : hasQuery && !hasServerSearch\n      ? 'No matching applications'")
edit(lst,"    : hasQuery\n      ? 'Try a different app name or keyword.'", "    : hasQuery && !hasServerSearch\n      ? 'Try a different app name or keyword.'")
surface='src/renderer/src/raycast-api/index.tsx'
edit(surface,'    onInvokeAction,\n  } = props','    onInvokeAction: invokeHost,\n  } = props\n  const listDraft = useRef<{ searchText: string; dirty: boolean; token: unknown } | null>(null)\n  const onInvokeAction = useCallback((id: string, values?: Record<string, unknown>) => {\n    const draft = listDraft.current\n    return invokeHost(id, root.props?.__draftGuard ? { ...values, __searchText: draft?.searchText ?? root.props.searchText, __draftDirty: draft?.dirty ?? false, __viewToken: draft?.token ?? root.props.__draftGuard } : values)\n  }, [invokeHost, root])')
edit(surface,'            <ListRuntime\n','            <ListRuntime\n              onDraftChange={(draft) => { listDraft.current = draft }}\n')
runtime='src/renderer/ExtensionRuntimeView.tsx'
edit(runtime,"              if (result.mode === 'no-view') {\n                setState((prev) => ({ ...prev, message: undefined }))\n                return\n              }","              if (result.mode === 'no-view') {\n                await window.tezbar.extensionDisposeSession({ sessionId: state.sessionId })\n                onBack()\n                return\n              }")
patch=[]
for f in sorted(out.rglob('*')):
 if f.is_file():
  rel=f.relative_to(out);old=(base/rel).read_text();new=f.read_text()
  if old!=new:patch.extend(difflib.unified_diff(old.splitlines(True),new.splitlines(True),fromfile='a/'+str(rel),tofile='b/'+str(rel)))
dest=pathlib.Path(__file__).parents[1]/'integration';dest.mkdir(exist_ok=True);(dest/'host-prerequisites.patch').write_text(''.join(patch))
print('Generated unapplied patch', len(''.join(patch)), 'bytes')
