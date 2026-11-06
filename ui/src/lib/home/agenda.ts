// The Agenda widget (section 5.4 of the Início spec): what comes in the next days, by day.
import type { AgendaItem } from "../api";
import { relativeDay } from "../dates";

export interface AgendaDay {
  date: string;
  label: string;
  today: boolean;
  items: AgendaItem[];
}

/** Without finished items and without today's posts, which Postar hoje already shows. */
export function homeAgenda(items: AgendaItem[], date: string): AgendaItem[] {
  return items.filter((i) => !i.done && !(i.kind === "publication" && i.when.date === date));
}

export function groupByDay(items: AgendaItem[], date: string): AgendaDay[] {
  const days = new Map<string, AgendaItem[]>();
  for (const i of items) {
    // An event that began before today is still going on today.
    const day = i.when.date < date ? date : i.when.date;
    const list = days.get(day);
    if (list) list.push(i);
    else days.set(day, [i]);
  }
  return [...days]
    .sort(([a], [b]) => (a < b ? -1 : a > b ? 1 : 0))
    .map(([day, list]) => {
      const label = relativeDay(day, date);
      return { date: day, label: label[0].toUpperCase() + label.slice(1), today: day === date, items: list };
    });
}
