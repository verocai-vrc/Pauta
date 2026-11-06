import { describe, expect, it } from "vitest";
import type { Board, Card, Index } from "../api";
import { ideaBoard, latestIdeas } from "./ideas";

function board(id: string, kind: Board["kind"]): Board {
  return {
    id,
    name: id,
    kind,
    color: null,
    columns: [
      { id: "idea", name: "Ideias", role: "inbox", limit: null },
      { id: "backlog", name: "Backlog", role: null, limit: null },
    ],
    dir: id,
    hash: "",
  };
}

function card(id: string, boardId: string, column: string, created: string | null): Card {
  return { id, board: boardId, column, created, title: id } as Card;
}

function index(boards: Board[], cards: Card[]): Index {
  return { boards, cards } as Index;
}

describe("ideaBoard", () => {
  it("returns the first content board", () => {
    const i = index([board("p", "project"), board("c1", "content"), board("c2", "content")], []);
    expect(ideaBoard(i)?.id).toBe("c1");
  });

  it("returns null without a content board", () => {
    expect(ideaBoard(index([board("p", "project")], []))).toBeNull();
    expect(ideaBoard(index([], []))).toBeNull();
  });
});

describe("latestIdeas", () => {
  it("lists inbox cards of content boards, newest first, undated last", () => {
    const i = index(
      [board("c", "content")],
      [
        card("old", "c", "idea", "2026-10-01T10:00:00-03:00"),
        card("none", "c", "idea", null),
        card("new", "c", "idea", "2026-11-02T10:00:00-03:00"),
        card("mid", "c", "idea", "2026-10-20T10:00:00-03:00"),
      ],
    );
    expect(latestIdeas(i).map((c) => c.id)).toEqual(["new", "mid", "old", "none"]);
  });

  it("ignores other columns and project boards, backlog included", () => {
    const i = index(
      [board("c", "content"), board("p", "project")],
      [
        card("a", "c", "idea", "2026-11-01T00:00:00Z"),
        card("b", "c", "backlog", "2026-11-02T00:00:00Z"),
        card("c", "p", "idea", "2026-11-03T00:00:00Z"),
        card("d", "p", "backlog", "2026-11-03T00:00:00Z"),
      ],
    );
    expect(latestIdeas(i).map((c) => c.id)).toEqual(["a"]);
  });

  it("collects ideas from every content board", () => {
    const i = index(
      [board("c1", "content"), board("c2", "content")],
      [card("a", "c1", "idea", "2026-11-01T00:00:00Z"), card("b", "c2", "idea", "2026-11-02T00:00:00Z")],
    );
    expect(latestIdeas(i).map((c) => c.id)).toEqual(["b", "a"]);
  });
});

describe("latestIdeas order", () => {
  it("keeps undated cards together at the end", () => {
    const i = index(
      [board("c", "content")],
      [card("x", "c", "idea", null), card("a", "c", "idea", "2026-11-01T00:00:00Z"), card("y", "c", "idea", null)],
    );
    expect(latestIdeas(i).map((c) => c.id)).toEqual(["a", "x", "y"]);
  });
});
