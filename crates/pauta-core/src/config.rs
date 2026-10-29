//! `pauta.yaml`: general configuration (section 5.2).

use jiff::civil::{Time, Weekday};
use jiff::tz::TimeZone;
use serde::{Deserialize, Serialize};
use serde_yaml_ng::{Mapping, Value};

use crate::error::{Error, Result, rule};
use crate::fields::*;
use crate::time::{QuietHours, fmt_hm, parse_time, zone};
use crate::yaml::Y;

pub const SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize)]
pub struct Config {
    pub schema_version: u32,
    pub timezone: String,
    #[serde(skip)]
    pub tz: TimeZone,
    pub language: String,
    pub board_order: Vec<String>,
    pub networks: Vec<NetworkConfig>,
    pub reminders: ReminderDefaults,
    pub quiet_hours: Option<QuietHours>,
    pub empty_queue_days: i64,
    pub stale_after_days: i64,
    pub auto_archive_days: Option<i64>,
    pub art: ArtConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NetworkConfig {
    pub id: String,
    pub label: String,
    pub url: Option<String>,
    pub char_limit: Option<u32>,
    pub slots: Vec<Slot>,
    /// Posts wanted per week or month (section 10). A hint, never a rule.
    #[serde(default)]
    pub cadence_goal: Option<CadenceGoal>,
    /// Visual formats this network takes, and whether a post needs media at all.
    #[serde(default)]
    pub media: MediaSpec,
}

/// What a network accepts as artwork. Sizes change and sources disagree, so they are
/// configuration, not code: check them against the platform's own help pages.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct MediaSpec {
    /// The network is image-first: a post without a file is not ready to go out.
    pub requires_media: bool,
    pub formats: Vec<MediaFormat>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MediaFormat {
    pub id: String,
    pub label: String,
    /// `WIDTHxHEIGHT` in pixels, e.g. `1080x1350`.
    pub size: String,
    /// Delivery hint for the brief, e.g. "entregar como PDF".
    #[serde(default)]
    pub note: Option<String>,
}

/// Settings for ordering artwork (see docs/adr/0007-arte-com-claude-design.md).
#[derive(Debug, Clone, Serialize, Default, PartialEq, Eq)]
pub struct ArtConfig {
    /// Where "open Claude Design" goes. There is no API: this is a plain web link.
    pub tool_url: String,
    /// The author's visual identity, appended to every brief.
    pub brand: Option<String>,
}

impl MediaSpec {
    /// Built-in starting points, used until `pauta.yaml` says otherwise.
    pub fn default_for(network: &str) -> MediaSpec {
        let f = |id: &str, label: &str, size: &str, note: Option<&str>| MediaFormat {
            id: id.into(),
            label: label.into(),
            size: size.into(),
            note: note.map(str::to_string),
        };
        match network {
            "linkedin" => MediaSpec {
                requires_media: false,
                formats: vec![
                    f("image", "Imagem única", "1200x627", None),
                    f(
                        "carousel",
                        "Carrossel (documento PDF)",
                        "1080x1350",
                        Some("entregar como um único PDF, uma página por slide"),
                    ),
                ],
            },
            "instagram" => MediaSpec {
                requires_media: true,
                formats: vec![
                    f("image", "Imagem no feed (4:5)", "1080x1350", None),
                    f(
                        "carousel",
                        "Carrossel (4:5)",
                        "1080x1350",
                        Some("todos os slides com a mesma proporção; um arquivo por slide"),
                    ),
                    f("story", "Story ou Reel (9:16)", "1080x1920", None),
                ],
            },
            _ => MediaSpec {
                requires_media: false,
                formats: vec![f("image", "Imagem", "1080x1080", None)],
            },
        }
    }

    fn from_value(network: &str, v: Option<&Value>) -> std::result::Result<MediaSpec, String> {
        let default = MediaSpec::default_for(network);
        let Some(m) = v.and_then(Value::as_mapping) else {
            return Ok(default);
        };
        let mut formats = Vec::new();
        if let Some(Value::Sequence(items)) = m.get("formats") {
            for item in items.iter().filter_map(Value::as_mapping) {
                let id = get_str(item, "id").ok_or_else(|| format!("networks.{network}.media.formats: item sem id"))?;
                let size = get_str(item, "size").unwrap_or_default();
                check_size(&size).map_err(|e| format!("networks.{network}.media.formats.{id}: {e}"))?;
                formats.push(MediaFormat {
                    label: get_str(item, "label").unwrap_or_else(|| id.clone()),
                    note: get_str(item, "note"),
                    id,
                    size,
                });
            }
        }
        Ok(MediaSpec {
            requires_media: get_str(m, "requires_media").map_or(default.requires_media, |v| v == "true"),
            formats: if formats.is_empty() { default.formats } else { formats },
        })
    }
}

/// `1080x1350`: two positive integers.
pub fn check_size(s: &str) -> std::result::Result<(u32, u32), String> {
    let bad = || format!("tamanho inválido '{s}': use LARGURAxALTURA, por exemplo 1080x1350");
    let (w, h) = s.split_once('x').ok_or_else(bad)?;
    match (w.parse::<u32>(), h.parse::<u32>()) {
        (Ok(w), Ok(h)) if (1..=20000).contains(&w) && (1..=20000).contains(&h) => Ok((w, h)),
        _ => Err(bad()),
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct CadenceGoal {
    pub posts: u32,
    pub per: Period,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Period {
    Week,
    Month,
}

impl CadenceGoal {
    fn from_value(network: &str, v: Option<&Value>) -> std::result::Result<Option<CadenceGoal>, String> {
        let Some(m) = v.and_then(Value::as_mapping) else {
            return Ok(None);
        };
        let key = format!("networks.{network}.cadence_goal");
        let posts = get_i64(m, "posts").ok_or_else(|| format!("{key}.posts ausente"))?;
        let posts = u32::try_from(posts)
            .ok()
            .filter(|p| (1..=100).contains(p))
            .ok_or_else(|| format!("{key}.posts deve estar entre 1 e 100 (recebido: {posts})"))?;
        let per = match get_str(m, "per").as_deref() {
            Some("week") | None => Period::Week,
            Some("month") => Period::Month,
            Some(other) => return Err(format!("{key}.per deve ser week ou month (recebido: {other})")),
        };
        Ok(Some(CadenceGoal { posts, per }))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Slot {
    /// `mon`..`sun`.
    pub days: Vec<String>,
    /// `HH:MM`.
    pub time: String,
}

impl Slot {
    pub fn weekdays(&self) -> Vec<Weekday> {
        self.days.iter().filter_map(|d| parse_weekday(d)).collect()
    }

    pub fn time_of_day(&self) -> Option<Time> {
        parse_time(&self.time).ok()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReminderDefaults {
    pub post_default: Vec<String>,
    pub deadline_default: Vec<String>,
}

pub fn parse_weekday(s: &str) -> Option<Weekday> {
    Some(match s.trim().to_lowercase().as_str() {
        "mon" | "seg" => Weekday::Monday,
        "tue" | "ter" => Weekday::Tuesday,
        "wed" | "qua" => Weekday::Wednesday,
        "thu" | "qui" => Weekday::Thursday,
        "fri" | "sex" => Weekday::Friday,
        "sat" | "sab" | "sáb" => Weekday::Saturday,
        "sun" | "dom" => Weekday::Sunday,
        _ => return None,
    })
}

impl Config {
    pub fn network(&self, id: &str) -> Option<&NetworkConfig> {
        self.networks.iter().find(|n| n.id == id)
    }

    pub fn from_mapping(m: &Mapping) -> std::result::Result<Config, String> {
        let timezone = get_str(m, "timezone").unwrap_or_else(|| "America/Recife".into());
        let tz = zone(&timezone).map_err(|e| e.to_string())?;
        let mut networks = Vec::new();
        if let Some(Value::Mapping(nets)) = m.get("networks") {
            for (k, v) in nets {
                let id = crate::yaml::scalar_to_string(k);
                let empty = Mapping::new();
                let n = v.as_mapping().unwrap_or(&empty);
                let slots = match n.get("slots") {
                    Some(Value::Sequence(items)) => items
                        .iter()
                        .filter_map(Value::as_mapping)
                        .map(|s| Slot {
                            days: get_list(s, "days"),
                            time: get_str(s, "time").unwrap_or_else(|| "09:00".into()),
                        })
                        .collect(),
                    _ => Vec::new(),
                };
                networks.push(NetworkConfig {
                    label: get_str(n, "label").unwrap_or_else(|| default_label(&id)),
                    url: get_str(n, "url"),
                    char_limit: get_i64(n, "char_limit").map(|v| v as u32),
                    slots,
                    cadence_goal: CadenceGoal::from_value(&id, n.get("cadence_goal"))?,
                    media: MediaSpec::from_value(&id, n.get("media"))?,
                    id,
                });
            }
        }
        let empty = Mapping::new();
        let reminders = m.get("reminders").and_then(Value::as_mapping).unwrap_or(&empty);
        let notifications = m.get("notifications").and_then(Value::as_mapping).unwrap_or(&empty);
        let quiet_hours = notifications
            .get("quiet_hours")
            .and_then(Value::as_mapping)
            .and_then(|q| {
                Some(QuietHours {
                    from: parse_time(&get_str(q, "from")?).ok()?,
                    to: parse_time(&get_str(q, "to")?).ok()?,
                })
            });
        let rhythm = m.get("rhythm").and_then(Value::as_mapping).unwrap_or(&empty);
        let archive = m.get("archive").and_then(Value::as_mapping).unwrap_or(&empty);
        Ok(Config {
            schema_version: get_i64(m, "schema_version").unwrap_or(1) as u32,
            tz,
            timezone,
            language: get_str(m, "language").unwrap_or_else(|| "pt-BR".into()),
            board_order: get_list(m, "board_order"),
            networks,
            reminders: ReminderDefaults {
                post_default: get_list_or(reminders, "post_default", &["-1h", "0"]),
                deadline_default: get_list_or(reminders, "deadline_default", &["-7d", "-1d", "09:00"]),
            },
            quiet_hours,
            empty_queue_days: match get_i64(rhythm, "empty_queue_days") {
                Some(d) => check_days("rhythm.empty_queue_days", d)?,
                None => 7,
            },
            stale_after_days: match get_i64(rhythm, "stale_after_days") {
                Some(d) => check_days("rhythm.stale_after_days", d)?,
                None => 14,
            },
            auto_archive_days: match get_i64(archive, "auto_after_days") {
                Some(d) => Some(check_days("archive.auto_after_days", d)?),
                None => None,
            },
            art: {
                let a = m.get("art").and_then(Value::as_mapping).unwrap_or(&empty);
                let url = get_str(a, "tool_url").unwrap_or_else(|| DEFAULT_ART_URL.into());
                check_url("art.tool_url", &url)?;
                ArtConfig {
                    tool_url: url,
                    brand: get_str(a, "brand")
                        .map(|b| b.trim().to_string())
                        .filter(|b| !b.is_empty()),
                }
            },
        })
    }
}

pub const DEFAULT_ART_URL: &str = "https://claude.ai";

pub fn check_url(key: &str, url: &str) -> std::result::Result<(), String> {
    if url.starts_with("https://") || url.starts_with("http://") {
        Ok(())
    } else {
        Err(format!("{key} deve começar com http:// ou https:// (recebido: {url})"))
    }
}

/// Longest accepted span for day counts (ten years). A zero or negative count would
/// make "older than N days" match everything (auto-archive would sweep every done
/// card), and a huge one overflows the date arithmetic.
pub const MAX_DAYS: i64 = 3650;

pub fn check_days(key: &str, days: i64) -> std::result::Result<i64, String> {
    if (1..=MAX_DAYS).contains(&days) {
        Ok(days)
    } else {
        Err(format!("{key} deve estar entre 1 e {MAX_DAYS} dias (recebido: {days})"))
    }
}

fn default_label(id: &str) -> String {
    match id {
        "linkedin" => "LinkedIn".into(),
        "instagram" => "Instagram".into(),
        other => {
            let mut c = other.chars();
            c.next()
                .map(|f| f.to_uppercase().collect::<String>() + c.as_str())
                .unwrap_or_default()
        }
    }
}

/// Changes from the settings screen. Absent fields are left untouched.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct ConfigPatch {
    pub timezone: Option<String>,
    pub networks: Option<Vec<NetworkConfig>>,
    pub reminders: Option<ReminderDefaults>,
    #[serde(default, with = "crate::fields::double_option")]
    pub quiet_hours: Option<Option<QuietHoursInput>>,
    pub empty_queue_days: Option<i64>,
    pub stale_after_days: Option<i64>,
    #[serde(default, with = "crate::fields::double_option")]
    pub auto_archive_days: Option<Option<i64>>,
    pub art: Option<ArtPatch>,
}

/// Empty strings clear the value (the link goes back to the default).
#[derive(Debug, Clone, Deserialize)]
pub struct ArtPatch {
    pub tool_url: Option<String>,
    pub brand: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct QuietHoursInput {
    pub from: String,
    pub to: String,
}

/// Applies a patch to the parsed file, returning the `(key, value)` blocks to rewrite.
/// Unknown keys inside the touched blocks (e.g. `cadence_goal`) are preserved.
pub(crate) fn patch_blocks(current: &Mapping, patch: &ConfigPatch) -> Result<Vec<(String, Y)>> {
    let mut out = Vec::new();
    if let Some(tz) = &patch.timezone {
        zone(tz)?;
        out.push(("timezone".into(), Y::str(tz)));
    }
    if let Some(nets) = &patch.networks {
        let existing = current.get("networks").and_then(Value::as_mapping);
        let mut entries = Vec::new();
        for n in nets {
            if n.id.trim().is_empty() {
                return rule("rede sem identificador");
            }
            for s in &n.slots {
                parse_time(&s.time)?;
                if let Some(bad) = s.days.iter().find(|d| parse_weekday(d).is_none()) {
                    return rule(format!("dia inválido '{bad}': use mon, tue, wed, thu, fri, sat, sun"));
                }
            }
            let slots = Y::Seq(
                n.slots
                    .iter()
                    .map(|s| {
                        Y::Map(vec![
                            ("days".into(), Y::str_list(&s.days)),
                            ("time".into(), Y::Quoted(s.time.clone())),
                        ])
                    })
                    .collect(),
            );
            let updates = vec![
                ("label".into(), Y::Quoted(n.label.clone())),
                ("url".into(), n.url.clone().map_or(Y::Null, Y::Quoted)),
                ("char_limit".into(), n.char_limit.map_or(Y::Null, |c| Y::Int(c.into()))),
                ("slots".into(), slots),
                ("media".into(), media_y(&n.id, &n.media)?),
                (
                    "cadence_goal".into(),
                    match n.cadence_goal {
                        Some(g) => {
                            if !(1..=100).contains(&g.posts) {
                                return rule(format!("meta de cadência de '{}' deve estar entre 1 e 100", n.id));
                            }
                            Y::Map(vec![
                                ("posts".into(), Y::Int(g.posts.into())),
                                (
                                    "per".into(),
                                    Y::str(match g.per {
                                        Period::Week => "week",
                                        Period::Month => "month",
                                    }),
                                ),
                            ])
                        }
                        None => Y::Null,
                    },
                ),
            ];
            let prev = existing.and_then(|m| m.get(n.id.as_str()));
            entries.push((n.id.clone(), merge_map(prev, updates)));
        }
        out.push(("networks".into(), Y::Map(entries)));
    }
    if let Some(r) = &patch.reminders {
        for s in r.post_default.iter().chain(&r.deadline_default) {
            crate::time::ReminderSpec::parse(s)?;
        }
        out.push((
            "reminders".into(),
            merge_map(
                current.get("reminders"),
                vec![
                    ("post_default".into(), Y::str_list(&r.post_default)),
                    ("deadline_default".into(), Y::str_list(&r.deadline_default)),
                ],
            ),
        ));
    }
    if let Some(q) = &patch.quiet_hours {
        let value = match q {
            Some(q) => Y::Map(vec![
                ("from".into(), Y::Quoted(fmt_hm(parse_time(&q.from)?))),
                ("to".into(), Y::Quoted(fmt_hm(parse_time(&q.to)?))),
            ]),
            None => Y::Null,
        };
        out.push((
            "notifications".into(),
            merge_map(current.get("notifications"), vec![("quiet_hours".into(), value)]),
        ));
    }
    if patch.empty_queue_days.is_some() || patch.stale_after_days.is_some() {
        let mut updates = Vec::new();
        if let Some(d) = patch.empty_queue_days {
            updates.push((
                "empty_queue_days".into(),
                Y::Int(check_days("empty_queue_days", d).map_err(Error::Rule)?),
            ));
        }
        if let Some(d) = patch.stale_after_days {
            updates.push((
                "stale_after_days".into(),
                Y::Int(check_days("stale_after_days", d).map_err(Error::Rule)?),
            ));
        }
        out.push(("rhythm".into(), merge_map(current.get("rhythm"), updates)));
    }
    if let Some(a) = &patch.auto_archive_days {
        let value = match a {
            Some(d) => Y::Int(check_days("auto_archive_days", *d).map_err(Error::Rule)?),
            None => Y::Null,
        };
        out.push((
            "archive".into(),
            merge_map(current.get("archive"), vec![("auto_after_days".into(), value)]),
        ));
    }
    if let Some(a) = &patch.art {
        let mut updates = Vec::new();
        if let Some(url) = &a.tool_url {
            let url = url.trim();
            if url.is_empty() {
                updates.push(("tool_url".to_string(), Y::Null));
            } else {
                check_url("art.tool_url", url).map_err(Error::Rule)?;
                updates.push(("tool_url".to_string(), Y::Quoted(url.to_string())));
            }
        }
        if let Some(brand) = &a.brand {
            let brand = brand.trim();
            // Multi-line notes are written as a quoted scalar with \n escapes.
            updates.push((
                "brand".to_string(),
                if brand.is_empty() {
                    Y::Null
                } else {
                    Y::Quoted(brand.to_string())
                },
            ));
        }
        out.push(("art".into(), merge_map(current.get("art"), updates)));
    }
    Ok(out)
}

fn media_y(network: &str, m: &MediaSpec) -> Result<Y> {
    let mut seen = std::collections::HashSet::new();
    let mut formats = Vec::new();
    for f in &m.formats {
        if f.id.trim().is_empty() || !seen.insert(f.id.as_str()) {
            return rule(format!("formato de mídia vazio ou repetido em '{network}': '{}'", f.id));
        }
        check_size(&f.size).map_err(|e| Error::Rule(format!("{network}/{}: {e}", f.id)))?;
        let mut entries = vec![
            ("id".to_string(), Y::str(&f.id)),
            ("label".to_string(), Y::Quoted(f.label.clone())),
            ("size".to_string(), Y::Quoted(f.size.clone())),
        ];
        if let Some(n) = f.note.as_ref().filter(|n| !n.trim().is_empty()) {
            entries.push(("note".to_string(), Y::Quoted(n.clone())));
        }
        formats.push(Y::Map(entries));
    }
    Ok(Y::Map(vec![
        ("requires_media".into(), Y::Bool(m.requires_media)),
        ("formats".into(), Y::Seq(formats)),
    ]))
}

/// The `pauta.yaml` written on first run.
pub fn default_config_text(timezone: &str, board_order: &[String]) -> String {
    format!(
        r#"schema_version: {SCHEMA_VERSION}
timezone: {timezone}
language: pt-BR
board_order: [{order}]
networks:
  linkedin:
    label: "LinkedIn"
    url: "https://www.linkedin.com/feed/"
    char_limit: 3000             # conferir: o limite da plataforma pode mudar
    slots:                       # faixas de postagem (seção 9.1)
      - {{ days: [tue, thu], time: "09:00" }}
    cadence_goal: {{ posts: 2, per: week }}
  instagram:
    label: "Instagram"
    url: "https://www.instagram.com/"
    char_limit: 2200             # conferir: o limite da plataforma pode mudar
    slots:
      - {{ days: [wed, sat], time: "19:00" }}
    cadence_goal: {{ posts: 2, per: week }}
reminders:
  post_default: ["-1h", "0"]     # antecedência e hora exata
  deadline_default: ["-7d", "-1d", "09:00"]
notifications:
  quiet_hours: {{ from: "23:00", to: "07:00" }}
rhythm:
  empty_queue_days: 7            # aviso de fila vazia (seção 10)
  stale_after_days: 14           # cartões parados
"#,
        order = board_order.join(", ")
    )
}
