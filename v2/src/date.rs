//! Native translation of dayjs 1.11.13's parse/format paths used by the two TSX commands.
//! Date's implementation-dependent string fallback is deliberately bounded; see LIMITS.md.
use chrono::{
    DateTime, Datelike, Duration, LocalResult, NaiveDate, NaiveDateTime, Offset, TimeZone, Timelike,
};
use chrono_tz::Tz;
use regex::Regex;
use std::sync::LazyLock;

pub const FORMAT_KEYS: [&str; 14] = [
    "isoDate",
    "isoDateTime",
    "formatWithMillis",
    "formatWithTimezone",
    "isoFormat",
    "utcIsoFormat",
    "unixTimestamp",
    "unixMillis",
    "localizedShortDate",
    "localizedShortDateTime",
    "localizedFullDateTime",
    "localizedLongDateTime",
    "localizedTime",
    "localizedSecondsTime",
];
pub const FORMATS: [&str; 14] = [
    "YYYY-MM-DD",
    "YYYY-MM-DD HH:mm:ss",
    "YYYY-MM-DD HH:mm:ss.SSS",
    "YYYY-MM-DD HH:mm:ssZ",
    "YYYY-MM-DDTHH:mm:ssZ",
    "YYYY-MM-DDTHH:mm:ss[Z]",
    "X",
    "x",
    "L",
    "L LT",
    "LLL",
    "LLLL",
    "LT",
    "LTS",
];
pub const MONTHS: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];
pub const DAYS: [&str; 7] = [
    "Sunday",
    "Monday",
    "Tuesday",
    "Wednesday",
    "Thursday",
    "Friday",
    "Saturday",
];
pub const MAX_TEXT: usize = 4096;
pub fn checked_text(s: &str) -> Result<(), String> {
    if s.len() > MAX_TEXT {
        Err("Input exceeds 4096 UTF-8 bytes".into())
    } else {
        Ok(())
    }
}
pub fn system_zone() -> Result<Tz, String> {
    if let Ok(s) = std::env::var("TZ") {
        return s
            .trim_start_matches(':')
            .parse()
            .map_err(|_| "Unsupported TZ; use an IANA timezone".into());
    }
    iana_time_zone::get_timezone()
        .map_err(|e| e.to_string())?
        .parse()
        .map_err(|_| "OS timezone is not in the pinned IANA database".into())
}
pub fn local(ms: i64, tz: Tz) -> Result<DateTime<Tz>, String> {
    DateTime::from_timestamp_millis(ms)
        .map(|d| d.with_timezone(&tz))
        .ok_or("Date outside native bounds".into())
}
pub fn bounded(d: NaiveDateTime) -> Result<NaiveDateTime, String> {
    if (-9999..=9999).contains(&d.year()) {
        Ok(d)
    } else {
        Err("Date outside supported civil years -9999..9999".into())
    }
}
/// JS compatible fold=earlier, gap=advance by the transition's displacement.
pub fn resolve(tz: Tz, n: NaiveDateTime, reject_gap: bool) -> Result<i64, String> {
    match tz.from_local_datetime(&n) {
        LocalResult::Single(d) => Ok(d.timestamp_millis()),
        LocalResult::Ambiguous(a, b) => Ok(a.timestamp_millis().min(b.timestamp_millis())),
        LocalResult::None if reject_gap => {
            Err("Invalid local date in a daylight-saving gap".into())
        }
        LocalResult::None => {
            // IANA transitions can skip an entire date (e.g. Pacific/Apia).
            let before =
                (1..=48).find_map(|h| tz.from_local_datetime(&(n - Duration::hours(h))).earliest());
            let after =
                (1..=48).find_map(|h| tz.from_local_datetime(&(n + Duration::hours(h))).earliest());
            match (before, after) {
                (Some(b), Some(a)) => {
                    let shift =
                        a.offset().fix().local_minus_utc() - b.offset().fix().local_minus_utc();
                    tz.from_local_datetime(&(n + Duration::seconds(i64::from(shift))))
                        .earliest()
                        .map(|d| d.timestamp_millis())
                        .ok_or("Unresolvable local date".into())
                }
                _ => Err("Unresolvable local date".into()),
            }
        }
    }
}
// Keep the positional JS Date constructor fields visible in the translation.
#[allow(clippy::too_many_arguments)]
pub fn normalized(
    y: i32,
    m: i32,
    d: i32,
    h: i64,
    min: i64,
    s: i64,
    ms: i64,
    js_year: bool,
) -> Result<NaiveDateTime, String> {
    let y = if js_year && (0..=99).contains(&y) {
        1900 + y
    } else {
        y
    };
    let y = y
        .checked_add((m - 1).div_euclid(12))
        .ok_or("Date overflow")?;
    let m = (m - 1).rem_euclid(12) + 1;
    let base = NaiveDate::from_ymd_opt(y, m as u32, 1)
        .and_then(|d| d.and_hms_opt(0, 0, 0))
        .ok_or("Date overflow")?;
    let delta = i64::from(d - 1)
        .checked_mul(86400000)
        .and_then(|v| v.checked_add(h.checked_mul(3600000)?))
        .and_then(|v| v.checked_add(min.checked_mul(60000)?))
        .and_then(|v| v.checked_add(s.checked_mul(1000)?))
        .and_then(|v| v.checked_add(ms))
        .ok_or("Date overflow")?;
    Ok(base
        .checked_add_signed(Duration::milliseconds(delta))
        .ok_or("Date overflow")?)
}
pub fn js_number(s: &str) -> Option<f64> {
    let t = s.trim_matches(|c: char| c.is_whitespace() || c == '\u{feff}');
    if t.is_empty() {
        return Some(0.0);
    }
    for (p, r) in [
        ("0x", 16),
        ("0X", 16),
        ("0o", 8),
        ("0O", 8),
        ("0b", 2),
        ("0B", 2),
    ] {
        if let Some(v) = t.strip_prefix(p) {
            return u64::from_str_radix(v, r).ok().map(|n| n as f64);
        }
    }
    if ["Infinity", "+Infinity"].contains(&t) {
        return Some(f64::INFINITY);
    }
    if t == "-Infinity" {
        return Some(f64::NEG_INFINITY);
    }
    static N: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"^[+-]?(?:\d+\.?\d*|\.\d+)(?:[eE][+-]?\d+)?$").unwrap());
    if N.is_match(t) {
        t.parse().ok()
    } else {
        None
    }
}
static DAYJS: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(\d{4})[-/]?(\d{1,2})?[-/]?(\d{0,2})[Tt\s]*(\d{1,2})?:?(\d{1,2})?:?(\d{1,2})?[.:]?(\d+)?$").unwrap()
});
fn num(c: &regex::Captures<'_>, i: usize, default: i64) -> i64 {
    c.get(i)
        .and_then(|m| m.as_str().parse().ok())
        .unwrap_or(default)
}
pub fn parse_dayjs(s: &str, tz: Tz) -> Result<Option<i64>, String> {
    checked_text(s)?;
    if !s.ends_with(['Z', 'z']) {
        if let Some(c) = DAYJS.captures(s) {
            let frac = c
                .get(7)
                .map(|m| &m.as_str()[..m.as_str().len().min(3)])
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);
            return Ok(normalized(
                num(&c, 1, 0) as i32,
                num(&c, 2, 1) as i32,
                num(&c, 3, 1) as i32,
                num(&c, 4, 0),
                num(&c, 5, 0),
                num(&c, 6, 0),
                frac,
                true,
            )
            .ok()
            .and_then(|n| resolve(tz, n, false).ok()));
        }
    }
    // ECMA's ISO path: date-only YYYY-MM-DD above is dayjs LOCAL; explicit Z/offset is UTC based.
    if regex::Regex::new(r":60(?:[.Zz+\-]|$)").unwrap().is_match(s) {
        return Ok(None);
    }
    if s.contains("T24:00:00") {
        let replaced = s.replace("T24:00:00", "T00:00:00");
        if let Ok(d) = DateTime::parse_from_rfc3339(&replaced) {
            return Ok(Some(d.timestamp_millis() + 86400000));
        }
    }
    static ISO: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"^(\d{4})-(\d{2})-(\d{2})[Tt](\d{2}):(\d{2})(?::(\d{2})(?:\.(\d+))?)?([Zz]|([+-])(\d{2}):?(\d{2}))$").unwrap()
    });
    if let Some(c) = ISO.captures(s) {
        let (y, m, d, h, minute, sec) = (
            num(&c, 1, 0),
            num(&c, 2, 0),
            num(&c, 3, 0),
            num(&c, 4, 0),
            num(&c, 5, 0),
            num(&c, 6, 0),
        );
        let frac = c
            .get(7)
            .map(|s| format!("{s:0<3}", s = s.as_str()))
            .unwrap_or_else(|| "000".into());
        let ms = frac[..3].parse::<i64>().unwrap_or(0);
        let oh = num(&c, 10, 0);
        let om = num(&c, 11, 0);
        if !(1..=12).contains(&m)
            || !(1..=31).contains(&d)
            || h > 24
            || minute > 59
            || sec > 59
            || oh > 23
            || om > 59
            || (h == 24 && (minute != 0 || sec != 0 || ms != 0))
        {
            return Ok(None);
        }
        let offset = (oh * 60 + om)
            * if c.get(9).is_some_and(|m| m.as_str() == "-") {
                -1
            } else {
                1
            };
        return Ok(
            normalized(y as i32, m as i32, d as i32, h, minute, sec, ms, false)
                .ok()
                .map(|n| n.and_utc().timestamp_millis() - offset * 60000),
        );
    }
    if let Ok(d) = DateTime::parse_from_rfc3339(s) {
        return Ok(Some(d.timestamp_millis()));
    }
    if let Ok(d) = DateTime::parse_from_rfc2822(s) {
        return Ok(Some(d.timestamp_millis()));
    }
    let trimmed = s.trim();
    // V8's well established month/day/year and English Date string families.
    let without_zone = trimmed.split(" (").next().unwrap_or(trimmed);
    for f in [
        "%a %b %d %Y %H:%M:%S GMT%z",
        "%a, %d %b %Y %H:%M:%S GMT",
        "%d %b %Y %H:%M:%S GMT",
    ] {
        if let Ok(d) = DateTime::parse_from_str(without_zone, f) {
            return Ok(Some(d.timestamp_millis()));
        }
    }
    for f in [
        "%b %d, %Y %I:%M %p",
        "%B %d, %Y %I:%M %p",
        "%B %d, %Y %H:%M:%S",
        "%B %d, %Y %H:%M",
        "%Y-%m-%d %H:%M:%S",
        "%m/%d/%Y %H:%M:%S",
        "%b %d %Y %H:%M:%S",
    ] {
        if let Ok(d) = NaiveDateTime::parse_from_str(trimmed, f) {
            return Ok(resolve(tz, d, false).ok());
        }
    }
    for f in ["%B %d, %Y", "%B %d %Y", "%b %d %Y", "%d %B %Y", "%d %b %Y"] {
        if let Ok(d) = NaiveDate::parse_from_str(trimmed, f) {
            return Ok(resolve(tz, d.and_hms_opt(0, 0, 0).unwrap(), false).ok());
        }
    }
    static SLASH: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"^(\d{1,2})[/.-](\d{1,2})[/.-](\d{2}|\d{4})$").unwrap());
    if let Some(c) = SLASH.captures(trimmed) {
        let mut y = num(&c, 3, 0) as i32;
        if y < 100 {
            y += if y < 50 { 2000 } else { 1900 }
        }
        let m = num(&c, 1, 0);
        let d = num(&c, 2, 0);
        if (1..=12).contains(&m) && (1..=31).contains(&d) {
            return Ok(normalized(y, m as i32, d as i32, 0, 0, 0, 0, false)
                .ok()
                .and_then(|n| resolve(tz, n, false).ok()));
        }
    }
    if let Some(local_text) = trimmed.strip_suffix(" GMT") {
        if let Some(ms) = parse_dayjs(local_text, chrono_tz::UTC)? {
            return Ok(Some(ms));
        }
    }
    static RFC: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"^(?:[A-Za-z]{3},? )?(\d{1,2}) ([A-Za-z]{3}) (\d{4}) (\d{2}):(\d{2}):(\d{2}) (?:GMT|UTC)$").unwrap()
    });
    if let Some(c) = RFC.captures(trimmed) {
        let month = MONTHS
            .iter()
            .position(|m| m[..3].eq_ignore_ascii_case(&c[2]))
            .map(|m| m as i32 + 1);
        if let Some(m) = month {
            if (1..=31).contains(&num(&c, 1, 0)) {
                return Ok(normalized(
                    num(&c, 3, 0) as i32,
                    m,
                    num(&c, 1, 0) as i32,
                    num(&c, 4, 0),
                    num(&c, 5, 0),
                    num(&c, 6, 0),
                    0,
                    false,
                )
                .ok()
                .map(|d| d.and_utc().timestamp_millis()));
            }
        }
    }
    // JS's numeric month/year Date fallback is relevant to the pre-validation gate.
    let trimmed = trimmed.trim_start_matches(['+', '-']);
    if trimmed.bytes().all(|b| b.is_ascii_digit()) && !trimmed.is_empty() {
        if let Ok(n) = trimmed.parse::<i32>() {
            let y = if (32..100).contains(&n) {
                if n < 50 {
                    2000 + n
                } else {
                    1900 + n
                }
            } else if (13..32).contains(&n) {
                return Ok(None);
            } else if n <= 12 {
                2001
            } else {
                n
            };
            let m = if (1..=12).contains(&n) { n } else { 1 };
            let y = if n == 0 { 2000 } else { y };
            return Ok(normalized(y, m, 1, 0, 0, 0, 0, false)
                .ok()
                .and_then(|n| resolve(tz, n, false).ok()));
        }
    }
    Ok(None)
}
#[derive(Debug, Clone, serde::Serialize)]
pub struct Conversion {
    pub valid: bool,
    pub values: Vec<String>,
}
pub fn convert(input: &str, now: i64, tz: Tz, enabled: &[bool; 14]) -> Result<Conversion, String> {
    checked_text(input)?;
    let date = if input.is_empty() || input == "now" {
        Some(now.div_euclid(1000) * 1000)
    } else {
        parse_dayjs(input, tz)?
    };
    if date.is_none() {
        return Ok(Conversion {
            valid: false,
            values: vec![],
        });
    }
    let date = if !input.is_empty() && input != "now" {
        if let Some(n) = js_number(input) {
            let n = match input.encode_utf16().count() {
                10 => n * 1000.0,
                13 => n,
                _ => {
                    return Ok(Conversion {
                        valid: true,
                        values: vec![],
                    })
                }
            };
            if !n.is_finite() || n.abs() > 8.64e15 {
                None
            } else {
                Some(n.trunc() as i64)
            }
        } else {
            date
        }
    } else {
        date
    };
    let values = FORMATS
        .iter()
        .enumerate()
        .filter(|(i, _)| enabled[*i])
        .map(|(i, f)| match date {
            Some(ms) => format(ms, if i == 5 { chrono_tz::UTC } else { tz }, f, true),
            None => Ok("Invalid Date".into()),
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Conversion {
        valid: true,
        values,
    })
}
pub fn valid_format(s: &str) -> bool {
    let mut s = s
        .chars()
        .filter(|c| !"-/.,: ".contains(*c))
        .collect::<String>();
    for t in [
        "YYYY", "YY", "MMMM", "MMM", "MM", "M", "DD", "D", "dddd", "ddd", "dd", "d", "HH", "H",
        "hh", "h", "mm", "m", "ss", "s", "SSS", "ZZ", "Z", "A", "a",
    ] {
        s = s.replace(t, "")
    }
    s.is_empty()
}
pub fn format(ms: i64, tz: Tz, fmt: &str, advanced: bool) -> Result<String, String> {
    checked_text(fmt)?;
    let d = local(ms, tz)?;
    bounded(d.naive_local())?;
    let y = d.year();
    let mo = d.month() as usize;
    let day = d.day();
    let wd = d.weekday().num_days_from_sunday() as usize;
    let h = d.hour();
    let h12 = if h % 12 == 0 { 12 } else { h % 12 };
    let off = d.offset().fix().local_minus_utc() / 60;
    let off = (-((-f64::from(off) / 15.0 + 0.5).floor()) * 15.0) as i32;
    let sign = if off < 0 { '-' } else { '+' };
    let off = off.abs();
    let z = format!("{sign}{:02}:{:02}", off / 60, off % 60);
    let fmt = if fmt.is_empty() {
        "YYYY-MM-DDTHH:mm:ssZ"
    } else {
        fmt
    };
    let mut expanded = String::new();
    let mut i = 0;
    // localizedFormat is loaded only in index.tsx; English is dayjs's actual default,
    // independent of OS locale. Natural command has no plugins.
    while i < fmt.len() {
        let s = &fmt[i..];
        if s.starts_with('[') {
            if let Some(j) = s.find(']') {
                expanded.push_str(&s[..=j]);
                i += j + 1;
                continue;
            }
        }
        if advanced {
            if let Some((t, r)) = [
                ("LLLL", "dddd, MMMM D, YYYY h:mm A"),
                ("LLL", "MMMM D, YYYY h:mm A"),
                ("LTS", "h:mm:ss A"),
                ("LL", "MMMM D, YYYY"),
                ("LT", "h:mm A"),
                ("L", "MM/DD/YYYY"),
            ]
            .into_iter()
            .find(|(t, _)| s.starts_with(t))
            {
                expanded.push_str(r);
                i += t.len();
                continue;
            }
        }
        let c = s.chars().next().unwrap();
        expanded.push(c);
        i += c.len_utf8();
    }
    let mut out = String::new();
    let mut i = 0;
    while i < expanded.len() {
        let s = &expanded[i..];
        if s.starts_with('[') {
            if let Some(j) = s.find(']') {
                out.push_str(&s[1..j]);
                i += j + 1;
                continue;
            }
        }
        if advanced {
            if let Some(t) = [
                "gggg", "GGGG", "zzz", "wo", "ww", "WW", "Do", "kk", "Q", "X", "x", "k", "w", "W",
                "z",
            ]
            .into_iter()
            .find(|t| s.starts_with(t))
            {
                let r = match t {
                    "Q" => mo.div_ceil(3).to_string(),
                    "X" => ms.div_euclid(1000).to_string(),
                    "x" => ms.to_string(),
                    "k" => if h == 0 { 24 } else { h }.to_string(),
                    "kk" => format!("{:02}", if h == 0 { 24 } else { h }),
                    "Do" => format!(
                        "{day}{}",
                        if (11..=13).contains(&(day % 100)) {
                            "th"
                        } else {
                            match day % 10 {
                                1 => "st",
                                2 => "nd",
                                3 => "rd",
                                _ => "th",
                            }
                        }
                    ),
                    _ => {
                        return Err(format!(
                            "Original dayjs throws for missing plugin token {t}"
                        ))
                    }
                };
                out.push_str(&r);
                i += t.len();
                continue;
            }
        }
        if let Some(t) = [
            "YYYY", "MMMM", "dddd", "MMM", "ddd", "SSS", "YY", "MM", "DD", "dd", "HH", "hh", "mm",
            "ss", "ZZ", "M", "D", "d", "H", "h", "m", "s", "Z", "A", "a",
        ]
        .into_iter()
        .find(|t| s.starts_with(t))
        {
            let r = match t {
                "YYYY" => {
                    let s = y.to_string();
                    format!("{}{}", "0".repeat(4usize.saturating_sub(s.len())), s)
                }
                "YY" => format!("{:02}", y % 100),
                "MMMM" => MONTHS[mo - 1].into(),
                "MMM" => MONTHS[mo - 1][..3].into(),
                "MM" => format!("{mo:02}"),
                "M" => mo.to_string(),
                "DD" => format!("{day:02}"),
                "D" => day.to_string(),
                "dddd" => DAYS[wd].into(),
                "ddd" => DAYS[wd][..3].into(),
                "dd" => DAYS[wd][..2].into(),
                "d" => wd.to_string(),
                "HH" => format!("{h:02}"),
                "H" => h.to_string(),
                "hh" => format!("{h12:02}"),
                "h" => h12.to_string(),
                "mm" => format!("{:02}", d.minute()),
                "m" => d.minute().to_string(),
                "ss" => format!("{:02}", d.second()),
                "s" => d.second().to_string(),
                "SSS" => format!("{:03}", d.timestamp_subsec_millis()),
                "Z" => z.clone(),
                "ZZ" => z.replace(':', ""),
                "A" => if h < 12 { "AM" } else { "PM" }.into(),
                "a" => if h < 12 { "am" } else { "pm" }.into(),
                _ => unreachable!(),
            };
            out.push_str(&r);
            i += t.len();
        } else {
            let c = s.chars().next().unwrap();
            out.push(c);
            i += c.len_utf8()
        }
    }
    Ok(out)
}
