//! Cadence goals and publication history (section 10).
//!
//! Everything here is a *view* over the files: counts come from the publication
//! records of cards (including archived ones, so archiving never rewrites
//! history). No streaks and no scores: it shows the rhythm, nothing more.

use std::collections::BTreeMap;

use jiff::civil::Date;
use jiff::tz::TimeZone;
use jiff::{Timestamp, ToSpan};
use serde::Serialize;

use crate::config::{CadenceGoal, Period};
use crate::model::{Card, Metrics, PubStatus};
use crate::store::Index;

/// A post that was published, flattened from its card.
#[derive(Debug, Clone, Serialize)]
pub struct PublishedPost {
    pub id: String,
    pub title: String,
    pub network: String,
    /// Local date it went out.
    pub date: Date,
    pub published_at: Option<Timestamp>,
    pub url: Option<String>,
    pub archived: bool,
    /// Hour of day (local) the post was planned for, else when it actually went out.
    pub hour: Option<u8>,
    pub metrics: Option<Metrics>,
}

impl PublishedPost {
    pub fn from_card(c: &Card, tz: &TimeZone, archived: bool) -> Option<PublishedPost> {
        let p = c.publication.as_ref().filter(|p| p.status == PubStatus::Published)?;
        let network = c.network.clone()?;
        let date = p
            .published_at
            .map(|t| t.to_zoned(tz.clone()).date())
            .or_else(|| p.at.as_ref().map(|w| w.date))
            .or_else(|| c.updated.map(|t| t.to_zoned(tz.clone()).date()))?;
        Some(PublishedPost {
            id: c.id.clone(),
            title: c.title.clone(),
            network,
            date,
            published_at: p.published_at,
            url: p.url.clone(),
            archived,
            hour: p
                .at
                .as_ref()
                .and_then(|w| w.time)
                .map(|t| t.hour() as u8)
                .or_else(|| p.published_at.map(|t| t.to_zoned(tz.clone()).hour() as u8)),
            metrics: c.metrics.clone(),
        })
    }
}

/// Monday to Sunday containing `d`.
pub fn week_bounds(d: Date) -> (Date, Date) {
    let start = d
        .checked_sub(i64::from(d.weekday().to_monday_zero_offset()).days())
        .unwrap_or(d);
    (start, start.checked_add(6.days()).unwrap_or(start))
}

pub fn month_bounds(d: Date) -> (Date, Date) {
    (d.first_of_month(), d.last_of_month())
}

fn bounds(d: Date, per: Period) -> (Date, Date) {
    match per {
        Period::Week => week_bounds(d),
        Period::Month => month_bounds(d),
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct CadenceProgress {
    pub network: String,
    pub label: String,
    pub goal: u32,
    pub per: Period,
    /// Published in the current period.
    pub done: u32,
    /// Still ahead in the current period: planned or scheduled, not yet published.
    pub scheduled: u32,
    pub period_start: Date,
    pub period_end: Date,
}

/// Progress towards each network's goal in the week or month containing `now`.
pub fn cadence(index: &Index, now: Timestamp) -> Vec<CadenceProgress> {
    let tz = &index.config.tz;
    let today = now.to_zoned(tz.clone()).date();
    index
        .config
        .networks
        .iter()
        .filter_map(|n| {
            let CadenceGoal { posts, per } = n.cadence_goal?;
            let (start, end) = bounds(today, per);
            let in_period = |d: Date| d >= start && d <= end;
            let done = index
                .published
                .iter()
                .filter(|p| p.network == n.id && in_period(p.date))
                .count() as u32;
            let scheduled = index
                .cards
                .iter()
                .filter(|c| c.network.as_deref() == Some(n.id.as_str()))
                .filter(|c| c.pending_publication_at().is_some_and(|w| in_period(w.date)))
                .count() as u32;
            Some(CadenceProgress {
                network: n.id.clone(),
                label: n.label.clone(),
                goal: posts,
                per,
                done,
                scheduled,
                period_start: start,
                period_end: end,
            })
        })
        .collect()
}

#[derive(Debug, Clone, Serialize)]
pub struct Bucket {
    pub start: Date,
    pub end: Date,
    /// Posts per network id (every configured network is present, even at 0).
    pub counts: BTreeMap<String, u32>,
    pub total: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct History {
    /// Oldest first, ending with the current week.
    pub weeks: Vec<Bucket>,
    /// Oldest first, ending with the current month.
    pub months: Vec<Bucket>,
    pub total_posts: u32,
}

fn bucket(index: &Index, start: Date, end: Date) -> Bucket {
    let mut counts: BTreeMap<String, u32> = index.config.networks.iter().map(|n| (n.id.clone(), 0)).collect();
    let mut total = 0;
    for p in index.published.iter().filter(|p| p.date >= start && p.date <= end) {
        *counts.entry(p.network.clone()).or_default() += 1;
        total += 1;
    }
    Bucket {
        start,
        end,
        counts,
        total,
    }
}

pub fn history(index: &Index, now: Timestamp, weeks: usize, months: usize) -> History {
    let today = now.to_zoned(index.config.tz.clone()).date();
    let (this_week, _) = week_bounds(today);
    let weeks_v = (0..weeks)
        .rev()
        .filter_map(|i| this_week.checked_sub((7 * i as i64).days()).ok())
        .map(|start| bucket(index, start, start.checked_add(6.days()).unwrap_or(start)))
        .collect();
    let months_v = (0..months)
        .rev()
        .filter_map(|i| today.first_of_month().checked_sub((i as i64).months()).ok())
        .map(|start| bucket(index, start, start.last_of_month()))
        .collect();
    History {
        weeks: weeks_v,
        months: months_v,
        total_posts: index.published.len() as u32,
    }
}

/// With fewer posts than this, a weekday average says little.
pub const MIN_SAMPLES: u32 = 3;

#[derive(Debug, Clone, Serialize)]
pub struct WeekdayStat {
    /// 0 = Monday ... 6 = Sunday.
    pub weekday: u8,
    pub posts: u32,
    pub avg_engagement: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct NetworkInsight {
    pub network: String,
    pub label: String,
    /// Published posts with numbers recorded.
    pub posts: u32,
    pub impressions: u64,
    pub engagement: u64,
    pub avg_engagement: f64,
    /// Engagement divided by impressions, over the posts that have both.
    pub engagement_rate: Option<f64>,
    pub by_weekday: Vec<WeekdayStat>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TopPost {
    pub id: String,
    pub title: String,
    pub network: String,
    pub date: Date,
    pub engagement: u64,
    pub impressions: Option<u64>,
}

/// What the author's own numbers say (section 9.1: the author's results count for
/// more than generic rules). Only posts with recorded metrics take part.
#[derive(Debug, Clone, Serialize)]
pub struct Insights {
    pub posts_with_metrics: u32,
    pub min_samples: u32,
    pub networks: Vec<NetworkInsight>,
    pub top: Vec<TopPost>,
}

pub fn insights(index: &Index) -> Insights {
    let with: Vec<(&PublishedPost, &Metrics)> = index
        .published
        .iter()
        .filter_map(|p| p.metrics.as_ref().map(|m| (p, m)))
        .collect();
    let networks = index
        .config
        .networks
        .iter()
        .filter_map(|n| {
            let mine: Vec<_> = with.iter().filter(|(p, _)| p.network == n.id).collect();
            if mine.is_empty() {
                return None;
            }
            let engagement: u64 = mine.iter().map(|(_, m)| m.engagement()).sum();
            let impressions: u64 = mine.iter().filter_map(|(_, m)| m.impressions).sum();
            let rated: Vec<_> = mine
                .iter()
                .filter(|(_, m)| m.impressions.is_some_and(|i| i > 0))
                .collect();
            let rate = (!rated.is_empty()).then(|| {
                let e: u64 = rated.iter().map(|(_, m)| m.engagement()).sum();
                let i: u64 = rated.iter().filter_map(|(_, m)| m.impressions).sum();
                e as f64 / i as f64
            });
            let by_weekday = (0..7u8)
                .filter_map(|wd| {
                    let day: Vec<_> = mine
                        .iter()
                        .filter(|(p, _)| p.date.weekday().to_monday_zero_offset() as u8 == wd)
                        .collect();
                    (!day.is_empty()).then(|| WeekdayStat {
                        weekday: wd,
                        posts: day.len() as u32,
                        avg_engagement: day.iter().map(|(_, m)| m.engagement()).sum::<u64>() as f64 / day.len() as f64,
                    })
                })
                .collect();
            Some(NetworkInsight {
                network: n.id.clone(),
                label: n.label.clone(),
                posts: mine.len() as u32,
                impressions,
                engagement,
                avg_engagement: engagement as f64 / mine.len() as f64,
                engagement_rate: rate,
                by_weekday,
            })
        })
        .collect();
    let mut top: Vec<TopPost> = with
        .iter()
        .map(|(p, m)| TopPost {
            id: p.id.clone(),
            title: p.title.clone(),
            network: p.network.clone(),
            date: p.date,
            engagement: m.engagement(),
            impressions: m.impressions,
        })
        .collect();
    top.sort_by(|a, b| b.engagement.cmp(&a.engagement).then_with(|| b.date.cmp(&a.date)));
    top.truncate(5);
    Insights {
        posts_with_metrics: with.len() as u32,
        min_samples: MIN_SAMPLES,
        networks,
        top,
    }
}

/// Everything the Rhythm screen shows.
#[derive(Debug, Clone, Serialize)]
pub struct Rhythm {
    pub cadence: Vec<CadenceProgress>,
    pub history: History,
    pub insights: Insights,
    /// Latest first.
    pub recent: Vec<PublishedPost>,
}

pub fn rhythm(index: &Index, now: Timestamp) -> Rhythm {
    let mut recent = index.published.clone();
    recent.sort_by(|a, b| b.date.cmp(&a.date).then_with(|| b.published_at.cmp(&a.published_at)));
    recent.truncate(20);
    Rhythm {
        cadence: cadence(index, now),
        history: history(index, now, 12, 6),
        insights: insights(index),
        recent,
    }
}
