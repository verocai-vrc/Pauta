// Types mirror the Rust structs serialized by pauta-core; commands mirror
// crates/pauta-app/src/commands.rs.
import { invoke } from "@tauri-apps/api/core";

export interface When {
  /** Local date, YYYY-MM-DD. */
  date: string;
  /** Local time HH:MM:SS, null for all-day items. */
  time: string | null;
  /** Absolute instant (UTC RFC 3339), null for all-day items. */
  at: string | null;
}

export type PubStatus = "planned" | "scheduled" | "published" | "skipped";

export interface Publication {
  at: When | null;
  status: PubStatus;
  url: string | null;
  published_at: string | null;
}

export interface Links {
  board: string | null;
  siblings: string[];
  article: string | null;
}

export interface Metrics {
  impressions: number | null;
  reactions: number | null;
  comments: number | null;
  shares: number | null;
  saves: number | null;
  clicks: number | null;
  recorded: string | null;
}

export interface Card {
  id: string;
  board: string;
  title: string;
  kind: string;
  column: string;
  rank: string;
  tags: string[];
  network: string | null;
  publication: Publication | null;
  due: When | null;
  reminders: string[] | null;
  links: Links;
  created: string | null;
  updated: string | null;
  metrics: Metrics | null;
  art: Art | null;
  assets: string[];
  /** When the card entered its current column; null if unknown (see `Card` in pauta-core). */
  column_since: string | null;
  body: string;
  checklist: { done: number; total: number };
  path: string;
  hash: string;
}

export type ColumnRole = "inbox" | "scheduled" | "done";

export interface Column {
  id: string;
  name: string;
  role: ColumnRole | null;
  limit: number | null;
}

export type BoardKind = "content" | "project" | "generic";

export interface Board {
  id: string;
  name: string;
  kind: BoardKind;
  color: string | null;
  columns: Column[];
  dir: string;
  hash: string;
}

export interface Slot {
  days: string[];
  time: string;
}

export interface CadenceGoal {
  posts: number;
  per: "week" | "month";
}

export interface MediaFormat {
  id: string;
  label: string;
  size: string;
  note: string | null;
}

export interface Problem {
  code: "no_text" | "over_limit" | "media_missing" | "art_pending";
  detail: string | null;
}

export interface Art {
  status: "requested" | "ready";
  format: string | null;
  requested_at: string | null;
}

export interface NetworkConfig {
  id: string;
  label: string;
  url: string | null;
  char_limit: number | null;
  slots: Slot[];
  cadence_goal: CadenceGoal | null;
  media: { requires_media: boolean; formats: MediaFormat[] };
}

export interface Config {
  schema_version: number;
  timezone: string;
  language: string;
  board_order: string[];
  networks: NetworkConfig[];
  reminders: { post_default: string[]; deadline_default: string[] };
  quiet_hours: { from: string; to: string } | null;
  empty_queue_days: number;
  stale_after_days: number;
  auto_archive_days: number | null;
  art: { tool_url: string; brand: string | null };
}

export interface PautaEvent {
  id: string;
  title: string;
  kind: string;
  when: When;
  end: When | null;
  reminders: string[] | null;
  links: Links;
  tags: string[];
  body: string;
  path: string;
  hash: string;
}

export interface InvalidFile {
  path: string;
  error: string;
}

export interface Index {
  config: Config;
  boards: Board[];
  cards: Card[];
  events: PautaEvent[];
  invalid: InvalidFile[];
  /** What stands between each pending post and "post it now"; ready cards are absent. */
  readiness: Record<string, Problem[]>;
}

export type AgendaKind = "publication" | "due" | "event";

export interface AgendaItem {
  kind: AgendaKind;
  id: string;
  board: string | null;
  title: string;
  when: When;
  end: When | null;
  network: string | null;
  status: string | null;
  column: string | null;
  done: boolean;
}

export interface CardRef {
  id: string;
  board: string;
  title: string;
  column: string;
  network: string | null;
}

export interface CadenceProgress {
  network: string;
  label: string;
  goal: number;
  per: "week" | "month";
  done: number;
  scheduled: number;
  period_start: string;
  period_end: string;
}

export interface Bucket {
  start: string;
  end: string;
  counts: Record<string, number>;
  total: number;
}

export interface PublishedPost {
  id: string;
  title: string;
  network: string;
  date: string;
  published_at: string | null;
  url: string | null;
  archived: boolean;
}

export interface WeekdayStat {
  weekday: number;
  posts: number;
  avg_engagement: number;
}

export interface NetworkInsight {
  network: string;
  label: string;
  posts: number;
  impressions: number;
  engagement: number;
  avg_engagement: number;
  engagement_rate: number | null;
  by_weekday: WeekdayStat[];
}

export interface Insights {
  posts_with_metrics: number;
  min_samples: number;
  networks: NetworkInsight[];
  top: { id: string; title: string; network: string; date: string; engagement: number; impressions: number | null }[];
}

export interface Rhythm {
  insights: Insights;
  cadence: CadenceProgress[];
  history: { weeks: Bucket[]; months: Bucket[]; total_posts: number };
  recent: PublishedPost[];
}

export interface Today {
  date: string;
  /** Local time in the configured zone, `HH:MM:SS`. */
  time: string;
  posts: AgendaItem[];
  upcoming: AgendaItem[];
  overdue: AgendaItem[];
  empty_queues: { network: string; label: string; days: number; candidates: CardRef[] }[];
  stale: CardRef[];
  cadence: CadenceProgress[];
  invalid_files: number;
}

export interface Status {
  root: string | null;
  suggested: string;
  error: string | null;
}

export interface NewCard {
  title: string;
  column?: string;
  kind?: string;
  network?: string;
  tags?: string[];
  body?: string;
  due?: string;
  publication_at?: string;
  reminders?: string[];
  links_board?: string;
  links_article?: string;
  top?: boolean;
}

/** Absent = unchanged; `null` = clear (for nullable fields). */
export interface CardPatch {
  title?: string;
  kind?: string;
  column?: string;
  tags?: string[];
  network?: string | null;
  due?: string | null;
  reminders?: string[] | null;
  publication?: { at?: string | null; status?: PubStatus; url?: string | null } | null;
  links_board?: string | null;
  metrics?: Partial<Omit<Metrics, "recorded">> | null;
  body?: string;
}

export interface NewEvent {
  title: string;
  kind?: string;
  date: string;
  end?: string;
  reminders?: string[];
  board?: string;
  tags?: string[];
  body?: string;
}

export interface EventPatch {
  title?: string;
  kind?: string;
  date?: string;
  end?: string | null;
  reminders?: string[] | null;
  board?: string | null;
  tags?: string[];
  body?: string;
}

export interface CmdError {
  kind: "conflict" | "not_found" | "invalid" | "rule" | "io" | "no_workspace" | "other";
  message: string;
  disk_content?: string;
  disk_hash?: string;
}

export function isCmdError(e: unknown): e is CmdError {
  return typeof e === "object" && e !== null && "kind" in e && "message" in e;
}

export function errorMessage(e: unknown): string {
  if (isCmdError(e)) return e.message;
  return String(e);
}

export interface TrashEntry {
  path: string;
  original: string;
  title: string;
  deleted_at: string;
}

export interface ArchivedItem {
  path: string;
  kind: "card" | "event" | "board";
  board: string | null;
  title: string;
}

export interface Preferences {
  close_to_tray: boolean;
  autostart: boolean;
  theme: "system" | "light" | "dark";
}

export interface McpSnippet {
  binary: string;
  binary_exists: boolean;
  claude_code: string;
  claude_desktop: string;
}

export interface ConfigPatch {
  timezone?: string;
  networks?: NetworkConfig[];
  reminders?: { post_default: string[]; deadline_default: string[] };
  quiet_hours?: { from: string; to: string } | null;
  empty_queue_days?: number;
  stale_after_days?: number;
  auto_archive_days?: number | null;
  art?: { tool_url?: string; brand?: string };
}

export interface ImportPreview {
  columns: { field: string; header: string }[];
  matched: { line: number; card_id: string; title: string; network: string | null; before: Metrics | null; after: Metrics }[];
  unmatched: [number, string][];
  skipped_cells: [number, string, string][];
}

export interface ImportResult {
  applied: string[];
  failed: [string, string][];
}

export const api = {
  status: () => invoke<Status>("app_status"),
  setup: (root: string) => invoke<Status>("setup_workspace", { root }),
  index: () => invoke<Index>("get_index"),
  today: () => invoke<Today>("get_today"),
  rhythm: () => invoke<Rhythm>("get_rhythm"),
  previewMetricsImport: (text: string) => invoke<ImportPreview>("preview_metrics_import", { text }),
  importMetrics: (text: string) => invoke<ImportResult>("import_metrics", { text }),
  exportIcs: (path: string, includeDone: boolean) => invoke<number>("export_ics", { path, includeDone }),
  agenda: (from: string, to: string) => invoke<AgendaItem[]>("get_agenda", { from, to }),
  findSlots: (network: string, count = 3, exclude?: string) =>
    invoke<{ local: string; date: string; time: string }[]>("find_slots", { network, count, exclude }),
  createCard: (board: string, card: NewCard) => invoke<Card>("create_card", { board, card }),
  updateCard: (id: string, patch: CardPatch, expected?: string) =>
    invoke<Card>("update_card", { id, patch, expected: expected ?? null }),
  moveCard: (id: string, column: string, index: number | null) => invoke<Card>("move_card", { id, column, index }),
  schedulePost: (id: string, at?: string) => invoke<Card>("schedule_post", { id, at: at ?? null }),
  markPublished: (id: string, url?: string, at?: string) =>
    invoke<Card>("mark_published", { id, url: url ?? null, at: at ?? null }),
  duplicateCard: (id: string, network: string) => invoke<Card>("duplicate_card", { id, network }),
  archiveCard: (id: string) => invoke<void>("archive_card", { id }),
  trashCard: (id: string) => invoke<void>("trash_card", { id }),
  createEvent: (event: NewEvent) => invoke<PautaEvent>("create_event", { event }),
  updateEvent: (id: string, patch: EventPatch, expected?: string) =>
    invoke<PautaEvent>("update_event", { id, patch, expected: expected ?? null }),
  archiveEvent: (id: string) => invoke<void>("archive_event", { id }),
  trashEvent: (id: string) => invoke<void>("trash_event", { id }),
  createBoard: (name: string, kind: BoardKind) => invoke<Board>("create_board", { name, kind }),
  updateBoard: (id: string, patch: { name?: string; color?: string | null; columns?: Column[] }, expected?: string) =>
    invoke<Board>("update_board", { id, patch, expected: expected ?? null }),
  setBoardOrder: (order: string[]) => invoke<void>("set_board_order", { order }),
  archiveBoard: (id: string) => invoke<void>("archive_board", { id }),
  listTrash: () => invoke<TrashEntry[]>("list_trash"),
  restoreTrash: (path: string) => invoke<void>("restore_trash", { path }),
  listArchive: () => invoke<ArchivedItem[]>("list_archive"),
  restoreArchived: (path: string) => invoke<void>("restore_archived", { path }),
  updateConfig: (patch: ConfigPatch) => invoke<Config>("update_config", { patch }),
  copyPostText: (id: string) => invoke<number>("copy_post_text", { id }),
  requestArt: (id: string, format?: string, brief?: string) =>
    invoke<Card>("request_art", { id, format: format ?? null, brief: brief ?? null }),
  setArtStatus: (id: string, status: "requested" | "ready" | "none") => invoke<Card>("set_art_status", { id, status }),
  attachAssets: (id: string, paths: string[]) => invoke<string[]>("attach_assets", { id, paths }),
  copyArtRequest: (id: string) => invoke<void>("copy_art_request", { id }),
  openArtTool: () => invoke<void>("open_art_tool"),
  copyText: (text: string) => invoke<void>("copy_text", { text }),
  listAssets: (id: string) => invoke<{ name: string; path: string }[]>("list_assets", { id }),
  openAssets: (id: string) => invoke<void>("open_assets", { id }),
  openPath: (path: string) => invoke<void>("open_path", { path }),
  openUrl: (url: string) => invoke<void>("open_url", { url }),
  preferences: () => invoke<Preferences>("get_preferences"),
  setPreference: (key: string, value: unknown) => invoke<Preferences>("set_preference", { key, value }),
  mcpSnippet: () => invoke<McpSnippet>("mcp_snippet"),
  openCapture: () => invoke<void>("open_capture"),
  closeCapture: () => invoke<void>("close_capture"),
};
