// How many items each widget shows before "+N · Ver mais" (section 4.1 of the Início spec).
export interface Limits {
  posts: number;
  warnings: number;
  agenda: number;
  rhythm: number;
  ideas: number;
  projects: number;
}

export const NORMAL: Limits = { posts: 2, warnings: 5, agenda: 12, rhythm: 3, ideas: 3, projects: 4 };

/** Windows at least 960px tall. */
export const ROOMY: Limits = { posts: 4, warnings: 9, agenda: 20, rhythm: 3, ideas: 6, projects: 6 };
