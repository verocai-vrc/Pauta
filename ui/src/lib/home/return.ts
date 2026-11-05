/**
 * Whether opening the window should take the author back to Início: only on the first opening of a
 * new day, and never out of an editor, modal, palette or trash that is open (they would lose their place).
 * `seen` is null until the first load finishes.
 */
export function shouldReturnHome(seen: string | null, date: string, busy: boolean): boolean {
  return seen !== null && seen !== date && !busy;
}
