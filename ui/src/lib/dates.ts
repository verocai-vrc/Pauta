// Calendar math on plain YYYY-MM-DD strings. All time-zone work happens in
// pauta-core; the UI only shows the local dates and times it receives.
import type { When } from "./api";

export function parseDate(d: string): Date {
  const [y, m, day] = d.split("-").map(Number);
  return new Date(Date.UTC(y, m - 1, day));
}

export function fmtDate(d: Date): string {
  return d.toISOString().slice(0, 10);
}

export function addDays(d: string, n: number): string {
  const x = parseDate(d);
  x.setUTCDate(x.getUTCDate() + n);
  return fmtDate(x);
}

/** 0 = Monday ... 6 = Sunday. */
export function weekday(d: string): number {
  return (parseDate(d).getUTCDay() + 6) % 7;
}

export function startOfWeek(d: string): string {
  return addDays(d, -weekday(d));
}

export function startOfMonth(d: string): string {
  return d.slice(0, 8) + "01";
}

export function addMonths(d: string, n: number): string {
  const x = parseDate(startOfMonth(d));
  x.setUTCMonth(x.getUTCMonth() + n);
  return fmtDate(x);
}

/** 6 weeks of dates starting on the Monday on or before the 1st. */
export function monthGrid(month: string): string[] {
  const start = startOfWeek(startOfMonth(month));
  return Array.from({ length: 42 }, (_, i) => addDays(start, i));
}

export function weekDays(d: string): string[] {
  const start = startOfWeek(d);
  return Array.from({ length: 7 }, (_, i) => addDays(start, i));
}

export function hm(when: When | null | undefined): string {
  return when?.time ? when.time.slice(0, 5) : "";
}

/** Value for <input type="datetime-local">. */
export function localInput(when: When | null | undefined): string {
  if (!when) return "";
  return when.time ? `${when.date}T${when.time.slice(0, 5)}` : when.date;
}

const MONTHS = ["jan", "fev", "mar", "abr", "mai", "jun", "jul", "ago", "set", "out", "nov", "dez"];
const MONTHS_LONG = [
  "janeiro", "fevereiro", "março", "abril", "maio", "junho",
  "julho", "agosto", "setembro", "outubro", "novembro", "dezembro",
];
const WEEKDAYS_LONG = [
  "segunda-feira", "terça-feira", "quarta-feira", "quinta-feira", "sexta-feira", "sábado", "domingo",
];
export const WEEKDAYS_SHORT = ["seg", "ter", "qua", "qui", "sex", "sáb", "dom"];

export function shortDate(d: string): string {
  const [, m, day] = d.split("-");
  return `${Number(day)} ${MONTHS[Number(m) - 1]}`;
}

export function monthTitle(d: string): string {
  const [y, m] = d.split("-");
  const name = MONTHS_LONG[Number(m) - 1];
  return `${name[0].toUpperCase()}${name.slice(1)} de ${y}`;
}

/** "Quarta-feira, 28 de outubro". */
export function longDate(d: string): string {
  const name = WEEKDAYS_LONG[weekday(d)];
  const [, m, day] = d.split("-");
  // The first of the month is an ordinal in Brazilian Portuguese: "1º de novembro".
  const n = Number(day) === 1 ? "1º" : String(Number(day));
  return `${name[0].toUpperCase()}${name.slice(1)}, ${n} de ${MONTHS_LONG[Number(m) - 1]}`;
}

/** "hoje", "amanhã", "ontem" or "qua, 4 nov". */
export function relativeDay(d: string, today: string): string {
  if (d === today) return "hoje";
  if (d === addDays(today, 1)) return "amanhã";
  if (d === addDays(today, -1)) return "ontem";
  return `${WEEKDAYS_SHORT[weekday(d)]}, ${shortDate(d)}`;
}

export function whenLabel(w: When, today: string): string {
  const t = hm(w);
  return t ? `${relativeDay(w.date, today)} ${t}` : relativeDay(w.date, today);
}

/** Moves a dated value to another day, keeping its time of day. */
export function moveToDay(w: When, day: string): string {
  return w.time ? `${day}T${w.time.slice(0, 5)}` : day;
}
