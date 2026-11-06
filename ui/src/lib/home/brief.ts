// What the Brief and Postar hoje say about the day. Pure: the widgets pass the data in.
import type { AgendaItem } from "../api";
import { hm, relativeDay } from "../dates";
import { t } from "../i18n";

/** By `today.time`: 05:00 to 11:59 is the morning, 12:00 to 17:59 the afternoon, the rest the night. */
export function greeting(time: string): string {
  const hour = Number(time.slice(0, 2));
  if (hour >= 5 && hour < 12) return t("home.greeting_morning");
  if (hour >= 12 && hour < 18) return t("home.greeting_afternoon");
  return t("home.greeting_evening");
}

/** Timed items first, by time; an all-day item goes after the timed ones of its day. */
function byMoment(a: AgendaItem, b: AgendaItem): number {
  if (a.when.date !== b.when.date) return a.when.date < b.when.date ? -1 : 1;
  if (!a.when.time !== !b.when.time) return a.when.time ? -1 : 1;
  if (a.when.time && b.when.time && a.when.time !== b.when.time) return a.when.time < b.when.time ? -1 : 1;
  return 0;
}

/** The first item that is not done and has not passed; an all-day item lasts until the end of its day. */
export function nextUp(items: AgendaItem[], date: string, time: string): AgendaItem | null {
  const ahead = items.filter((i) => {
    if (i.done || i.when.date < date) return false;
    return !(i.when.date === date && i.when.time && i.when.time < time);
  });
  return ahead.sort(byMoment)[0] ?? null;
}

export function nextUpText(item: AgendaItem, date: string, net: (id: string | null) => string): string {
  const time = hm(item.when);
  const day = relativeDay(item.when.date, date);
  if (item.kind === "publication" && time) {
    return item.when.date === date
      ? t("home.next_post_today", { time, net: net(item.network) })
      : t("home.next_post", { day, time, net: net(item.network) });
  }
  if (time) return t("home.next_item_time", { title: item.title, day, time });
  return t("home.next_item", { title: item.title, day });
}

/** The first pending post leads; the others follow, pending before published. */
export function splitPosts(posts: AgendaItem[]): { lead: AgendaItem | null; rest: AgendaItem[] } {
  const pending = posts.filter((p) => !p.done);
  const published = posts.filter((p) => p.done);
  return { lead: pending[0] ?? null, rest: [...pending.slice(1), ...published] };
}

export function isLate(post: AgendaItem, date: string, time: string): boolean {
  if (post.done) return false;
  if (post.when.date !== date) return post.when.date < date;
  return !!post.when.time && post.when.time < time;
}

/** The next post still to publish after today, for a day with none. */
export function nextScheduled(items: AgendaItem[], date: string): AgendaItem | null {
  return items.filter((i) => i.kind === "publication" && !i.done && i.when.date > date).sort(byMoment)[0] ?? null;
}
