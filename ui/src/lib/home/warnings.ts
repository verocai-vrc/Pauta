// The Avisos list (section 5.3 of the Início spec): what needs attention, most urgent first.
import type { AgendaItem, Index, Today } from "../api";
import { addDays, relativeDay } from "../dates";
import { t, tn } from "../i18n";
import { problemShort } from "../readiness";

export interface Warning {
  key: string;
  tone: "danger" | "warn" | "quiet";
  text: string;
  aux: string | null;
  /** The card to open; null means the Hoje tab. */
  card: string | null;
}

/** The backend sends at most this many stale cards, so a full list means "at least". */
const STALE_CAP = 10;

/** A post this close with something missing is worth a line; further out there is time. */
const READY_WINDOW_DAYS = 3;

export function buildWarnings(today: Today, readiness: Index["readiness"], agenda: AgendaItem[], leadId: string | null): Warning[] {
  const out: Warning[] = [];

  if (today.invalid_files > 0) {
    out.push({ key: "invalid", tone: "danger", text: tn("home.warn_invalid", today.invalid_files), aux: null, card: null });
  }

  // Postar hoje already shows the pending posts of today, late or not.
  const shownAbove = new Set(today.posts.filter((p) => !p.done).map((p) => p.id));
  for (const i of today.overdue) {
    if (i.kind === "publication" && shownAbove.has(i.id)) continue;
    out.push({ key: `overdue:${i.kind}:${i.id}`, tone: "danger", text: i.title, aux: relativeDay(i.when.date, today.date), card: i.id });
  }

  const horizon = addDays(today.date, READY_WINDOW_DAYS);
  for (const i of agenda) {
    if (i.kind !== "publication" || i.done || i.id === leadId || i.when.date > horizon) continue;
    const first = readiness[i.id]?.[0];
    if (!first) continue;
    const text = t("home.warn_not_ready", { day: relativeDay(i.when.date, today.date), problem: problemShort(first) });
    out.push({ key: `ready:${i.id}`, tone: "warn", text, aux: null, card: i.id });
  }

  for (const q of today.empty_queues) {
    out.push({ key: `queue:${q.network}`, tone: "warn", text: t("home.warn_queue", { net: q.label, days: q.days }), aux: null, card: null });
  }

  if (today.stale.length > 0) {
    const n = today.stale.length;
    const text = n >= STALE_CAP ? t("home.warn_stale_many", { n }) : tn("home.warn_stale", n);
    out.push({ key: "stale", tone: "quiet", text, aux: null, card: null });
  }

  return out;
}
