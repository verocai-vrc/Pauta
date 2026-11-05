// How many items each widget shows before "+N · Ver mais" (section 4.1 of the Início spec).
export interface Limits {
  posts: number;
  warnings: number;
  agenda: number;
  rhythm: number;
  ideas: number;
  projects: number;
}

export const NORMAL: Limits = { posts: 2, warnings: 5, agenda: 12, rhythm: 2, ideas: 3, projects: 4 };

/** Windows at least 960px tall. Sized so the worst day fits 1920x1000, a maximized window on a 1080p screen. */
export const ROOMY: Limits = { posts: 2, warnings: 6, agenda: 14, rhythm: 3, ideas: 4, projects: 5 };
