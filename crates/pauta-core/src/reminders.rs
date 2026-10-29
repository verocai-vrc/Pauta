//! Which reminders were delivered or snoozed (sections 9.2 and 9.5).
//!
//! Kept in `.pauta/reminders.json`. The folder is disposable: losing it only means
//! reminders from the last few days may be shown again as missed.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use jiff::{SignedDuration, Timestamp};
use serde::{Deserialize, Serialize};

use crate::config::Config;
use crate::error::Result;
use crate::schedule::{Reminder, reminders_between};
use crate::store::{Index, write_atomic};

/// How far back a reminder can still be reported as missed.
pub const LOOKBACK_HOURS: i64 = 72;
/// A reminder this late is "missed" rather than delivered as if on time.
pub const GRACE_MINUTES: i64 = 10;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReminderState {
    #[serde(default)]
    pub delivered: BTreeMap<String, Timestamp>,
    #[serde(default)]
    pub snoozed: BTreeMap<String, Timestamp>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct Due {
    /// On time: one notification each.
    pub fresh: Vec<Reminder>,
    /// Missed while the computer was off or asleep: one grouped notification,
    /// one entry per item.
    pub missed: Vec<Reminder>,
    /// Keys of every missed reminder (an item can have several), to mark delivered.
    pub missed_keys: Vec<String>,
}

impl Due {
    pub fn all_keys(&self) -> impl Iterator<Item = String> + '_ {
        self.fresh
            .iter()
            .map(|r| r.key.clone())
            .chain(self.missed_keys.iter().cloned())
    }
}

impl ReminderState {
    pub fn path(root: &Path) -> PathBuf {
        root.join(".pauta").join("reminders.json")
    }

    pub fn load(root: &Path) -> ReminderState {
        std::fs::read_to_string(Self::path(root))
            .ok()
            .and_then(|t| serde_json::from_str(&t).ok())
            .unwrap_or_default()
    }

    pub fn save(&mut self, root: &Path, now: Timestamp) -> Result<()> {
        let cutoff = now - SignedDuration::from_hours(24 * 30);
        self.delivered.retain(|_, t| *t > cutoff);
        self.snoozed.retain(|_, t| *t > cutoff);
        let text = serde_json::to_string_pretty(self).unwrap_or_default();
        write_atomic(&Self::path(root), &text)
    }

    /// When the reminder should fire, taking a snooze into account. A snooze that
    /// ends inside quiet hours waits for them to end, like any other reminder.
    fn effective(&self, r: &Reminder, config: &Config) -> Timestamp {
        match self.snoozed.get(&r.key) {
            Some(until) if *until > r.fire_at => match &config.quiet_hours {
                Some(q) => q.defer(*until, &config.tz),
                None => *until,
            },
            _ => r.fire_at,
        }
    }

    /// Reminders due at `now` that were not delivered yet.
    pub fn due(&self, index: &Index, now: Timestamp) -> Due {
        let from = now - SignedDuration::from_hours(LOOKBACK_HOURS);
        let grace = SignedDuration::from_mins(GRACE_MINUTES);
        let mut due = Due::default();
        for r in reminders_between(index, from, now + SignedDuration::from_hours(48)) {
            let eff = self.effective(&r, &index.config);
            if eff > now || self.delivered.get(&r.key).is_some_and(|d| *d >= eff) {
                continue;
            }
            if eff + grace < now {
                due.missed.push(r);
            } else {
                due.fresh.push(r);
            }
        }
        due.missed_keys = due.missed.iter().map(|r| r.key.clone()).collect();
        // Several reminders of the same item can be missed at once; one is enough.
        let mut seen = std::collections::HashSet::new();
        due.missed.retain(|r| seen.insert((r.item.kind, r.item.id.clone())));
        due
    }

    pub fn mark_delivered(&mut self, keys: impl IntoIterator<Item = String>, now: Timestamp) {
        for k in keys {
            self.delivered.insert(k, now);
        }
    }

    pub fn snooze(&mut self, key: &str, until: Timestamp) {
        self.snoozed.insert(key.to_string(), until);
    }
}
