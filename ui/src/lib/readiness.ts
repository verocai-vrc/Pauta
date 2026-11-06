// The words for what a post still lacks before it can go out. One place, because
// Problems.svelte (the long sentence) and Início (the short phrase) must agree.
import type { Problem } from "./api";
import { t } from "./i18n";

/** The full sentence, for the card editor and Hoje. */
export function problemText(p: Problem): string {
  return p.code === "art_pending"
    ? t(`ready.art_pending_${p.detail === "files_present" ? "files" : "waiting"}`)
    : t(`ready.${p.code}`, { n: p.detail ?? "" });
}

/** A few words that fit after "Post de amanhã:". */
export function problemShort(p: Problem): string {
  return p.code === "art_pending"
    ? t(`home.problem_art_pending_${p.detail === "files_present" ? "files" : "waiting"}`)
    : t(`home.problem_${p.code}`);
}
