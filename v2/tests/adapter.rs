use datetime_format_converter_v2::adapter::Adapter;
use serde_json::{json, Value};
use tezbar_extension_sdk::{Extension, ExtensionPaths, TestContext};
fn context() -> TestContext {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(".qa/adapter")
        .join(uuid::Uuid::new_v4().to_string());
    TestContext::new(
        ExtensionPaths {
            package_dir: root.clone(),
            data_dir: root.join("data"),
            temp_dir: root.join("tmp"),
        },
        &[
            "clipboard.pasteText",
            "clipboard.read",
            "clipboard.write",
            "window.visibility",
        ],
    )
}
#[tokio::test]
async fn fixed_clock_adapter_preserves_relative_defaults_and_automatic_paste() {
    let c = context();
    c.reply(Ok(json!({"ok":true})));
    let a = Adapter::fixed(1730426187456, chrono_tz::UTC);
    let mut state = json!({});
    let p = json!({"sessionId":"relative","argumentValues":{"prompt":"tomorrow","format":"YYYY-MM-DD HH:mm:ss.SSS"},"preferences":{}});
    let r = c
        .run(a.handle(
            &c.context,
            "insert-natural-language-date",
            "extension:run-command",
            &p,
            &mut state,
        ))
        .await
        .unwrap();
    assert_eq!(r["mode"], "no-view");
    assert_eq!(
        c.calls()[0]["params"],
        json!({"text":"2024-11-02 01:56:27.456"})
    );
    assert_eq!(c.calls().len(), 1);
}
#[tokio::test]
async fn prepared_text_survives_an_edit_and_retry() {
    let c = context();
    let a = Adapter::fixed(1730426187456, chrono_tz::UTC);
    let mut state = json!({});
    let p = json!({"sessionId":"s","preferences":{"autoPasteClipboard":false},"searchText":"now"});
    let view = a
        .handle(&c.context, "index", "extension:run-command", &p, &mut state)
        .await
        .unwrap();
    let invoke = |v: &Value| json!({"sessionId":"s","preferences":{},"actionId":v["actions"][1]["id"],"formValues":{"__viewToken":v["root"]["props"]["__draftGuard"],"__draftDirty":false,"__searchText":v["root"]["props"]["searchText"]}});
    c.reply(Ok(Value::Null));
    let recovery = c
        .run(a.handle(
            &c.context,
            "index",
            "extension:invoke-action",
            &invoke(&view),
            &mut state,
        ))
        .await
        .unwrap();
    assert_eq!(recovery["mode"], "view");
    let changed = json!({"sessionId":"s","preferences":{},"searchText":"2025-12-31"});
    let view = a
        .handle(
            &c.context,
            "index",
            "extension:search-text-changed",
            &changed,
            &mut state,
        )
        .await
        .unwrap();
    let retry = json!({"sessionId":"s","preferences":{},"actionId":view["actions"][0]["id"],"formValues":{"__viewToken":view["root"]["props"]["__draftGuard"],"__draftDirty":false,"__searchText":"2025-12-31"}});
    c.reply(Ok(json!({"ok":true})));
    c.run(a.handle(
        &c.context,
        "index",
        "extension:invoke-action",
        &retry,
        &mut state,
    ))
    .await
    .unwrap();
    assert_eq!(c.calls()[0]["params"], c.calls()[1]["params"]);
    assert_eq!(c.calls()[1]["params"], json!({"text":"2024-11-01"}));
}
