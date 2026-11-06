import type { Board, Card, Index } from "../api";

/** The board ideas are saved to: the first content board, as the capture window picks it. */
export function ideaBoard(index: Index): Board | null {
  return index.boards.find((b) => b.kind === "content") ?? null;
}

function time(card: Card): number {
  const ms = card.created ? Date.parse(card.created) : NaN;
  // A finite floor, so two undated cards compare as equal instead of -Infinity - -Infinity = NaN.
  return Number.isNaN(ms) ? Number.MIN_SAFE_INTEGER : ms;
}

/** Cards waiting in the `inbox` column of content boards, newest first; project backlogs stay out. */
export function latestIdeas(index: Index): Card[] {
  const inbox = new Set<string>();
  for (const b of index.boards) {
    if (b.kind !== "content") continue;
    for (const c of b.columns) if (c.role === "inbox") inbox.add(`${b.id}/${c.id}`);
  }
  // Array.prototype.sort is stable, so cards without a date keep the index order among themselves.
  return index.cards.filter((c) => inbox.has(`${c.board}/${c.column}`)).sort((a, b) => time(b) - time(a));
}
