//! Dates, instants and reminder offsets (invariant 6: explicit time zones).

use jiff::civil::{Date, DateTime, Time};
use jiff::tz::TimeZone;
use jiff::{Span, Timestamp, ToSpan, Zoned};
use serde::Serialize;

use crate::error::{Result, rule};

/// Something with a date: either a whole day or an exact instant.
///
/// Serialized for the UI with the local (configured time zone) date and time
/// precomputed, plus the absolute instant, so the front end never does time-zone
/// math of its own.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct When {
    /// Local calendar date.
    pub date: Date,
    /// Local wall-clock time, absent for all-day items.
    pub time: Option<Time>,
    /// The instant, absent for all-day items.
    pub at: Option<Timestamp>,
}

impl When {
    pub fn day(date: Date) -> When {
        When {
            date,
            time: None,
            at: None,
        }
    }

    pub fn instant(at: Timestamp, tz: &TimeZone) -> When {
        let z = at.to_zoned(tz.clone());
        When {
            date: z.date(),
            time: Some(z.time()),
            at: Some(at),
        }
    }

    pub fn all_day(&self) -> bool {
        self.at.is_none()
    }

    /// Parses `2026-12-05`, `2026-11-04T09:00:00-03:00`, or a local
    /// `2026-11-04T09:00` / `2026-11-04 09:00` (interpreted in `tz`).
    pub fn parse(s: &str, tz: &TimeZone) -> Result<When> {
        let s = s.trim();
        // jiff's `Date` parser also accepts datetimes (dropping the time), so
        // only a bare `YYYY-MM-DD` counts as all-day.
        if !s.contains(['T', 't', ' '])
            && let Ok(d) = s.parse::<Date>()
        {
            return Ok(When::day(d));
        }
        parse_instant(s, tz).map(|ts| When::instant(ts, tz))
    }

    /// The text written to files.
    pub fn to_file_string(&self, tz: &TimeZone) -> String {
        match self.at {
            Some(ts) => format_instant(ts, tz),
            None => self.date.to_string(),
        }
    }

    /// Start of the item, for sorting and windows; all-day items start at local midnight.
    pub fn start(&self, tz: &TimeZone) -> Timestamp {
        self.at.unwrap_or_else(|| {
            self.date
                .to_zoned(tz.clone())
                .map(|z| z.timestamp())
                .unwrap_or(Timestamp::UNIX_EPOCH)
        })
    }
}

pub fn parse_instant(s: &str, tz: &TimeZone) -> Result<Timestamp> {
    let s = s.trim();
    if let Ok(ts) = s.parse::<Timestamp>() {
        return Ok(ts);
    }
    let normalized = s.replacen(' ', "T", 1);
    match normalized.parse::<DateTime>() {
        Ok(dt) => dt
            .to_zoned(tz.clone())
            .map(|z| z.timestamp())
            .or_else(|e| rule(format!("data inválida '{s}': {e}"))),
        Err(_) => rule(format!(
            "data inválida '{s}': use AAAA-MM-DD ou AAAA-MM-DDTHH:MM (com fuso opcional)"
        )),
    }
}

/// RFC 3339 with the offset of the configured zone, e.g. `2026-11-04T09:00:00-03:00`.
pub fn format_instant(ts: Timestamp, tz: &TimeZone) -> String {
    ts.to_zoned(tz.clone()).strftime("%Y-%m-%dT%H:%M:%S%:z").to_string()
}

pub fn zone(name: &str) -> Result<TimeZone> {
    TimeZone::get(name).or_else(|e| rule(format!("fuso horário desconhecido '{name}': {e}")))
}

pub fn parse_time(s: &str) -> Result<Time> {
    let s = s.trim();
    let padded = if s.len() == 4 { format!("0{s}") } else { s.to_string() };
    padded
        .parse::<Time>()
        .or_else(|_| rule(format!("horário inválido '{s}': use HH:MM")))
}

pub fn fmt_hm(t: Time) -> String {
    format!("{:02}:{:02}", t.hour(), t.minute())
}

/// One entry of a `reminders:` list.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReminderSpec {
    /// `-1h`, `0`, `+15m`, `-7d`, `-2w`: relative to the item.
    Offset { amount: i64, unit: char },
    /// `09:00`: at that time of day, on the item's day.
    AtTime(Time),
}

impl ReminderSpec {
    pub fn parse(s: &str) -> Result<ReminderSpec> {
        let s = s.trim();
        if s == "0" {
            return Ok(ReminderSpec::Offset { amount: 0, unit: 'm' });
        }
        if s.contains(':') {
            return parse_time(s).map(ReminderSpec::AtTime);
        }
        let unit = s.chars().last().unwrap_or(' ');
        if !matches!(unit, 'm' | 'h' | 'd' | 'w') {
            return rule(format!("lembrete inválido '{s}': use -15m, -1h, -7d, -1w, 0 ou HH:MM"));
        }
        let amount: i64 = s[..s.len() - 1]
            .parse()
            .or_else(|_| rule(format!("lembrete inválido '{s}'")))?;
        Ok(ReminderSpec::Offset { amount, unit })
    }

    fn span(amount: i64, unit: char) -> Span {
        match unit {
            'm' => amount.minutes(),
            'h' => amount.hours(),
            'd' => amount.days(),
            _ => amount.weeks(),
        }
    }

    /// When this reminder fires for `item`. All-day items use `day_time` as their
    /// time of day (the `HH:MM` entry of the list, or 09:00).
    pub fn fire_at(&self, item: &When, day_time: Time, tz: &TimeZone) -> Option<Timestamp> {
        let base: Zoned = match item.at {
            Some(at) => at.to_zoned(tz.clone()),
            None => item.date.to_datetime(day_time).to_zoned(tz.clone()).ok()?,
        };
        match *self {
            ReminderSpec::Offset { amount, unit } => {
                base.checked_add(Self::span(amount, unit)).ok().map(|z| z.timestamp())
            }
            ReminderSpec::AtTime(t) => base
                .date()
                .to_datetime(t)
                .to_zoned(tz.clone())
                .ok()
                .map(|z| z.timestamp()),
        }
    }
}

/// Parses a reminder list; returns the specs and the time of day for all-day items.
pub fn reminder_plan(list: &[String]) -> (Vec<ReminderSpec>, Time) {
    let specs: Vec<ReminderSpec> = list.iter().filter_map(|s| ReminderSpec::parse(s).ok()).collect();
    let day_time = specs
        .iter()
        .find_map(|s| match s {
            ReminderSpec::AtTime(t) => Some(*t),
            _ => None,
        })
        .unwrap_or_else(|| Time::constant(9, 0, 0, 0));
    (specs, day_time)
}

/// Quiet hours: a daily window (may cross midnight) in which notifications wait.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct QuietHours {
    pub from: Time,
    pub to: Time,
}

impl QuietHours {
    fn contains(&self, t: Time) -> bool {
        if self.from <= self.to {
            t >= self.from && t < self.to
        } else {
            t >= self.from || t < self.to
        }
    }

    /// Moves an instant inside the window to the end of the window.
    pub fn defer(&self, ts: Timestamp, tz: &TimeZone) -> Timestamp {
        let z = ts.to_zoned(tz.clone());
        if !self.contains(z.time()) {
            return ts;
        }
        let mut day = z.date();
        if self.from > self.to && z.time() >= self.from {
            day = day.tomorrow().unwrap_or(day);
        }
        day.to_datetime(self.to)
            .to_zoned(tz.clone())
            .map(|z| z.timestamp())
            .unwrap_or(ts)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jiff::civil::date;

    fn tz() -> TimeZone {
        zone("America/Recife").unwrap()
    }

    #[test]
    fn parse_and_format() {
        let w = When::parse("2026-11-04T09:00", &tz()).unwrap();
        assert_eq!(w.to_file_string(&tz()), "2026-11-04T09:00:00-03:00");
        let w2 = When::parse("2026-11-04T12:00:00Z", &tz()).unwrap();
        assert_eq!(w, w2);
        assert_eq!(w.time, Some(Time::constant(9, 0, 0, 0)));
        let d = When::parse("2026-12-05", &tz()).unwrap();
        assert!(d.all_day());
        assert_eq!(d.to_file_string(&tz()), "2026-12-05");
        assert!(When::parse("amanhã", &tz()).is_err());
    }

    #[test]
    fn reminders_for_timed_and_all_day_items() {
        let tz = tz();
        let post = When::parse("2026-11-04T09:00", &tz).unwrap();
        let (specs, day_time) = reminder_plan(&["-1h".into(), "0".into()]);
        let fired: Vec<String> = specs
            .iter()
            .map(|s| format_instant(s.fire_at(&post, day_time, &tz).unwrap(), &tz))
            .collect();
        assert_eq!(fired, ["2026-11-04T08:00:00-03:00", "2026-11-04T09:00:00-03:00"]);

        let deadline = When::day(date(2026, 12, 5));
        let (specs, day_time) = reminder_plan(&["-7d".into(), "-1d".into(), "09:00".into()]);
        let fired: Vec<String> = specs
            .iter()
            .map(|s| format_instant(s.fire_at(&deadline, day_time, &tz).unwrap(), &tz))
            .collect();
        assert_eq!(
            fired,
            [
                "2026-11-28T09:00:00-03:00",
                "2026-12-04T09:00:00-03:00",
                "2026-12-05T09:00:00-03:00"
            ]
        );
    }

    #[test]
    fn quiet_hours_cross_midnight() {
        let tz = tz();
        let q = QuietHours {
            from: Time::constant(23, 0, 0, 0),
            to: Time::constant(7, 0, 0, 0),
        };
        let at = |s: &str| parse_instant(s, &tz).unwrap();
        assert_eq!(q.defer(at("2026-11-04T23:30"), &tz), at("2026-11-05T07:00"));
        assert_eq!(q.defer(at("2026-11-05T02:00"), &tz), at("2026-11-05T07:00"));
        assert_eq!(q.defer(at("2026-11-05T08:00"), &tz), at("2026-11-05T08:00"));
    }

    #[test]
    fn spec_parsing() {
        assert!(ReminderSpec::parse("-15m").is_ok());
        assert!(ReminderSpec::parse("+2w").is_ok());
        assert!(ReminderSpec::parse("9:30").is_ok());
        assert!(ReminderSpec::parse("-1x").is_err());
    }
}
