//! iCalendar (.ics) export (section 17.3): posts, card deadlines and calendar events
//! for a phone calendar to import. One-way and read-only: nothing flows back.
//!
//! Times are written in UTC and all-day items as dates, so no VTIMEZONE block is
//! needed. Reminders become absolute alarms computed by `pauta-core`, so a phone
//! rings at the same moment the desktop would (quiet hours aside: the phone has its own).

use jiff::civil::Date;
use jiff::{SignedDuration, Timestamp, ToSpan};

use crate::schedule::{AgendaItem, AgendaKind, all_items, reminder_list};
use crate::store::Index;
use crate::time::{ReminderSpec, reminder_plan};

#[derive(Debug, Clone, Copy, Default)]
pub struct IcsOptions {
    /// Also export published posts and finished cards (default: only what is still ahead).
    pub include_done: bool,
}

/// Length of the block for a timed item that has no end.
const DEFAULT_DURATION: SignedDuration = SignedDuration::from_mins(30);

fn utc(t: Timestamp) -> String {
    t.strftime("%Y%m%dT%H%M%SZ").to_string()
}

fn date(d: Date) -> String {
    format!("{:04}{:02}{:02}", d.year(), d.month(), d.day())
}

/// RFC 5545 text escaping.
fn escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            ';' => out.push_str("\\;"),
            ',' => out.push_str("\\,"),
            '\n' => out.push_str("\\n"),
            '\r' => {}
            c => out.push(c),
        }
    }
    out
}

/// Folds a content line at 75 octets without splitting a character, and ends it with CRLF.
fn push_line(out: &mut String, line: &str) {
    let mut width = 0;
    for c in line.chars() {
        let len = c.len_utf8();
        // Continuation lines start with a space, which counts towards the limit.
        if width + len > 75 {
            out.push_str("\r\n ");
            width = 1;
        }
        out.push(c);
        width += len;
    }
    out.push_str("\r\n");
}

fn summary(index: &Index, i: &AgendaItem) -> String {
    match i.kind {
        AgendaKind::Publication => {
            let label = i
                .network
                .as_deref()
                .and_then(|n| index.config.network(n))
                .map_or("Post", |n| n.label.as_str());
            format!("[{label}] {}", i.title)
        }
        AgendaKind::Due => format!("Prazo: {}", i.title),
        AgendaKind::Event => match i.status.as_deref() {
            Some("deadline") => format!("Prazo: {}", i.title),
            Some("milestone") => format!("Marco: {}", i.title),
            Some("launch") => format!("Lançamento: {}", i.title),
            _ => i.title.clone(),
        },
    }
}

fn describe(index: &Index, i: &AgendaItem) -> String {
    let board = i
        .board
        .as_deref()
        .map(|b| index.board(b).map_or(b, |b| b.name.as_str()));
    let mut parts = vec!["Pauta".to_string()];
    parts.extend(board.map(str::to_string));
    if i.kind == AgendaKind::Publication
        && let Some(s) = &i.status
    {
        parts.push(format!("estado: {s}"));
    }
    parts.join(" · ")
}

/// Builds the calendar text, with CRLF line endings.
pub fn export(index: &Index, now: Timestamp, opts: IcsOptions) -> String {
    let tz = &index.config.tz;
    let mut out = String::new();
    for l in [
        "BEGIN:VCALENDAR",
        "VERSION:2.0",
        "PRODID:-//Pauta//Pauta calendar//PT-BR",
        "CALSCALE:GREGORIAN",
        "METHOD:PUBLISH",
        "X-WR-CALNAME:Pauta",
    ] {
        push_line(&mut out, l);
    }
    let stamp = utc(now);
    for item in all_items(index).iter().filter(|i| opts.include_done || !i.done) {
        let kind = match item.kind {
            AgendaKind::Publication => "pub",
            AgendaKind::Due => "due",
            AgendaKind::Event => "event",
        };
        let title = summary(index, item);
        push_line(&mut out, "BEGIN:VEVENT");
        push_line(&mut out, &format!("UID:{kind}-{}@pauta", item.id));
        push_line(&mut out, &format!("DTSTAMP:{stamp}"));
        match item.when.at {
            Some(start) => {
                let end = item
                    .end
                    .as_ref()
                    .and_then(|e| e.at)
                    .filter(|e| *e > start)
                    .unwrap_or_else(|| start + DEFAULT_DURATION);
                push_line(&mut out, &format!("DTSTART:{}", utc(start)));
                push_line(&mut out, &format!("DTEND:{}", utc(end)));
            }
            None => {
                // All-day: DTEND is exclusive, so it is the day after the last day.
                let last = item.end.as_ref().map_or(item.when.date, |e| e.date).max(item.when.date);
                let end = last.checked_add(1.day()).unwrap_or(last);
                push_line(&mut out, &format!("DTSTART;VALUE=DATE:{}", date(item.when.date)));
                push_line(&mut out, &format!("DTEND;VALUE=DATE:{}", date(end)));
            }
        }
        push_line(&mut out, &format!("SUMMARY:{}", escape(&title)));
        push_line(&mut out, &format!("DESCRIPTION:{}", escape(&describe(index, item))));
        if item.kind == AgendaKind::Publication && item.status.as_deref() != Some("published") {
            push_line(&mut out, "STATUS:TENTATIVE");
        }

        let list = reminder_list(index, item);
        let (specs, day_time) = reminder_plan(&list);
        let mut fires: Vec<Timestamp> = specs
            .iter()
            .filter_map(|s: &ReminderSpec| s.fire_at(&item.when, day_time, tz))
            .filter(|t| *t > now)
            .collect();
        fires.sort();
        fires.dedup();
        for t in fires {
            push_line(&mut out, "BEGIN:VALARM");
            push_line(&mut out, "ACTION:DISPLAY");
            push_line(&mut out, &format!("DESCRIPTION:{}", escape(&title)));
            push_line(&mut out, &format!("TRIGGER;VALUE=DATE-TIME:{}", utc(t)));
            push_line(&mut out, "END:VALARM");
        }
        push_line(&mut out, "END:VEVENT");
    }
    push_line(&mut out, "END:VCALENDAR");
    out
}
