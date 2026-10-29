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
  let msg = current[key] ?? ptBR[key] ?? key;
  if (vars) {
    for (const [k, v] of Object.entries(vars)) msg = msg.replaceAll(`{${k}}`, String(v));
  }
  return msg;
}
