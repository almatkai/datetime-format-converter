//! Session authority lives outside SDK's staged JSON state. Consume actions BEFORE
//! awaiting host effects so cancellation/unknown acknowledgement cannot replay them.
use crate::{date, natural};
use chrono::Utc;
use chrono_tz::Tz;
use serde_json::{json, Value};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use tezbar_extension_sdk::{
    no_view, unchanged, view, BoxFuture, CoreResult, Extension, ExtensionContext,
};
#[derive(Clone)]
struct Effect {
    method: &'static str,
    text: String,
}
#[derive(Default)]
struct Authority {
    draft: String,
    values: Vec<String>,
    valid: bool,
    message: Option<String>,
    actions: HashMap<String, Action>,
    token: String,
    prepared: Option<Effect>,
    pending: bool,
    load_failed: bool,
    revision: u64,
    natural: bool,
}
#[derive(Clone)]
enum Action {
    Effect(Effect),
    Retry,
    Load,
    Settings,
}
type Shared = Arc<Mutex<Authority>>;
#[derive(Default)]
pub struct Adapter {
    sessions: Mutex<HashMap<String, (Shared, ExtensionContext)>>,
    fixed: Option<(i64, Tz)>,
}
impl Adapter {
    pub fn fixed(now: i64, tz: Tz) -> Self {
        Self {
            fixed: Some((now, tz)),
            ..Self::default()
        }
    }
    fn clock(&self) -> Result<(i64, Tz), String> {
        self.fixed
            .map(Ok)
            .unwrap_or_else(|| Ok((Utc::now().timestamp_millis(), date::system_zone()?)))
    }
    fn session(&self, id: &str) -> Result<Shared, String> {
        self.sessions
            .lock()
            .map_err(|_| "Session lock poisoned")?
            .get(id)
            .map(|(authority, _)| authority.clone())
            .ok_or("Session no longer active".into())
    }
    async fn prune_cancelled_scopes(&self) -> Result<(), String> {
        // Event-triggered, bounded ownership cleanup covers cancellation in the
        // narrow SDK commit boundary after our open handler has returned.
        let owners = self
            .sessions
            .lock()
            .map_err(|_| "Session lock poisoned")?
            .iter()
            .map(|(id, (a, ctx))| (id.clone(), a.clone(), ctx.clone()))
            .collect::<Vec<_>>();
        for (id, authority, owner) in owners {
            let cancelled = tokio::select! {biased;_=owner.cancelled()=>true,_=async{}=>false};
            if cancelled {
                let mut sessions = self.sessions.lock().map_err(|_| "Session lock poisoned")?;
                if sessions
                    .get(&id)
                    .is_some_and(|(a, _)| Arc::ptr_eq(a, &authority))
                {
                    sessions.remove(&id);
                }
            }
        }
        Ok(())
    }
    fn render(&self, a: &mut Authority) -> Result<Value, String> {
        a.actions.clear();
        a.revision = a
            .revision
            .checked_add(1)
            .ok_or("Session revision exhausted")?;
        let token = uuid::Uuid::new_v4().to_string();
        a.token = token.clone();
        let mut actions = vec![];
        let mut rows = vec![];
        let mut add = |title: &str, action: Action| {
            let id = format!("{token}:{}", actions.len());
            actions.push(json!({"id":id,"title":title,"kind":"callback"}));
            a.actions.insert(id.clone(), action);
            id
        };
        if a.prepared.is_some() {
            let retry = add(
                if a.prepared
                    .as_ref()
                    .is_some_and(|e| e.method == "host.clipboard.pasteText")
                {
                    "Retry Paste — May Repeat Previous Paste"
                } else {
                    "Retry Exact Prepared Text"
                },
                Action::Retry,
            );
            rows.push(json!({"type":"List.Item","props":{"id":"retry","title":"Delivery outcome is unknown; retry may repeat it","actionIds":[retry]}}));
        } else if a.load_failed {
            let retry = add("Retry Clipboard Read", Action::Load);
            rows.push(json!({"type":"List.Item","props":{"id":"load","title":"Clipboard could not be read. Edit the search or retry.","actionIds":[retry]}}));
        } else {
            for (i, text) in a.values.iter().enumerate() {
                let copy = add(
                    "Copy to Clipboard",
                    Action::Effect(Effect {
                        method: "host.clipboard.write",
                        text: text.clone(),
                    }),
                );
                let paste = add(
                    "Paste",
                    Action::Effect(Effect {
                        method: "host.clipboard.pasteText",
                        text: text.clone(),
                    }),
                );
                rows.push(json!({"type":"List.Item","props":{"id":format!("format-{i}"),"title":text,"actionIds":[copy,paste]}}));
            }
        }
        if rows.is_empty() {
            let settings = if a.valid {
                Some(add("Open Extension Preferences", Action::Settings))
            } else {
                None
            };
            rows.push(json!({"type":"List.EmptyView","props":{"title":if a.valid{"No Date Time Format is selected in preferences"}else{"An error occurred"},"description":if a.valid{"press ↵ to Open Extension Preferences"}else{"This is not a time format."},"actionIds":settings.into_iter().collect::<Vec<_>>()}}));
        }
        let mut result = view(
            if a.natural {
                "Insert Natural Language Date"
            } else {
                "Format Date"
            },
            json!({"type":"List","props":{"searchText":a.draft,"searchBarPlaceholder":"Enter a time or date","__hasServerSearch":!a.natural,"__disableSearch":a.natural,"__draftGuard":token},"children":rows}),
            actions,
        );
        result["refreshMs"] = Value::Null;
        if let Some(message) = &a.message {
            result["message"] = json!(message)
        }
        Ok(result)
    }
    fn parse(&self, a: &mut Authority, preferences: &Value) -> Result<(), String> {
        let (now, tz) = self.clock()?;
        let mut enabled = [true; 14];
        for (i, key) in date::FORMAT_KEYS.iter().enumerate() {
            enabled[i] = preference_bool(preferences, key, true)?
        }
        let r = date::convert(&a.draft, now, tz, &enabled)?;
        a.values = r.values;
        a.valid = r.valid;
        Ok(())
    }
    async fn load(&self, ctx: &ExtensionContext, authority: &Shared, prefs: &Value) -> CoreResult {
        // A canceled future leaves load_failed=true and no old actions authorized.
        {
            let mut a = authority.lock().map_err(|_| "Session lock poisoned")?;
            a.actions.clear();
            a.load_failed = true;
        }
        let reply = ctx.host("host.clipboard.read", json!({})).await;
        let mut a = authority.lock().map_err(|_| "Session lock poisoned")?;
        match reply.and_then(|v| {
            v.as_str()
                .map(str::to_owned)
                .ok_or("Clipboard read must return a string".into())
        }) {
            Ok(text) => {
                if date::checked_text(&text).is_err() {
                    a.message = Some(
                        "Clipboard exceeds the 4096-byte input bound. Edit the search or retry."
                            .into(),
                    );
                    return self.render(&mut a);
                }
                a.draft = text;
                a.load_failed = false;
                a.message = None;
                if !a.draft.is_empty() {
                    if let Err(e) = self.parse(&mut a, prefs) {
                        a.valid = false;
                        a.values.clear();
                        a.message = Some(e)
                    }
                }
            }
            Err(_) => {
                a.message = Some("Failed to read clipboard. Edit the search or retry.".into())
            }
        }
        self.render(&mut a)
    }
    async fn effect(
        &self,
        ctx: &ExtensionContext,
        authority: &Shared,
        effect: Effect,
    ) -> CoreResult {
        {
            let mut a = authority.lock().map_err(|_| "Session lock poisoned")?;
            a.actions.clear();
            a.prepared = Some(effect.clone());
            a.pending = true;
            a.message =
                Some("Delivery outcome is unknown. Explicit retry may repeat the effect.".into());
        }
        let params = if effect.method == "host.window.visibility" {
            json!({"visible":false})
        } else {
            json!({"text":effect.text})
        };
        let mut result = ctx
            .host(effect.method, params)
            .await
            .and_then(|v| ack(effect.method, &v));
        if result.is_ok() && effect.method == "host.clipboard.write" {
            // Raycast Action.CopyToClipboard closes the window after copying.
            // Preserve this ACK'd write stage before awaiting window visibility.
            {
                let mut a = authority.lock().map_err(|_| "Session lock poisoned")?;
                a.prepared = Some(Effect {
                    method: "host.window.visibility",
                    text: effect.text.clone(),
                });
            }
            result = ctx
                .host("host.window.visibility", json!({"visible":false}))
                .await
                .and_then(|v| ack("host.window.visibility", &v));
        }
        let mut a = authority.lock().map_err(|_| "Session lock poisoned")?;
        a.pending = false;
        match result {
            Ok(()) => {
                a.prepared = None;
                a.message = None;
                Ok(no_view(if effect.method == "host.clipboard.pasteText" {
                    "Pasted text"
                } else {
                    "Copied to clipboard"
                }))
            }
            Err(_) => self.render(&mut a),
        }
    }
}
struct OpenGuard<'a> {
    adapter: &'a Adapter,
    id: String,
    keep: bool,
}
impl Drop for OpenGuard<'_> {
    fn drop(&mut self) {
        if !self.keep {
            if let Ok(mut sessions) = self.adapter.sessions.lock() {
                sessions.remove(&self.id);
            }
        }
    }
}
fn preference_bool(p: &Value, key: &str, default: bool) -> Result<bool, String> {
    match p.get(key) {
        None => Ok(default),
        Some(Value::Bool(v)) => Ok(*v),
        Some(Value::String(s)) if s == "true" => Ok(true),
        Some(Value::String(s)) if s == "false" => Ok(false),
        _ => Err(format!("Invalid boolean preference {key}")),
    }
}
fn argument<'a>(p: &'a Value, key: &str) -> Result<&'a str, String> {
    match p["argumentValues"].get(key) {
        None => Ok(""),
        Some(Value::String(s)) => {
            date::checked_text(s)?;
            Ok(s)
        }
        _ => Err(format!("Argument {key} must be text")),
    }
}
/// Closed producer-specific replies. Window null is valid ONLY for visibility.
pub fn ack(method: &str, value: &Value) -> Result<(), String> {
    if method == "host.window.visibility" {
        return if value.is_null() {
            Ok(())
        } else {
            Err("Invalid visibility acknowledgement".into())
        };
    }
    let object = value.as_object().ok_or("Invalid host acknowledgement")?;
    if object.get("ok") != Some(&Value::Bool(true))
        || object.keys().any(|k| k != "ok" && k != "message")
        || object.get("message").is_some_and(|v| !v.is_string())
    {
        Err("Invalid host acknowledgement".into())
    } else {
        Ok(())
    }
}
impl Extension for Adapter {
    fn id(&self) -> &'static str {
        "com.tezbar.native.datetime-format-converter"
    }
    fn version(&self) -> &'static str {
        "2.0.0"
    }
    fn commands(&self) -> &'static [&'static str] {
        &["index", "insert-natural-language-date"]
    }
    fn handle<'a>(
        &'a self,
        ctx: &'a ExtensionContext,
        command: &'a str,
        operation: &'a str,
        p: &'a Value,
        _state: &'a mut Value,
    ) -> BoxFuture<'a, CoreResult> {
        Box::pin(async move {
            let id = p["sessionId"].as_str().ok_or("Missing session id")?;
            if operation == "extension:dispose-session" {
                self.sessions
                    .lock()
                    .map_err(|_| "Session lock poisoned")?
                    .remove(id);
                return Ok(no_view("Closed"));
            }
            let prefs = &p["preferences"];
            if operation == "extension:run-command" {
                self.prune_cancelled_scopes().await?;
                let authority = Arc::new(Mutex::new(Authority {
                    natural: command == "insert-natural-language-date",
                    ..Authority::default()
                }));
                {
                    let mut map = self.sessions.lock().map_err(|_| "Session lock poisoned")?;
                    if map.len() >= 128 {
                        return Err("Adapter session capacity reached".into());
                    }
                    if map
                        .insert(id.into(), (authority.clone(), ctx.clone()))
                        .is_some()
                    {
                        return Err("Duplicate session".into());
                    }
                }
                let mut guard = OpenGuard {
                    adapter: self,
                    id: id.into(),
                    keep: false,
                };
                let result = if command == "insert-natural-language-date" {
                    let prompt = argument(p, "prompt")?;
                    let prompt = if prompt.is_empty() { "today" } else { prompt };
                    let (now, tz) = self.clock()?;
                    // Source parses BEFORE validating its format, and inserts automatically.
                    let parsed = natural::parse(prompt, now, tz)?;
                    if let Some(ms) = parsed {
                        let explicit = argument(p, "format")?;
                        let format = if explicit.is_empty() {
                            match prefs.get("naturalLanguageDateFormat") {
                                None => "YYYY-MM-DD",
                                Some(Value::String(s)) => s,
                                _ => {
                                    return Err(
                                        "Natural language format preference must be text".into()
                                    )
                                }
                            }
                        } else {
                            explicit
                        };
                        date::checked_text(format)?;
                        if !date::valid_format(format) {
                            Ok(no_view("Invalid date format. Edit the format argument or extension preference and run again."))
                        } else {
                            let text = date::format(ms, tz, format, false)?;
                            self.effect(
                                ctx,
                                &authority,
                                Effect {
                                    method: "host.clipboard.pasteText",
                                    text,
                                },
                            )
                            .await
                        }
                    } else {
                        Ok(no_view(
                            "Failed to parse date prompt. Edit the prompt argument and run again.",
                        ))
                    }
                } else if command == "index" {
                    if let Some(text) = p["searchText"].as_str().filter(|s| !s.is_empty()) {
                        let mut a = authority.lock().map_err(|_| "Session lock poisoned")?;
                        a.draft = text.into();
                        if let Err(e) = self.parse(&mut a, prefs) {
                            a.message = Some(e)
                        }
                        self.render(&mut a)
                    } else if preference_bool(prefs, "autoPasteClipboard", true)? {
                        self.load(ctx, &authority, prefs).await
                    } else {
                        let mut a = authority.lock().map_err(|_| "Session lock poisoned")?;
                        self.render(&mut a)
                    }
                } else {
                    Err("Unknown command".into())
                };
                guard.keep = result.is_ok();
                return result;
            }
            let authority = self.session(id)?;
            match operation {
                "extension:refresh-session" => {
                    let mut a = authority.lock().map_err(|_| "Session lock poisoned")?;
                    if a.prepared.is_some() || a.load_failed {
                        self.render(&mut a)
                    } else {
                        Ok(unchanged())
                    }
                }
                "extension:load-more" => Ok(unchanged()),
                "extension:search-text-changed" => {
                    if command != "index" {
                        return Err("Command has no search".into());
                    }
                    let draft = p["searchText"].as_str().ok_or("Missing searchText")?;
                    let mut a = authority.lock().map_err(|_| "Session lock poisoned")?;
                    a.actions.clear();
                    date::checked_text(draft)?;
                    a.draft = draft.into();
                    a.load_failed = false;
                    a.message = None;
                    if a.prepared.is_some() {
                        a.values.clear();
                        a.message=Some("Previous delivery may have completed. Retry uses the exact prepared bytes.".into())
                    } else if let Err(e) = self.parse(&mut a, prefs) {
                        a.valid = false;
                        a.values.clear();
                        a.message = Some(e)
                    }
                    self.render(&mut a)
                }
                "extension:invoke-action" => {
                    let action = {
                        let mut a = authority.lock().map_err(|_| "Session lock poisoned")?;
                        let fields = &p["formValues"];
                        if fields["__viewToken"].as_str() != Some(a.token.as_str())
                            || fields["__draftDirty"] != false
                            || fields["__searchText"].as_str() != Some(a.draft.as_str())
                        {
                            return Err("Search draft changed; wait for its current results".into());
                        }
                        let id = p["actionId"].as_str().ok_or("Missing actionId")?;
                        let action = a
                            .actions
                            .remove(id)
                            .ok_or("Action is stale or already consumed")?;
                        a.actions.clear();
                        action
                    };
                    match action {
                        Action::Effect(e) => self.effect(ctx, &authority, e).await,
                        Action::Retry => {
                            let e = authority
                                .lock()
                                .map_err(|_| "Session lock poisoned")?
                                .prepared
                                .clone()
                                .ok_or("No prepared text")?;
                            self.effect(ctx, &authority, e).await
                        }
                        Action::Load => self.load(ctx, &authority, prefs).await,
                        Action::Settings => {
                            let result = ctx
                                .host(
                                    "host.settings.open",
                                    json!({"surface":"settings-extensions"}),
                                )
                                .await
                                .and_then(|v| ack("host.settings.open", &v));
                            let mut a = authority.lock().map_err(|_| "Session lock poisoned")?;
                            a.message = Some(
                                if result.is_ok() {
                                    "Reopen this command after changing preferences."
                                } else {
                                    "Failed to open extension preferences; retry."
                                }
                                .into(),
                            );
                            self.render(&mut a)
                        }
                    }
                }
                _ => Err("Unsupported operation".into()),
            }
        })
    }
}
