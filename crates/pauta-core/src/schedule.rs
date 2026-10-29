//! Free slots, the agenda, the "Today" summary and reminders (sections 8 to 10).
//! Pure functions over an [`Index`] and an explicit `now`.

use std::collections::HashMap;

use jiff::civil::Date;
use jiff::{SignedDuration, Timestamp, ToSpan};
use serde::Serialize;

use crate::error::{Result, rule};
use crate::model::{Card, ColumnRole, PubStatus};
use crate::store::Index;
use crate::time::{When, reminder_plan};

/// Next `count` free posting slots of `network` after `after`.
///
/// A day with *k* publications on that network fills its first *k* slots, so a
/// post at an arbitrary time still "uses up" a slot that day. `ignore` excludes
/// one card (the one being scheduled) from the count.
pub fn find_slots(
    index: &Index,
    network: &str,
    after: Timestamp,
    count: usize,
    ignore: Option<&str>,
) -> Result<Vec<Timestamp>> {
    let cfg = &index.config;
    let tz = &cfg.tz;
    let Some(net) = cfg.network(network) else {
        return rule(format!("rede '{network}' não configurada"));
    };
    if net.slots.is_empty() {
        return Ok(Vec::new());
    }
    let mut used: HashMap<Date, usize> = HashMap::new();
    for c in &index.cards {
        if c.network.as_deref() != Some(network) || Some(c.id.as_str()) == ignore {
            continue;
        }
        if let Some(p) = &c.publication
            && p.status != PubStatus::Skipped
            && let Some(at) = &p.at
        {
            *used.entry(at.date).or_default() += 1;
        }
    }
    let mut out = Vec::new();
    let start = after.to_zoned(tz.clone()).date();
    for offset in 0..366 {
        let Ok(day) = start.checked_add(offset.days()) else {
            break;
        };
        let mut times: Vec<_> = net
            .slots
            .iter()
            .filter(|s| s.weekdays().contains(&day.weekday()))
            .filter_map(|s| s.time_of_day())
            .collect();
        times.sort();
        times.dedup();
        let taken = used.get(&day).copied().unwrap_or(0);
        for t in times.into_iter().skip(taken) {
            let Ok(z) = day.to_datetime(t).to_zoned(tz.clone()) else {
                continue;
            };
            if z.timestamp() > after {
                out.push(z.timestamp());
                if out.len() == count {
                    return Ok(out);
                }
            }
        }
    }
    Ok(out)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum AgendaKind {
    Publication,
    Due,
    Event,
}

/// Anything with a date, from any board or the calendar folder.
#[derive(Debug, Clone, Serialize)]
pub struct AgendaItem {
    pub kind: AgendaKind,
    pub id: String,
    pub board: Option<String>,
    pub title: String,
    pub when: When,
    pub end: Option<When>,
    pub network: Option<String>,
    /// Publication status, or the event kind (`deadline`, `launch`, ...).
    pub status: Option<String>,
    pub column: Option<String>,
    pub done: bool,
}

fn card_items(index: &Index, c: &Card) -> Vec<AgendaItem> {
    let mut out = Vec::new();
    let done = index.is_done(c);
    if let Some(p) = &c.publication
        && let Some(at) = &p.at
    {
        out.push(AgendaItem {
            kind: AgendaKind::Publication,
            id: c.id.clone(),
            board: Some(c.board.clone()),
            title: c.title.clone(),
            when: at.clone(),
            end: None,
            network: c.network.clone(),
            status: Some(p.status.as_str().into()),
            column: Some(c.column.clone()),
            done: !p.status.pending(),
        });
    }
    if let Some(due) = &c.due {
        out.push(AgendaItem {
            kind: AgendaKind::Due,
            id: c.id.clone(),
            board: Some(c.board.clone()),
            title: c.title.clone(),
            when: due.clone(),
            end: None,
            network: c.network.clone(),
            status: None,
            column: Some(c.column.clone()),
            done,
        });
    }
    out
}

pub fn all_items(index: &Index) -> Vec<AgendaItem> {
    let mut out: Vec<AgendaItem> = index.cards.iter().flat_map(|c| card_items(index, c)).collect();
    for e in &index.events {
        out.push(AgendaItem {
            kind: AgendaKind::Event,
            id: e.id.clone(),
            board: e.links.board.clone(),
            title: e.title.clone(),
            when: e.when.clone(),
            end: e.end.clone(),
            network: None,
            status: Some(e.kind.clone()),
            column: None,
            done: false,
        });
    }
    let tz = &index.config.tz;
    out.sort_by_key(|i| (i.when.start(tz), i.when.all_day()));
    out
}

/// Everything dated between `from` and `to` (inclusive local dates).
pub fn agenda(index: &Index, from: Date, to: Date) -> Vec<AgendaItem> {
    all_items(index)
        .into_iter()
        .filter(|i| {
            let end = i.end.as_ref().map_or(i.when.date, |e| e.date);
            i.when.date <= to && end >= from
        })
        .collect()
}

#[derive(Debug, Clone, Serialize)]
pub struct CardRef {
    pub id: String,
    pub board: String,
    pub title: String,
    pub column: String,
    pub network: Option<String>,
}

impl From<&Card> for CardRef {
    fn from(c: &Card) -> Self {
        CardRef {
            id: c.id.clone(),
            board: c.board.clone(),
            title: c.title.clone(),
            column: c.column.clone(),
            network: c.network.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct EmptyQueue {
    pub network: String,
    pub label: String,
    pub days: i64,
    /// Cards that could fill the gap (drafts and productions for that network).
    pub candidates: Vec<CardRef>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Today {
    pub date: Date,
    /// Local wall-clock time (whole minutes) in the configured zone, for the "now" line.
    pub time: jiff::civil::Time,
    pub posts: Vec<AgendaItem>,
    pub upcoming: Vec<AgendaItem>,
    pub overdue: Vec<AgendaItem>,
    pub empty_queues: Vec<EmptyQueue>,
    pub stale: Vec<CardRef>,
    pub cadence: Vec<crate::rhythm::CadenceProgress>,
    pub invalid_files: usize,
}

/// The default screen (section 8.2).
pub fn today(index: &Index, now: Timestamp) -> Today {
    let cfg = &index.config;
    let tz = &cfg.tz;
    let local = now.to_zoned(tz.clone());
    let date = local.date();
    let time = jiff::civil::time(local.hour(), local.minute(), 0, 0);
    let items = all_items(index);
    let horizon = date.checked_add(7.days()).unwrap_or(date);

    let posts = items
        .iter()
        .filter(|i| i.kind == AgendaKind::Publication && i.when.date == date && i.status.as_deref() != Some("skipped"))
        .cloned()
        .collect();
    let upcoming = items
        .iter()
        .filter(|i| i.kind != AgendaKind::Publication && !i.done && i.when.date >= date && i.when.date <= horizon)
        .filter(|i| i.when.all_day() || i.when.at.is_some_and(|a| a >= now) || i.kind == AgendaKind::Event)
        .cloned()
        .collect();
    let overdue = items
        .iter()
        .filter(|i| i.kind != AgendaKind::Event && !i.done)
        .filter(|i| match i.when.at {
            Some(at) => at < now,
            None => i.when.date < date,
        })
        .cloned()
        .collect();

    let mut empty_queues = Vec::new();
    let queue_end = now + SignedDuration::from_hours(24 * cfg.empty_queue_days);
    for net in &cfg.networks {
        let has_next = index.cards.iter().any(|c| {
            c.network.as_deref() == Some(net.id.as_str())
                && c.pending_publication_at()
                    .and_then(|w| w.at)
                    .is_some_and(|at| at >= now && at <= queue_end)
        });
        if has_next {
            continue;
        }
        let candidates = index
            .cards
            .iter()
            .filter(|c| c.network.as_deref().is_none_or(|n| n == net.id))
            .filter(|c| {
                let board = index.board(&c.board);
                board.is_some_and(|b| b.kind == crate::model::BoardKind::Content)
                    && !index.is_done(c)
                    && c.pending_publication_at().is_none()
                    && board
                        .and_then(|b| b.column(&c.column))
                        .is_some_and(|col| col.role != Some(ColumnRole::Inbox))
            })
            .take(5)
            .map(CardRef::from)
            .collect();
        empty_queues.push(EmptyQueue {
            network: net.id.clone(),
            label: net.label.clone(),
            days: cfg.empty_queue_days,
            candidates,
        });
    }

    let stale_cutoff = now - SignedDuration::from_hours(24 * cfg.stale_after_days);
    let stale = index
        .cards
        .iter()
        // Time in the current column; cards without that record fall back to the last edit.
        .filter(|c| !index.is_done(c) && c.column_since.or(c.updated).is_some_and(|u| u < stale_cutoff))
        .filter(|c| {
            index
                .board(&c.board)
                .and_then(|b| b.column(&c.column))
                .is_some_and(|col| col.role != Some(ColumnRole::Inbox))
        })
        .take(10)
        .map(CardRef::from)
        .collect();

    Today {
        date,
        time,
        posts,
        upcoming,
        overdue,
        empty_queues,
        stale,
        cadence: crate::rhythm::cadence(index, now),
        invalid_files: index.invalid.len(),
    }
}

/// One firing of a reminder for one item.
#[derive(Debug, Clone, Serialize)]
pub struct Reminder {
    /// Stable identity: changes when the item is rescheduled, so a new time
    /// produces a new reminder.
    pub key: String,
    pub fire_at: Timestamp,
    pub spec: String,
    pub item: AgendaItem,
}

/// The reminder specs of an item: its own, or the configured defaults for its kind.
pub fn reminder_list(index: &Index, item: &AgendaItem) -> Vec<String> {
    let cfg = &index.config;
    match item.kind {
        AgendaKind::Publication => index
            .card(&item.id)
            .and_then(|c| c.reminders.clone())
            .unwrap_or_else(|| cfg.reminders.post_default.clone()),
        AgendaKind::Due => index
            .card(&item.id)
            .and_then(|c| c.reminders.clone())
            .unwrap_or_else(|| cfg.reminders.deadline_default.clone()),
        AgendaKind::Event => index
            .event(&item.id)
            .and_then(|e| e.reminders.clone())
            .unwrap_or_else(|| cfg.reminders.deadline_default.clone()),
    }
}

/// All reminders of pending items firing in `[from, to]`, after quiet hours.
pub fn reminders_between(index: &Index, from: Timestamp, to: Timestamp) -> Vec<Reminder> {
    let cfg = &index.config;
    let tz = &cfg.tz;
    let mut out = Vec::new();
    for item in all_items(index) {
        if item.done {
            continue;
        }
        let list = reminder_list(index, &item);
        let (specs, day_time) = reminder_plan(&list);
        for (spec, raw) in specs
            .iter()
            .zip(list.iter().filter(|s| crate::time::ReminderSpec::parse(s).is_ok()))
        {
            let Some(mut fire) = spec.fire_at(&item.when, day_time, tz) else {
                continue;
            };
            if let Some(q) = &cfg.quiet_hours {
                fire = q.defer(fire, tz);
            }
            if fire < from || fire > to {
                continue;
            }
            let kind = match item.kind {
                AgendaKind::Publication => "pub",
                AgendaKind::Due => "due",
                AgendaKind::Event => "event",
            };
            out.push(Reminder {
                key: format!("{kind}:{}:{}:{raw}", item.id, item.when.to_file_string(tz)),
                fire_at: fire,
                spec: raw.clone(),
                item: item.clone(),
            });
        }
    }
    out.sort_by_key(|r| r.fire_at);
    out
}

/// The next reminder strictly after `now` (for the tray and the scheduler).
pub fn next_reminder(index: &Index, now: Timestamp) -> Option<Reminder> {
    reminders_between(
        index,
        now + SignedDuration::from_secs(1),
        now + SignedDuration::from_hours(24 * 60),
    )
    .into_iter()
    .next()
}
