import DOMPurify from "dompurify";
import { marked } from "marked";

marked.setOptions({ gfm: true, breaks: false });

export function renderMarkdown(src: string): string {
  return DOMPurify.sanitize(marked.parse(src, { async: false }) as string);
}

const TASK = /^(\s*[-*+]\s+\[)([ xX])(\])/gm;

/** Flips the n-th `- [ ]` / `- [x]` in the text. */
export function toggleTask(body: string, n: number): string {
  let i = 0;
  return body.replace(TASK, (m, pre: string, mark: string, post: string) => {
    if (i++ !== n) return m;
    return pre + (mark === " " ? "x" : " ") + post;
  });
}

interface Section {
  title: string;
  start: number;
  contentStart: number;
  end: number;
}

/** `## Heading` sections outside code fences, like pauta-core's markdown::sections. */
export function sections(body: string): Section[] {
  const out: Section[] = [];
  let inFence = false;
  let offset = 0;
  for (const line of body.split(/(?<=\n)/)) {
    if (/^\s*(```|~~~)/.test(line)) inFence = !inFence;
    const m = !inFence && /^##\s+(.+?)\s*#*\s*$/.exec(line.replace(/\r?\n$/, ""));
    if (m) {
      if (out.length) out[out.length - 1].end = offset;
      out.push({ title: m[1].trim(), start: offset, contentStart: offset + line.length, end: body.length });
    }
    offset += line.length;
  }
  return out;
}

/** The text that gets posted; must match pauta-core's markdown::post_text. */
export function postText(rawBody: string, networkNames: string[]): string {
  // The `## Arte` section is a brief for the design tool, never post text.
  const art = sections(rawBody).find((s) => s.title.toLowerCase() === "arte");
  const body = art ? rawBody.slice(0, art.start) + rawBody.slice(art.end) : rawBody;
  const secs = sections(body);
  const find = (name: string) => secs.find((s) => s.title.toLowerCase() === name.toLowerCase());
  for (const n of networkNames) {
    const s = find(n);
    if (s) return body.slice(s.contentStart, s.end).trim();
  }
  const notes = find("Notas");
  if (!notes) return body.trim();
  return (body.slice(0, notes.start) + body.slice(notes.end)).trim();
}

/** Characters as platforms count them (code points, not UTF-16 units). */
export function charCount(s: string): number {
  return [...s].length;
}
