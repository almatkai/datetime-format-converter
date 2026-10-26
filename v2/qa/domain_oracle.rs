use datetime_format_converter_v2::{date, natural};
use std::io::{self, BufRead};
fn main() {
    for line in io::stdin().lock().lines() {
        let result = (|| -> Result<serde_json::Value, String> {
            let v: serde_json::Value = serde_json::from_str(&line.map_err(|e| e.to_string())?)
                .map_err(|e| e.to_string())?;
            let tz = v["tz"].as_str().ok_or("tz")?.parse().map_err(|_| "tz")?;
            let now = v["now"].as_i64().ok_or("now")?;
            let text = v["text"].as_str().ok_or("text")?;
            match v["kind"].as_str() {
                Some("natural") => {
                    let format = v["format"]
                        .as_str()
                        .filter(|s| !s.is_empty())
                        .unwrap_or("YYYY-MM-DD");
                    let parsed =
                        natural::parse(if text.is_empty() { "today" } else { text }, now, tz)?;
                    Ok(match parsed {
                        None => serde_json::json!({"failure":"parse"}),
                        Some(_) if !date::valid_format(format) => {
                            serde_json::json!({"failure":"format"})
                        }
                        Some(ms) => serde_json::json!({"text":date::format(ms,tz,format,false)?}),
                    })
                }
                _ => {
                    let mut enabled = [true; 14];
                    if let Some(a) = v["enabled"].as_array() {
                        for (i, e) in enabled.iter_mut().enumerate() {
                            *e = a.get(i).and_then(|v| v.as_bool()).unwrap_or(false)
                        }
                    }
                    serde_json::to_value(date::convert(text, now, tz, &enabled)?)
                        .map_err(|e| e.to_string())
                }
            }
        })();
        println!(
            "{}",
            match result {
                Ok(v) => v,
                Err(e) => serde_json::json!({"error":e}),
            }
        );
    }
}
