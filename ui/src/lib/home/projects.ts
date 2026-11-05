// What each project board has to say on Início: when it was last touched, what changed last
// and what comes next (section 5.7 of the spec). Pure: the index and the day come in as arguments.
import type { Board, Card, Index } from "../api";
import { addDays, relativeDay, shortDate } from "../dates";
import { t } from "../i18n";

export interface ProjectLine {
  text: string;
  kind: "card" | "event";
  id: string;
}

export interface ProjectSummary {
  id: string;
  name: string;
  /** Local day of the latest card moment, null when no card has one. */
  updated: string | null;
  last: ProjectLine | null;
  next: ProjectLine | null;
  /** The board has no cards at all. */
  empty: boolean;
}

/**
 * The local day of a UTC instant. The only time-zone arithmetic in the interface (ADR 0009):
 * the backend sends instants, and "which day was that for the user" needs `config.timezone`.
 */
export function localDay(timestamp: string, timezone: string): string {
  const parts = new Intl.DateTimeFormat("en-CA", { timeZone: timezone, year: "numeric", month: "2-digit", day: "2-digit" }).formatToParts(
    new Date(timestamp),
  );
  const get = (type: string) => parts.find((p) => p.type === type)?.value ?? "";
  return `${get("year")}-${get("month")}-${get("day")}`;
}

export function updatedLabel(day: string, date: string): string {
  if (day === date) return t("home.proj_updated_today");
  if (day === addDays(date, -1)) return t("home.proj_updated_yesterday");
  return t("home.proj_updated_on", { date: shortDate(day) });
}

/** An RFC 3339 instant in milliseconds; a missing or unreadable value is absent. */
function instant(value: string | null | undefined): number | null {
  if (!value) return null;
  const ms = Date.parse(value);
  return Number.isNaN(ms) ? null : ms;
}

interface Stamped {
  card: Card;
  created: number | null;
  updated: number | null;
  since: number | null;
  /** The greatest of the three, null when the card has none. */
  moment: number | null;
}

function stamp(card: Card): Stamped {
  const created = instant(card.created);
  const updated = instant(card.updated);
  const since = instant(card.column_since);
  const known = [created, updated, since].filter((v): v is number => v !== null);
  return { card, created, updated, since, moment: known.length ? Math.max(...known) : null };
}

const byRank = (a: Card, b: Card) => (a.rank < b.rank ? -1 : a.rank > b.rank ? 1 : 0);

function lastLine(board: Board, cards: Stamped[]): ProjectLine | null {
  // The first one wins a tie, so the order of the index decides.
  let latest: Stamped | null = null;
  for (const s of cards) if (s.moment !== null && (!latest || s.moment > latest.moment!)) latest = s;
  if (!latest) return null;

  const { card, created, updated, since } = latest;
  const title = card.title;
  const column = board.columns.find((c) => c.id === card.column);
  let text: string;
  if (created !== null && created === updated) text = t("home.proj_created", { title });
  else if (since !== null && since === updated)
    text = column?.role === "done" ? t("home.proj_done", { title }) : t("home.proj_moved", { column: column?.name ?? card.column, title });
  else text = t("home.proj_edited", { title });
  return { text, kind: "card", id: card.id };
}

interface Dated {
  date: string;
  time: string | null;
  /** 0 for a card, 1 for an event: a card wins a tie. */
  rank: number;
  title: string;
  line: Pick<ProjectLine, "kind" | "id">;
}

function earlier(a: Dated, b: Dated): boolean {
  if (a.date !== b.date) return a.date < b.date;
  if (a.time !== b.time) return a.time !== null && (b.time === null || a.time < b.time);
  return a.rank < b.rank;
}

function nextLine(board: Board, cards: Card[], index: Index, date: string): ProjectLine | null {
  const doneColumns = new Set(board.columns.filter((c) => c.role === "done").map((c) => c.id));
  const dated: Dated[] = [];
  for (const c of cards) {
    if (c.due && !doneColumns.has(c.column)) dated.push({ date: c.due.date, time: c.due.time, rank: 0, title: c.title, line: { kind: "card", id: c.id } });
  }
  for (const e of index.events) {
    if (e.links.board === board.id && e.when.date >= date) dated.push({ date: e.when.date, time: e.when.time, rank: 1, title: e.title, line: { kind: "event", id: e.id } });
  }
  let nearest: Dated | null = null;
  for (const d of dated) if (!nearest || earlier(d, nearest)) nearest = d;
  if (nearest) return { ...nearest.line, text: t("home.proj_dated", { title: nearest.title, day: relativeDay(nearest.date, date) }) };

  // From the most advanced column to the least; the board lists them from first to last.
  const active = board.columns.filter((c) => c.role !== "inbox" && c.role !== "done").reverse();
  for (const column of active) {
    const first = cards.filter((c) => c.column === column.id).sort(byRank)[0];
    if (first) return { text: first.title, kind: "card", id: first.id };
  }
  const inbox = board.columns.find((c) => c.role === "inbox");
  const first = inbox && cards.filter((c) => c.column === inbox.id).sort(byRank)[0];
  return first ? { text: first.title, kind: "card", id: first.id } : null;
}

/** The boards that are not for content, the most recently touched first. */
export function projectSummaries(index: Index, date: string): ProjectSummary[] {
  const rows = index.boards
    .filter((b) => b.kind !== "content")
    .map((board) => {
      const cards = index.cards.filter((c) => c.board === board.id);
      const stamped = cards.map(stamp);
      const moment = stamped.reduce<number | null>((max, s) => (s.moment !== null && (max === null || s.moment > max) ? s.moment : max), null);
      const summary: ProjectSummary = {
        id: board.id,
        name: board.name,
        updated: moment === null ? null : localDay(new Date(moment).toISOString(), index.config.timezone),
        last: lastLine(board, stamped),
        next: nextLine(board, cards, index, date),
        empty: cards.length === 0,
      };
      // Boards with a moment first, then those whose cards carry none, then the empty ones.
      return { summary, moment, group: moment !== null ? 0 : cards.length ? 1 : 2 };
    });
  // Array.sort is stable, so equal rows keep the order of the boards.
  rows.sort((a, b) => a.group - b.group || (b.moment ?? 0) - (a.moment ?? 0));
  return rows.map((r) => r.summary);
}
