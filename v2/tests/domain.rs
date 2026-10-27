use datetime_format_converter_v2::{date, natural};
use serde_json::{json, Value};
#[test]
fn whole_original_tsx_vectors() {
    let rows: Vec<Value> =
        serde_json::from_str(include_str!("../evidence/source-vectors.json")).unwrap();
    for r in rows {
        let tz = r["tz"].as_str().unwrap().parse().unwrap();
        let now = r["now"].as_i64().unwrap();
        let text = r["text"].as_str().unwrap();
        let actual = if r["kind"] == "natural" {
            let parsed =
                natural::parse(if text.is_empty() { "today" } else { text }, now, tz).unwrap();
            let format = r["format"]
                .as_str()
                .filter(|s| !s.is_empty())
                .unwrap_or("YYYY-MM-DD");
            match parsed {
                None => json!({"failure":"parse"}),
                Some(_) if !date::valid_format(format) => json!({"failure":"format"}),
                Some(ms) => json!({"text": date::format(ms,tz,format,false).unwrap()}),
            }
        } else {
            let mut enabled = [true; 14];
            if let Some(a) = r["enabled"].as_array() {
                for (i, e) in enabled.iter_mut().enumerate() {
                    *e = a[i].as_bool().unwrap()
                }
            }
            serde_json::to_value(date::convert(text, now, tz, &enabled).unwrap()).unwrap()
        };
        assert_eq!(actual, r["expected"], "input: {r}");
    }
}
#[test]
fn bounded_inputs_and_extremes_do_not_panic() {
    let now = 1730426187456;
    for tz in [
        chrono_tz::UTC,
        chrono_tz::America::New_York,
        chrono_tz::Pacific::Apia,
    ] {
        assert!(date::convert(&"x".repeat(4097), now, tz, &[true; 14]).is_err());
        assert!(natural::parse(&"tomorrow ".repeat(1000), now, tz).is_err());
        assert!(natural::parse("in 10001 days", now, tz).is_err());
        for text in [
            "💥",
            "\0",
            "你好 tomorrow",
            "éFriday",
            "in -1e100 years",
            "2024-99-99",
            "-999999999999999999",
            " ",
            "[link](https://example.invalid)",
            "<img src=x onerror=alert(1)>",
        ] {
            let _ = date::convert(text, now, tz, &[true; 14]);
            let _ = natural::parse(text, now, tz);
        }
    }
}
#[test]
fn method_specific_closed_ack_contract() {
    use datetime_format_converter_v2::adapter::ack;
    for method in [
        "host.clipboard.write",
        "host.clipboard.pasteText",
        "host.settings.open",
    ] {
        for value in [
            Value::Null,
            json!(true),
            json!(false),
            json!({}),
            json!({"ok":false}),
            json!({"ok":true,"error":"x"}),
            json!({"ok":true,"message":null}),
            json!({"ok":true,"extra":0}),
        ] {
            assert!(ack(method, &value).is_err());
        }
        assert!(ack(method, &json!({"ok":true})).is_ok());
        assert!(ack(method, &json!({"ok":true,"message":"synthetic"})).is_ok());
    }
    assert!(ack("host.window.visibility", &Value::Null).is_ok());
    assert!(ack("host.window.visibility", &json!({"ok":true})).is_err());
}
