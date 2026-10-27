//! Native translation of the locked chrono-node English casual grammar.
use crate::date::{self, local, normalized, resolve};
use chrono::{Datelike, Duration, NaiveDate, NaiveDateTime, Offset, Timelike};
use chrono_tz::Tz;
use fancy_regex::Regex;
use serde::Deserialize;
use serde_json::Value;
use std::{collections::BTreeMap, sync::LazyLock};
const Y: usize = 0;
const M: usize = 1;
const D: usize = 2;
const H: usize = 3;
const MIN: usize = 4;
const S: usize = 5;
const MS: usize = 6;
const WD: usize = 7;
const AP: usize = 8;
const TZ: usize = 9;
#[derive(Deserialize)]
struct Grammar {
    parsers: Vec<ParserData>,
    constants: Value,
    timezones: Value,
}
#[derive(Deserialize)]
struct ParserData {
    name: String,
    pattern: String,
    boundary: bool,
}
struct Parser {
    name: String,
    re: Regex,
    boundary: bool,
}
static DATA: LazyLock<Grammar> =
    LazyLock::new(|| serde_json::from_str(include_str!("../resources/chrono-en.json")).unwrap());
static PARSERS: LazyLock<Vec<Parser>> = LazyLock::new(|| {
    DATA.parsers
        .iter()
        .map(|p| Parser {
            name: p.name.clone(),
            re: rx(&if p.boundary {
                format!(r"(\W|^){}", p.pattern)
            } else {
                p.pattern.clone()
            }),
            boundary: p.boundary,
        })
        .collect()
});
fn rx(s: &str) -> Regex {
    let mut s = s
        .replace(r"\w", "[A-Za-z0-9_]")
        .replace(r"\W", "[^A-Za-z0-9_]")
        .replace(r"\d", "[0-9]");
    for c in ['-', ':', '/', '–', '〜', '，', '（', '）', '~', '：'] {
        s = s.replace(&format!("\\{c}"), &c.to_string())
    }
    Regex::new(&format!("(?i){s}")).unwrap()
}
#[derive(Clone, Debug)]
struct C {
    v: [Option<i32>; 10],
    known: u16,
}
impl C {
    fn new(n: NaiveDateTime) -> Self {
        Self {
            v: [
                Some(n.year()),
                Some(n.month() as i32),
                Some(n.day() as i32),
                Some(12),
                Some(0),
                Some(0),
                Some(0),
                None,
                None,
                None,
            ],
            known: 0,
        }
    }
    fn get(&self, k: usize) -> i32 {
        self.v[k].unwrap_or(0)
    }
    fn certain(&self, k: usize) -> bool {
        self.known & (1 << k) != 0
    }
    fn assign(&mut self, k: usize, v: i32) {
        self.v[k] = Some(v);
        self.known |= 1 << k
    }
    fn imply(&mut self, k: usize, v: i32) {
        if !self.certain(k) {
            self.v[k] = Some(v)
        }
    }
    fn date(&mut self, n: NaiveDateTime, known: bool) {
        for (k, v) in [(Y, n.year()), (M, n.month() as i32), (D, n.day() as i32)] {
            if known {
                self.assign(k, v)
            } else {
                self.imply(k, v)
            }
        }
    }
    fn time(&mut self, n: NaiveDateTime, known: bool) {
        for (k, v) in [
            (H, n.hour() as i32),
            (MIN, n.minute() as i32),
            (S, n.second() as i32),
            (MS, (n.nanosecond() / 1000000) as i32),
        ] {
            if known {
                self.assign(k, v)
            } else {
                self.imply(k, v)
            }
        }
    }
    fn naive(&self) -> Option<NaiveDateTime> {
        normalized(
            self.get(Y),
            self.get(M),
            self.get(D),
            self.get(H).into(),
            self.get(MIN).into(),
            self.get(S).into(),
            self.get(MS).into(),
            false,
        )
        .ok()
    }
    fn instant(&self, tz: Tz, now: i64, validate: bool) -> Option<i64> {
        let n = self.naive()?;
        let ms = resolve(tz, n, validate).ok()?;
        if validate {
            let actual = local(ms, tz).ok()?.naive_local();
            for (k, v) in [
                (Y, actual.year()),
                (M, actual.month() as i32),
                (D, actual.day() as i32),
                (H, actual.hour() as i32),
                (MIN, actual.minute() as i32),
            ] {
                if v != self.get(k) {
                    return None;
                }
            }
        }
        let offset_date = if ms < 0 { now } else { ms };
        let system = local(offset_date, tz)
            .ok()?
            .offset()
            .fix()
            .local_minus_utc()
            / 60;
        Some(ms + i64::from(system - self.v[TZ].unwrap_or(system)) * 60000)
    }
    fn only_date(&self) -> bool {
        ![H, MIN, S].iter().any(|k| self.certain(*k))
    }
    fn only_time(&self) -> bool {
        ![Y, M, D, WD].iter().any(|k| self.certain(*k))
    }
    fn only_weekday(&self) -> bool {
        self.certain(WD) && !self.certain(D) && !self.certain(M)
    }
}
#[derive(Clone)]
struct R {
    start: usize,
    end: usize,
    c: C,
    to: Option<C>,
}
fn n(v: &[String], i: usize) -> i32 {
    v.get(i).and_then(|s| s.parse().ok()).unwrap_or(0)
}
fn dict(which: &str, key: &str) -> Option<i32> {
    DATA.constants[which][key.to_lowercase()]
        .as_i64()
        .map(|n| n as i32)
}
fn ordinal(s: &str) -> i32 {
    dict("ORDINAL_WORD_DICTIONARY", s).unwrap_or_else(|| {
        s.trim_end_matches(|c: char| !c.is_ascii_digit())
            .parse()
            .unwrap_or(0)
    })
}
fn year(s: &str) -> i32 {
    let s = s.to_uppercase();
    let raw = s
        .trim_end_matches(|c: char| c.is_ascii_alphabetic() || c.is_whitespace())
        .parse()
        .unwrap_or(0);
    if s.contains("BE") {
        raw - 543
    } else if s.contains("BC") {
        -raw
    } else if s.contains("AD") || s.contains("CE") {
        raw
    } else if raw < 100 {
        if raw > 50 {
            1900 + raw
        } else {
            2000 + raw
        }
    } else {
        raw
    }
}
fn closest_year(now: NaiveDateTime, day: i32, month: i32) -> i32 {
    let mut best = now.year();
    let mut distance = i64::MAX;
    for y in [now.year(), now.year() + 1, now.year() - 1] {
        if let Ok(t) = normalized(
            y,
            month,
            day,
            i64::from(now.hour()),
            i64::from(now.minute()),
            i64::from(now.second()),
            i64::from(now.nanosecond() / 1000000),
            false,
        ) {
            let d = (t - now).num_milliseconds().abs();
            if d < distance {
                distance = d;
                best = y
            }
        }
    }
    best
}
fn number(s: &str) -> f64 {
    let s = s.to_lowercase();
    dict("INTEGER_WORD_DICTIONARY", &s)
        .map(f64::from)
        .unwrap_or_else(|| {
            if ["a", "an", "the"].contains(&s.as_str()) {
                1.0
            } else if s.contains("few") {
                3.0
            } else if s.contains("half") {
                0.5
            } else if s.contains("couple") {
                2.0
            } else if s.contains("several") {
                7.0
            } else {
                s.parse().unwrap_or(0.0)
            }
        })
}
fn durations(s: &str, negative: bool) -> Result<BTreeMap<String, f64>, String> {
    static RE: LazyLock<Regex> = LazyLock::new(|| {
        let mut units = DATA.constants["TIME_UNIT_DICTIONARY"]
            .as_object()
            .unwrap()
            .keys()
            .map(|s| regex::escape(s))
            .collect::<Vec<_>>();
        units.sort_by_key(|s| std::cmp::Reverse(s.len()));
        let units = units.join("|");
        rx(&format!(
            "({})\\s{{0,3}}({units})",
            DATA.constants["NUMBER_PATTERN"].as_str().unwrap()
        ))
    });
    let mut result = BTreeMap::new();
    for c in RE.captures_iter(s) {
        let c = c.map_err(|_| "Natural language regex resource limit")?;
        if c.get(0)
            .unwrap()
            .as_str()
            .chars()
            .all(|c| c.is_ascii_alphabetic())
        {
            continue;
        }
        let unit = DATA.constants["TIME_UNIT_DICTIONARY"][c[2].to_lowercase()]
            .as_str()
            .unwrap();
        let unit = match unit {
            "d" => "day",
            "w" => "week",
            s => s,
        };
        let value = number(&c[1]) * if negative { -1.0 } else { 1.0 };
        if value.abs() > 10000.0 {
            return Err("Relative amount exceeds 10000 units".into());
        }
        result.insert(unit.into(), value);
    }
    Ok(result)
}
fn add_duration(reference: i64, tz: Tz, mut ds: BTreeMap<String, f64>) -> Option<i64> {
    let mut instant = reference;
    for (unit, next, mult) in [
        ("year", "month", 12.0),
        ("quarter", "", 0.0),
        ("month", "week", 4.0),
        ("week", "day", 7.0),
        ("day", "hour", 24.0),
        ("hour", "minute", 60.0),
        ("minute", "second", 60.0),
        ("second", "millisecond", 1000.0),
        ("millisecond", "", 0.0),
    ] {
        if let Some(&value) = ds.get(unit) {
            let floor = value.floor() as i32;
            let f = value - value.floor();
            let t = local(instant, tz).ok()?.naive_local();
            let t = match unit {
                "year" => normalized(
                    t.year() + floor,
                    t.month() as i32,
                    t.day() as i32,
                    i64::from(t.hour()),
                    i64::from(t.minute()),
                    i64::from(t.second()),
                    i64::from(t.nanosecond() / 1000000),
                    false,
                )
                .ok()?,
                "quarter" | "month" => normalized(
                    t.year(),
                    t.month() as i32 + floor * if unit == "quarter" { 3 } else { 1 },
                    t.day() as i32,
                    i64::from(t.hour()),
                    i64::from(t.minute()),
                    i64::from(t.second()),
                    i64::from(t.nanosecond() / 1000000),
                    false,
                )
                .ok()?,
                _ => t.checked_add_signed(Duration::milliseconds(
                    i64::from(floor)
                        * match unit {
                            "week" => 604800000,
                            "day" => 86400000,
                            "hour" => 3600000,
                            "minute" => 60000,
                            "second" => 1000,
                            _ => 1,
                        },
                ))?,
            };
            instant = resolve(tz, t, false).ok()?;
            if f > 0.0 && !next.is_empty() {
                let f = f * mult;
                let f = if ["year", "month"].contains(&unit) {
                    f
                } else {
                    (f + 0.5).floor()
                };
                *ds.entry(next.into()).or_default() += f;
            }
        }
    }
    Some(instant)
}
fn relative(reference: i64, tz: Tz, ds: BTreeMap<String, f64>) -> Option<C> {
    let instant = add_duration(reference, tz, ds.clone())?;
    let n = local(instant, tz).ok()?.naive_local();
    let mut c = C::new(n);
    let timed = ["hour", "minute", "second"]
        .iter()
        .any(|k| ds.get(*k).is_some_and(|v| *v != 0.0));
    c.time(n, timed);
    c.date(n, timed);
    let off = local(reference, tz).ok()?.offset().fix().local_minus_utc() / 60;
    if timed {
        c.assign(TZ, off)
    } else {
        c.imply(TZ, off);
        if ds.get("day").is_some_and(|v| *v != 0.0) {
            c.date(n, true);
            c.assign(WD, n.weekday().num_days_from_sunday() as i32)
        } else if ds.get("week").is_some_and(|v| *v != 0.0) {
            c.date(n, true);
            c.imply(WD, n.weekday().num_days_from_sunday() as i32)
        } else if ds.get("month").is_some_and(|v| *v != 0.0) {
            c.assign(M, n.month() as i32);
            c.assign(Y, n.year())
        } else if ds.get("year").is_some_and(|v| *v != 0.0) {
            c.assign(Y, n.year())
        }
    }
    Some(c)
}
fn extract(
    name: &str,
    v: &[String],
    matched: &str,
    now: i64,
    tz: Tz,
) -> Result<Option<(C, Option<C>)>, String> {
    let reference = local(now, tz)?.naive_local();
    let mut c = C::new(reference);
    let mut to = None;
    match name {
        "ENYearMonthDayParser" => {
            let mut month = if v[2].is_empty() {
                n(v, 3)
            } else {
                dict("MONTH_DICTIONARY", &v[2]).unwrap_or(0)
            };
            let mut day = n(v, 4);
            if !(1..=12).contains(&month) && (1..=12).contains(&day) {
                std::mem::swap(&mut month, &mut day)
            }
            if !(1..=31).contains(&day) {
                return Ok(None);
            }
            c.assign(Y, n(v, 1));
            c.assign(M, month);
            c.assign(D, day)
        }
        "ISOFormatParser" => {
            c.assign(Y, n(v, 1));
            c.assign(M, n(v, 2));
            c.assign(D, n(v, 3));
            if !v[4].is_empty() {
                c.assign(H, n(v, 4));
                c.assign(MIN, n(v, 5));
                if !v[6].is_empty() {
                    c.assign(S, n(v, 6))
                }
                if !v[7].is_empty() {
                    c.assign(MS, n(v, 7))
                }
                if !v[8].is_empty() {
                    let h = n(v, 9);
                    c.assign(TZ, h * 60 + if h < 0 { -n(v, 10) } else { n(v, 10) })
                }
            }
        }
        "SlashDateFormatParser" => {
            let text = matched.trim_matches(|c: char| !c.is_ascii_digit());
            if !text.contains('/') && v[4].is_empty() {
                return Ok(None);
            }
            if text.contains('.')
                && text.len() <= 8
                && text.split('.').next().unwrap_or("").len() == 1
            {
                return Ok(None);
            }
            let (mut mo, mut d) = (n(v, 2), n(v, 3));
            if mo > 12 && mo <= 31 && (1..=12).contains(&d) {
                std::mem::swap(&mut mo, &mut d)
            }
            if !(1..=31).contains(&d) {
                return Ok(None);
            }
            c.assign(M, mo);
            c.assign(D, d);
            if !v[4].is_empty() {
                c.assign(Y, year(&v[4]))
            } else {
                c.imply(Y, closest_year(reference, d, mo))
            }
        }
        "ENMonthNameLittleEndianParser" | "ENMonthNameMiddleEndianParser" => {
            let little = name == "ENMonthNameLittleEndianParser";
            let mo = dict("MONTH_DICTIONARY", &v[if little { 3 } else { 1 }]).unwrap_or(0);
            let d = ordinal(&v[if little { 1 } else { 2 }]);
            if d > 31 {
                return Ok(None);
            }
            c.assign(M, mo);
            c.assign(D, d);
            if !v[4].is_empty() {
                c.assign(Y, year(&v[4]))
            } else {
                c.imply(Y, closest_year(reference, d, mo))
            }
            let end = &v[if little { 2 } else { 3 }];
            if !end.is_empty() {
                let mut e = c.clone();
                e.assign(D, ordinal(end));
                to = Some(e)
            }
        }
        "ENSlashMonthFormatParser" => {
            c.assign(Y, n(v, 2));
            c.assign(M, n(v, 1));
            c.imply(D, 1)
        }
        "ENMonthNameParser" => {
            let word = &v[2];
            if matched.len() <= 3 && dict("FULL_MONTH_NAME_DICTIONARY", word).is_none() {
                return Ok(None);
            }
            let mo = dict("MONTH_DICTIONARY", word).unwrap_or(0);
            c.assign(M, mo);
            c.imply(D, 1);
            if !v[3].is_empty() {
                c.assign(Y, year(&v[3]))
            } else {
                c.imply(Y, closest_year(reference, 1, mo))
            }
        }
        "ENCasualDateParser" => {
            let text = matched.to_lowercase();
            let mut d = reference;
            match text.as_str() {
                "now" => {
                    c.date(d, true);
                    c.time(d, true);
                    c.assign(TZ, local(now, tz)?.offset().fix().local_minus_utc() / 60)
                }
                "today" => {
                    c.date(d, true);
                    c.time(d, false)
                }
                "yesterday" | "tomorrow" | "tmr" | "tmrw" | "overmorrow" => {
                    d += Duration::days(match text.as_str() {
                        "yesterday" => -1,
                        "overmorrow" => 2,
                        _ => 1,
                    });
                    c.date(d, true);
                    c.time(d, false)
                }
                "tonight" => {
                    c.date(d, true);
                    c.imply(H, 22);
                    c.imply(AP, 1)
                }
                _ => {
                    if d.hour() > 6 {
                        d -= Duration::days(1)
                    }
                    c.date(d, true);
                    c.imply(H, 0)
                }
            }
        }
        "ENCasualTimeParser" => match v[1].to_lowercase().as_str() {
            "morning" => {
                c.imply(H, 6);
                c.imply(AP, 0)
            }
            "afternoon" => {
                c.imply(H, 15);
                c.imply(AP, 1)
            }
            "evening" | "night" => {
                c.imply(H, 20);
                c.imply(AP, 1)
            }
            "midnight" => {
                if reference.hour() > 2 {
                    c.date(reference + Duration::days(1), false)
                }
                c.assign(H, 0)
            }
            "noon" | "midday" => {
                c.assign(H, 12);
                c.imply(AP, 0)
            }
            _ => return Ok(None),
        },
        "ENWeekdayParser" => {
            let modifier = if !v[1].is_empty() {
                v[1].to_lowercase()
            } else {
                v[3].to_lowercase()
            };
            let w = reference.weekday().num_days_from_sunday() as i32;
            let wd = match v[2].to_lowercase().as_str() {
                "weekend" => {
                    if modifier == "last" {
                        0
                    } else {
                        6
                    }
                }
                "weekday" => {
                    if w == 0 || w == 6 {
                        if modifier == "last" {
                            5
                        } else {
                            1
                        }
                    } else {
                        let d = w - 1;
                        ((if modifier == "last" { d - 1 } else { d + 1 }) % 5) + 1
                    }
                }
                s => dict("WEEKDAY_DICTIONARY", s).unwrap_or(0),
            };
            let forward = (wd - w).rem_euclid(7);
            let backward = if wd - w >= 0 { wd - w - 7 } else { wd - w };
            let diff = match modifier.as_str() {
                "this" => forward,
                "last" | "past" => backward,
                "next" => {
                    if w == 0 {
                        if wd == 0 {
                            7
                        } else {
                            wd
                        }
                    } else if w == 6 {
                        if wd == 6 {
                            7
                        } else if wd == 0 {
                            8
                        } else {
                            1 + wd
                        }
                    } else if wd < w && wd != 0 {
                        forward
                    } else {
                        forward + 7
                    }
                }
                _ => {
                    if forward < -backward {
                        forward
                    } else {
                        backward
                    }
                }
            };
            c.date(reference + Duration::days(diff.into()), false);
            c.assign(WD, wd)
        }
        "ENRelativeDateFormatParser" => {
            let modifier = v[1].to_lowercase();
            let unit = DATA.constants["TIME_UNIT_DICTIONARY"][v[2].to_lowercase()]
                .as_str()
                .unwrap_or("day");
            let unit = match unit {
                "d" => "day",
                "w" => "week",
                s => s,
            };
            if modifier != "this" {
                let sign = if ["last", "past"].contains(&modifier.as_str()) {
                    -1.0
                } else {
                    1.0
                };
                return Ok(
                    relative(now, tz, BTreeMap::from([(unit.into(), sign)])).map(|c| (c, None))
                );
            }
            let word = v[2].to_lowercase();
            if word.contains("week") {
                c.date(
                    reference - Duration::days(reference.weekday().num_days_from_sunday().into()),
                    false,
                )
            } else if word.contains("month") {
                c.imply(D, 1);
                c.assign(Y, reference.year());
                c.assign(M, reference.month() as i32)
            } else if word.contains("year") {
                c.imply(D, 1);
                c.imply(M, 1);
                c.assign(Y, reference.year())
            }
        }
        "ENTimeUnitWithinFormatParser"
        | "ENTimeUnitAgoFormatParser"
        | "ENTimeUnitLaterFormatParser"
        | "ENTimeUnitCasualRelativeFormatParser" => {
            if matched.starts_with("for the ") {
                return Ok(None);
            }
            let casual = name == "ENTimeUnitCasualRelativeFormatParser";
            let neg = name == "ENTimeUnitAgoFormatParser"
                || (casual && ["last", "past", "-"].contains(&v[1].to_lowercase().as_str()));
            let ds = durations(&v[if casual { 2 } else { 1 }], neg)?;
            if ds.is_empty() {
                return Ok(None);
            }
            return Ok(relative(now, tz, ds).map(|c| (c, None)));
        }
        "ENTimeExpressionParser" => {
            let mut h = n(v, 2);
            let mut minute = 0;
            if h > 100 {
                if !v[3].is_empty() {
                    return Ok(None);
                }
                minute = h % 100;
                h /= 100
            }
            if h > 24 {
                return Ok(None);
            }
            if !v[3].is_empty() {
                if v[3].len() == 1 && v[6].is_empty() {
                    return Ok(None);
                }
                minute = n(v, 3)
            }
            if minute >= 60 {
                return Ok(None);
            }
            if h > 12 {
                c.assign(AP, 1)
            }
            if !v[6].is_empty() {
                if h > 12 {
                    return Ok(None);
                }
                if v[6].to_lowercase().starts_with('a') {
                    c.assign(AP, 0);
                    if h == 12 {
                        h = 0
                    }
                } else {
                    c.assign(AP, 1);
                    if h != 12 {
                        h += 12
                    }
                }
            } else {
                c.imply(AP, if h < 12 { 0 } else { 1 })
            }
            if !v[4].is_empty() {
                if n(v, 4) >= 60 {
                    return Ok(None);
                }
                c.assign(S, n(v, 4))
            }
            if !v[5].is_empty() {
                c.assign(MS, v[5][..v[5].len().min(3)].parse().unwrap_or(0))
            }
            if matched.ends_with("night") {
                if (6..12).contains(&h) {
                    h += 12;
                    c.assign(AP, 1)
                } else if h < 6 {
                    c.assign(AP, 0)
                }
            }
            if matched.ends_with("afternoon") {
                c.assign(AP, 1);
                if (0..=6).contains(&h) {
                    h += 12
                }
            }
            if matched.ends_with("morning") {
                c.assign(AP, 0)
            }
            c.assign(H, h);
            c.assign(MIN, minute);
            if rx(r"\d[ap]$").is_match(matched).unwrap_or(false)
                || rx(r"[^\d:.](\d[\d.]+)$")
                    .captures(matched)
                    .ok()
                    .flatten()
                    .is_some_and(|v| v[1].parse::<f64>().is_ok_and(|v| v > 24.0))
            {
                return Ok(None);
            }
        }
        _ => return Err(format!("Unported locked parser {name}")),
    }
    Ok(Some((c, to)))
}
fn overlap(rs: Vec<R>) -> Vec<R> {
    let mut out: Vec<R> = vec![];
    for r in rs {
        if let Some(prev) = out.last_mut() {
            if r.start < prev.end {
                if r.end - r.start > prev.end - prev.start {
                    *prev = r
                }
                continue;
            }
        }
        out.push(r)
    }
    out
}
fn merge_time(date: &C, time: &C) -> C {
    let mut c = date.clone();
    for k in [H, MIN, S, MS] {
        if time.certain(k) || (k == MIN && time.certain(H)) {
            c.assign(k, time.get(k))
        } else {
            c.imply(k, time.get(k))
        }
    }
    if time.certain(TZ) {
        c.assign(TZ, time.get(TZ))
    }
    if time.certain(AP) {
        c.assign(AP, time.get(AP))
    } else if c.v[AP].is_none() {
        c.v[AP] = time.v[AP]
    }
    if c.get(AP) == 1 && c.get(H) < 12 {
        if time.certain(H) {
            c.assign(H, c.get(H) + 12)
        } else {
            c.imply(H, c.get(H) + 12)
        }
    }
    c
}
fn timezone_abbr(word: &str, c: &C) -> Option<i32> {
    let t = &DATA.timezones[word.to_uppercase()];
    if let Some(n) = t.as_i64() {
        return Some(n as i32);
    }
    if !t.is_object() {
        return None;
    }
    let y = c.get(Y);
    let n = c.naive()?;
    let nth = |month: u32, k: u32, h: u32| -> Option<NaiveDateTime> {
        let first = NaiveDate::from_ymd_opt(y, month, 1)?;
        let day = 1 + (7 - first.weekday().num_days_from_sunday()) % 7 + (k - 1) * 7;
        NaiveDate::from_ymd_opt(y, month, day)?.and_hms_opt(h, 0, 0)
    };
    let last = |month: u32, h: u32| -> Option<NaiveDateTime> {
        let next = NaiveDate::from_ymd_opt(y, month + 1, 1)?;
        let d = next
            - Duration::days(if next.weekday().num_days_from_sunday() == 0 {
                7
            } else {
                next.weekday().num_days_from_sunday().into()
            });
        d.and_hms_opt(h, 0, 0)
    };
    let (start, end) = if word.to_uppercase() == "CET" {
        (last(3, 2)?, last(10, 3)?)
    } else {
        (nth(3, 2, 2)?, nth(11, 1, 2)?)
    };
    t[if n > start && n <= end {
        "timezoneOffsetDuringDst"
    } else {
        "timezoneOffsetNonDst"
    }]
    .as_i64()
    .map(|v| v as i32)
}
/// Source supplies no forwardDate option. Keep nearest weekday/year semantics.
pub fn parse(text: &str, now: i64, tz: Tz) -> Result<Option<i64>, String> {
    date::checked_text(text)?;
    let mut rs = vec![];
    for p in PARSERS.iter() {
        let mut pos = 0;
        let mut count = 0;
        while pos < text.len() {
            count += 1;
            if count > 128 {
                return Err("Natural language match limit (128/parser)".into());
            }
            let caps =
                p.re.captures(&text[pos..])
                    .map_err(|_| "Natural language regex resource limit")?;
            let Some(caps) = caps else { break };
            let full = caps.get(0).unwrap();
            let mut start = pos + full.start();
            let end = pos + full.end();
            let mut v = (0..caps.len())
                .map(|i| {
                    caps.get(i)
                        .map(|m| m.as_str().to_owned())
                        .unwrap_or_default()
                })
                .collect::<Vec<_>>();
            if p.boundary {
                let header = v[1].len();
                start += header;
                v.remove(1);
                v[0] = v[0][header..].into()
            }
            if p.name == "ENTimeExpressionParser" {
                start += v[1].len();
                v[0] = v[0][v[1].len()..].into()
            }
            let mut actual_end = end;
            if let Some((mut c, mut to)) = extract(&p.name, &v, &v[0], now, tz)? {
                if p.name == "SlashDateFormatParser" {
                    start += v[1].len();
                    actual_end -= v[5].len();
                    if start > 0 && text[..start].ends_with(|c: char| c.is_ascii_digit()) {
                        pos = end.max(pos + 1);
                        continue;
                    }
                }
                if p.name == "ENTimeExpressionParser" {
                    static FOLLOW: LazyLock<Regex> = LazyLock::new(|| {
                        rx(
                            r"^(\s*(?:-|–|~|〜|to|until|through|till|\?)\s*)(\d{1,4})(?:[.:：](\d{1,2})(?:[.:：](\d{1,2})(?:\.(\d{1,6}))?)?)?(?:\s*(a\.m\.|p\.m\.|am?|pm?))?(?!/)(?=\W|$)",
                        )
                    });
                    let following = FOLLOW
                        .captures(&text[end..])
                        .map_err(|_| "Regex resource limit")?;
                    if let Some(f) = following {
                        let mut e = C::new(local(now, tz)?.naive_local());
                        let raw_h = f[2].parse::<i32>().unwrap_or(0);
                        let mut h = raw_h;
                        let mut minute = f
                            .get(3)
                            .and_then(|m| m.as_str().parse::<i32>().ok())
                            .unwrap_or(0);
                        if f.get(3).is_none() && h > 100 {
                            minute = h % 100;
                            h /= 100;
                        }
                        let sec = f.get(4).and_then(|m| m.as_str().parse::<i32>().ok());
                        let millis = f.get(5).map(|m| {
                            let m = m.as_str();
                            m[..m.len().min(3)].parse::<i32>().unwrap_or(0)
                        });
                        let ap = f.get(6);
                        let valid = minute < 60
                            && h <= 24
                            && sec.is_none_or(|s| s < 60)
                            && !(ap.is_some() && h > 12);
                        let skipped = rx(r"^\s*[+-]\s*\d{3,4}$")
                            .is_match(f.get(0).unwrap().as_str())
                            .unwrap_or(false);
                        if valid && !skipped {
                            let mut meridiem = if h >= 12 { Some(1) } else { None };
                            if let Some(ap) = ap {
                                if ap.as_str().to_lowercase().starts_with('a') {
                                    meridiem = Some(0);
                                    if h == 12 {
                                        h = 0;
                                        e.imply(D, e.get(D) + 1);
                                    }
                                } else {
                                    meridiem = Some(1);
                                    if h != 12 {
                                        h += 12
                                    }
                                }
                                if !c.certain(AP) {
                                    if meridiem == Some(0) {
                                        c.imply(AP, 0);
                                        if c.get(H) == 12 {
                                            c.assign(H, 0)
                                        }
                                    } else {
                                        c.imply(AP, 1);
                                        if c.get(H) != 12 {
                                            c.assign(H, c.get(H) + 12)
                                        }
                                    }
                                }
                            }
                            e.assign(H, h);
                            e.assign(MIN, minute);
                            if let Some(v) = sec {
                                e.assign(S, v)
                            }
                            if let Some(v) = millis {
                                e.assign(MS, v)
                            }
                            if let Some(v) = meridiem {
                                e.assign(AP, v)
                            } else if c.certain(AP) && c.get(H) > 12 {
                                if c.get(H) - 12 > h {
                                    e.imply(AP, 0)
                                } else if h <= 12 {
                                    e.assign(H, h + 12);
                                    e.assign(AP, 1)
                                }
                            } else {
                                e.imply(AP, if h > 12 { 1 } else { 0 })
                            }
                            if e.instant(tz, now, false) < c.instant(tz, now, false) {
                                e.imply(D, e.get(D) + 1)
                            }
                            to = Some(e);
                            actual_end = end + f.get(0).unwrap().end();
                        }
                        if rx(r"^\d+-\d+$")
                            .is_match(&text[start..actual_end])
                            .unwrap_or(false)
                        {
                            pos = actual_end.max(pos + 1);
                            continue;
                        }
                    } else if v[0].chars().all(|c| c.is_ascii_digit()) {
                        pos = end.max(pos + 1);
                        continue;
                    }
                }
                rs.push(R {
                    start,
                    end: actual_end,
                    c,
                    to,
                });
                if rs.len() > 512 {
                    return Err("Natural language result limit (512)".into());
                }
            }
            pos = actual_end.max(pos + 1);
            while pos < text.len() && !text.is_char_boundary(pos) {
                pos += 1
            }
        }
    }
    rs.sort_by_key(|r| r.start);
    rs = overlap(rs);
    for mode in 0..2 {
        let mut i = 0;
        while i + 1 < rs.len() {
            let a = &rs[i];
            let b = &rs[i + 1];
            if a.end > b.start {
                i += 1;
                continue;
            }
            let between = &text[a.end..b.start];
            let atext = text[a.start..a.end].to_lowercase();
            let btext = &text[b.start..b.end];
            let merge = between.trim().is_empty()
                && if mode == 0 {
                    btext.starts_with(['+', '-'])
                } else {
                    [" before", " from", " after", " since"]
                        .iter()
                        .any(|s| atext.ends_with(s))
                };
            if merge {
                let reference = if mode == 0 {
                    a.c.instant(tz, now, false)
                } else {
                    b.c.instant(tz, now, false)
                };
                let duration_text = if mode == 0 {
                    btext
                } else {
                    &text[a.start..a.end]
                };
                let negative = if mode == 0 {
                    btext.starts_with('-')
                } else {
                    atext.ends_with(" before") || atext.ends_with(" from")
                };
                if let Some(reference) = reference {
                    if let Some(c) = relative(reference, tz, durations(duration_text, negative)?) {
                        let end = b.end;
                        rs[i].c = c;
                        rs[i].end = end;
                        rs.remove(i + 1);
                        continue;
                    }
                }
            }
            i += 1
        }
    }
    for pass in 0..2 {
        if pass == 0 {
            for r in &mut rs {
                if r.c.certain(TZ) {
                    continue;
                }
                if let Some(m) = rx(r"^\s*(?:\(?(?:GMT|UTC)\s?)?([+-])(\d{1,2})(?::?(\d{2}))?\)?")
                    .captures(&text[r.end..])
                    .map_err(|_| "Regex resource limit")?
                {
                    let h = m[2].parse::<i32>().unwrap_or(0);
                    let min = m
                        .get(3)
                        .and_then(|s| s.as_str().parse::<i32>().ok())
                        .unwrap_or(0);
                    let offset = h * 60 + min;
                    if offset <= 840 {
                        r.c.assign(TZ, offset * if &m[1] == "-" { -1 } else { 1 });
                        r.end += m.get(0).unwrap().end()
                    }
                }
            }
        }
        let mut i = 0;
        while i + 1 < rs.len() {
            if rs[i].end > rs[i + 1].start {
                i += 1;
                continue;
            }
            let a = &rs[i];
            let b = &rs[i + 1];
            let between = &text[a.end..b.start];
            let weekday = pass == 0
                && a.c.only_weekday()
                && !a.c.certain(H)
                && b.c.certain(D)
                && between.trim_matches(',').trim().is_empty();
            let datetime = ((a.c.only_date() && b.c.only_time())
                || (a.c.only_time() && b.c.only_date()))
                && rx(r"^\s*(T|at|after|before|on|of|,|-|\.|∙|:)?\s*$")
                    .is_match(between)
                    .unwrap_or(false);
            if weekday || datetime {
                let mut c = if weekday {
                    b.c.clone()
                } else if a.c.only_date() {
                    merge_time(&a.c, &b.c)
                } else {
                    merge_time(&b.c, &a.c)
                };
                if weekday {
                    c.assign(WD, a.c.get(WD))
                }
                let merged_to = if weekday {
                    let mut e = b.to.clone();
                    if let Some(e) = &mut e {
                        e.assign(WD, a.c.get(WD))
                    }
                    e
                } else if a.to.is_some() || b.to.is_some() {
                    let (date, time) = if a.c.only_date() { (a, b) } else { (b, a) };
                    let mut end = merge_time(
                        date.to.as_ref().unwrap_or(&date.c),
                        time.to.as_ref().unwrap_or(&time.c),
                    );
                    if date.to.is_none() && end.instant(tz, now, false) < c.instant(tz, now, false)
                    {
                        if let Some(n) = end.naive() {
                            let n = n + Duration::days(1);
                            let known = end.certain(D);
                            end.date(n, known)
                        }
                    }
                    Some(end)
                } else {
                    None
                };
                let end = b.end;
                rs[i].to = merged_to;
                rs[i].c = c;
                rs[i].end = end;
                rs.remove(i + 1);
            } else {
                i += 1
            }
        }
        if pass == 0 {
            for r in &mut rs {
                if let Some(m) = rx(r"^\s*,?\s*\(?([A-Z]{2,4})\)?(?=\W|$)")
                    .captures(&text[r.end..])
                    .map_err(|_| "Regex resource limit")?
                {
                    let word = &m[1];
                    if let Some(offset) = timezone_abbr(word, &r.c) {
                        if r.c.certain(TZ) && r.c.get(TZ) != offset {
                            continue;
                        }
                        if word != word.to_uppercase()
                            && (r.c.only_date() || r.c.v[TZ].is_some_and(|v| v != offset))
                        {
                            continue;
                        }
                        r.end += m.get(0).unwrap().end();
                        if !r.c.certain(TZ) {
                            r.c.assign(TZ, offset)
                        }
                    }
                }
            }
            rs = overlap(rs);
            // The locked common validity filter runs BEFORE the second datetime
            // merge and year-suffix refiner. Do not revalidate after year/ranges:
            // later Date construction can intentionally normalize those fields.
            rs.retain(|r| {
                !rx(r"^\d*(\.\d*)?$")
                    .is_match(&text[r.start..r.end].replacen(' ', "", 1))
                    .unwrap_or(false)
                    && r.c.instant(tz, now, true).is_some()
                    && r.to
                        .as_ref()
                        .is_none_or(|c| c.instant(tz, now, true).is_some())
            });
        }
    }
    let suffix_year = rx(&format!(
        r"^\s*({})",
        DATA.constants["YEAR_PATTERN"].as_str().unwrap()
    ));
    for r in &mut rs {
        if r.c.certain(M) && !r.c.certain(Y) {
            if let Some(m) = suffix_year
                .captures(&text[r.end..])
                .map_err(|_| "Regex resource limit")?
            {
                if m.get(0).unwrap().as_str().trim().len() > 3 {
                    let y = year(&m[1]);
                    r.c.assign(Y, y);
                    if let Some(e) = &mut r.to {
                        e.assign(Y, y)
                    }
                    r.end += m.get(0).unwrap().end();
                }
            }
        }
    }
    let mut i = 0;
    while i + 1 < rs.len() {
        let a = &rs[i];
        let b = &rs[i + 1];
        if a.end > b.start {
            i += 1;
            continue;
        }
        if a.to.is_none()
            && b.to.is_none()
            && rx(r"^\s*(to|-|–|until|through|till)\s*$")
                .is_match(&text[a.end..b.start])
                .unwrap_or(false)
        {
            let mut a = a.clone();
            let mut b = b.clone();
            if !a.c.only_weekday() && !b.c.only_weekday() {
                for k in 0..10 {
                    if b.c.certain(k) {
                        a.c.imply(k, b.c.get(k))
                    }
                    if a.c.certain(k) {
                        b.c.imply(k, a.c.get(k))
                    }
                }
            }
            if a.c.instant(tz, now, false) > b.c.instant(tz, now, false) {
                if b.c.only_weekday() {
                    if let Some(n) = b.c.naive() {
                        b.c.date(n + Duration::days(7), false)
                    }
                } else if a.c.only_weekday() {
                    if let Some(n) = a.c.naive() {
                        a.c.date(n - Duration::days(7), false)
                    }
                } else if b.c.certain(M) && !b.c.certain(Y) {
                    b.c.imply(Y, b.c.get(Y) + 1)
                } else if a.c.certain(M) && !a.c.certain(Y) {
                    a.c.imply(Y, a.c.get(Y) - 1)
                } else {
                    std::mem::swap(&mut a.c, &mut b.c)
                }
            }
            rs[i].c = a.c;
            rs[i].to = Some(b.c);
            rs[i].end = b.end;
            rs.remove(i + 1);
        } else {
            i += 1
        }
    }
    for r in rs {
        let matched = text[r.start..r.end].trim();
        if rx(r"^\d*(\.\d*)?$")
            .is_match(&matched.replacen(' ', "", 1))
            .unwrap_or(false)
        {
            continue;
        }
        if matched != text.trim()
            && matched.eq_ignore_ascii_case("may")
            && !text[..r.start].trim_end().to_lowercase().ends_with("in")
        {
            continue;
        }
        if matched.to_lowercase().ends_with("the second") && matched != text.trim() {
            continue;
        }
        if let Some(ms) = r.c.instant(tz, now, false) {
            return Ok(Some(ms));
        }
    }
    Ok(None)
}
