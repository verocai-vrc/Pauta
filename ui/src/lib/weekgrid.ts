// Layout of timed items in a day column of the week view.
import type { AgendaItem } from "./api";

export const HOUR_PX = 56;
export const MIN_BLOCK_MIN = 30;
const DAY_MIN = 24 * 60;

export interface Placed {
  item: AgendaItem;
  /** Minutes from midnight. */
  start: number;
  end: number;
  lane: number;
  lanes: number;
}

export function minutesOf(time: string | null): number {
  if (!time) return 0;
  const [h, m] = time.split(":").map(Number);
  return h * 60 + m;
}

/** Blocks for the timed items of one day, overlapping ones side by side. */
export function layoutDay(items: AgendaItem[], day: string): Placed[] {
  const timed = items
    .filter((i) => i.when.time !== null && i.when.date === day)
    .map((item) => {
      const start = minutesOf(item.when.time);
      // An end on the same day sets the length; otherwise a short default block.
      const sameDayEnd = item.end && item.end.date === day && item.end.time ? minutesOf(item.end.time) : null;
      const end = Math.min(DAY_MIN, Math.max(start + MIN_BLOCK_MIN, sameDayEnd ?? 0));
      return { item, start, end, lane: 0, lanes: 1 };
    })
    .sort((a, b) => a.start - b.start || a.end - b.end);

  // Greedy lanes inside each cluster of overlapping blocks.
  let cluster: Placed[] = [];
  let clusterEnd = -1;
  const laneEnds: number[] = [];
  const flush = () => {
    for (const p of cluster) p.lanes = laneEnds.length;
    cluster = [];
    laneEnds.length = 0;
  };
  for (const p of timed) {
    if (p.start >= clusterEnd) flush();
    let lane = laneEnds.findIndex((e) => e <= p.start);
    if (lane < 0) lane = laneEnds.length;
    laneEnds[lane] = p.end;
    p.lane = lane;
    cluster.push(p);
    clusterEnd = Math.max(clusterEnd, p.end);
  }
  flush();
  return timed;
}

/** Local time `HH:MM` for a vertical offset in a day column, snapped to 15 minutes. */
export function timeAtOffset(offsetPx: number): string {
  const minutes = Math.min(DAY_MIN - 15, Math.max(0, Math.round(((offsetPx / HOUR_PX) * 60) / 15) * 15));
  return `${String(Math.floor(minutes / 60)).padStart(2, "0")}:${String(minutes % 60).padStart(2, "0")}`;
}
