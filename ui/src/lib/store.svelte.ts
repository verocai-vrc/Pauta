// App-wide reactive state. Everything is reloaded from the backend whenever the
// data folder changes (our writes, the MCP server, or an external editor).
import { listen } from "@tauri-apps/api/event";
import { api, errorMessage, type Board, type Card, type Index, type Status, type Today } from "./api";

export type View = "home" | "boards" | "calendar" | "rhythm" | "settings";
export type CalendarMode = "today" | "month" | "week" | "agenda";

export interface Toast {
  id: number;
  text: string;
  kind: "info" | "error";
}

export const app = $state({
  status: null as Status | null,
  index: null as Index | null,
  today: null as Today | null,
  loadError: null as string | null,
  view: "home" as View,
  calendarMode: "month" as CalendarMode,
  boardId: null as string | null,
  /** Card open in the side editor. */
  openCard: null as string | null,
  /** Event open in the editor; "new" with `newEventDate` to create one. */
  openEvent: null as string | null,
  newEventDate: null as string | null,
  paletteOpen: false,
  trashOpen: false,
  toasts: [] as Toast[],
  /** Bumped on every reload so views can refetch derived data (e.g. agenda). */
  version: 0,
});

let toastId = 0;

export function toast(text: string, kind: Toast["kind"] = "info") {
  const id = ++toastId;
  app.toasts.push({ id, text, kind });
  setTimeout(() => (app.toasts = app.toasts.filter((t) => t.id !== id)), kind === "error" ? 7000 : 3500);
}

export function fail(e: unknown) {
  toast(errorMessage(e), "error");
}

export async function reload() {
  try {
    app.status = await api.status();
    if (!app.status.root) return;
    app.index = await api.index();
    app.today = await api.today();
    app.loadError = null;
    if (!app.boardId || !app.index.boards.some((b) => b.id === app.boardId)) {
      app.boardId = app.index.boards[0]?.id ?? null;
    }
    app.version++;
  } catch (e) {
    app.loadError = errorMessage(e);
  }
}

export function board(id: string | null | undefined): Board | undefined {
  return app.index?.boards.find((b) => b.id === id);
}

export function card(id: string | null | undefined): Card | undefined {
  return app.index?.cards.find((c) => c.id === id);
}

export function networkLabel(id: string | null | undefined): string {
  if (!id) return "";
  return app.index?.config.networks.find((n) => n.id === id)?.label ?? id;
}

/** The day in detail: a tab of the calendar since Início became the first screen. */
export function openToday() {
  app.calendarMode = "today";
  app.view = "calendar";
}

export function openItem(kind: "card" | "event" | string, id: string) {
  if (kind === "event") app.openEvent = id;
  else app.openCard = id;
}

/** Keeps "today" and the clock fresh while the window stays open (midnight, the now-line). */
async function refreshToday() {
  if (!app.status?.root || app.loadError) return;
  try {
    const next = await api.today();
    if (app.today && next.date !== app.today.date) await reload();
    else app.today = next;
  } catch {
    // The next tick or file change will try again.
  }
}

export async function start() {
  await reload();
  setInterval(() => document.visibilityState === "visible" && refreshToday(), 60_000);
  document.addEventListener("visibilitychange", () => document.visibilityState === "visible" && refreshToday());
  await listen("pauta://changed", () => reload());
  // The tray and the notifications still ask for "today", which is no longer a view.
  await listen<string>("pauta://navigate", (e) => (e.payload === "today" ? openToday() : (app.view = e.payload as View)));
  await listen<{ kind: string; id: string }>("pauta://open", (e) => openItem(e.payload.kind, e.payload.id));
}
