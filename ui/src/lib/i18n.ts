// Every UI string goes through t(), so English can be added later as another
// dictionary without touching components (section 12.3).
import { ptBR } from "./locales/pt-BR";

export type Messages = Record<string, string>;

const dictionaries: Record<string, Messages> = { "pt-BR": ptBR };
let current: Messages = ptBR;

export function setLanguage(lang: string) {
  current = dictionaries[lang] ?? ptBR;
}

/** `t("today.count", { n: 3 })` replaces `{n}` in the message. */
export function t(key: string, vars?: Record<string, string | number>): string {
  const msg = current[key] ?? ptBR[key] ?? key;
  if (!vars) return msg;
  // One pass with a function: a value holding `$&` or `{title}` must come out as typed, and not be filled again.
  return msg.replace(/\{(\w+)\}/g, (match, name: string) => (name in vars ? String(vars[name]) : match));
}

/** A message with a singular and a plural: `tn("x", 1)` reads `x_one`, anything else reads `x_other`. */
export function tn(key: string, n: number, vars?: Record<string, string | number>): string {
  return t(`${key}_${n === 1 ? "one" : "other"}`, { n, ...vars });
}
