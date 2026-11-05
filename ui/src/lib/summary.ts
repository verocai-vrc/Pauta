// The sentence under "Hoje", built from the counts that `today` already carries.
import type { Today } from "./api";
import { t } from "./i18n";

/** One to ten in words, the rest as digits: "três posts", "12 itens". */
function count(n: number): string {
  return n >= 1 && n <= 10 ? t(`today.number_${n}`) : String(n);
}

function part(kind: "posts" | "overdue" | "upcoming", n: number): string {
  return t(`today.summary_${kind}_${n === 1 ? "one" : "other"}`, { n: count(n) });
}

export function todaySummary(today: Today): string {
  const parts = [
    ["posts", today.posts.filter((p) => !p.done).length],
    ["overdue", today.overdue.length],
    ["upcoming", today.upcoming.length],
  ] as const;
  const phrases = parts.filter(([, n]) => n > 0).map(([kind, n]) => part(kind, n));
  if (phrases.length === 0) return t("today.nothing");
  const last = phrases.pop() as string;
  const text = phrases.length ? `${phrases.join(", ")}${t("today.and")}${last}` : last;
  return `${text[0].toUpperCase()}${text.slice(1)}.`;
}
