import type { Caret, Standing } from "./pageCaret";
import { type Kept, keptUnlaid, type Writer } from "./pageWriter";

export type Behind = { readonly kind: string; readonly keyed: boolean };

type Acting = {
  readonly unplaced: (kind: string, standing: Standing) => boolean;
  readonly act: (held: Behind, standing: Standing) => void;
};

export const behindOn = (writer: Writer, caret: Caret, { unplaced, act }: Acting) => {
  const { hands } = writer;
  const ours = () => hands.ours?.() !== false;
  let behind: Behind[] = [];
  let heldOn: { readonly at: number; readonly text: string } | null = null;
  let again = false;

  const standsAt = () => (writer.kept ? { at: writer.kept.at, text: hands.text() } : null);

  const dropped = () => {
    behind = [];
    heldOn = null;
  };

  const hold = (kind: string, keyed: boolean) => {
    if (again) return;
    if (!behind.length) heldOn = standsAt();
    behind.push({ kind, keyed });
  };

  const laidHere = (kept: Kept, { range }: Standing) => {
    const { at } = kept;
    if (keptUnlaid(kept)) return true;
    const here = range?.collapsed && caret.spanOfRange(range)?.from === at;
    return Boolean(here && caret.shownAt(at));
  };

  const caughtUp = (): Standing | null => {
    let acted = false;
    for (let first = behind[0]; first; first = behind[0]) {
      const kept = writer.kept;
      const same = kept && heldOn?.at === kept.at && heldOn.text === hands.text();
      if (!kept || !same || !ours()) {
        dropped();
        return null;
      }
      const standing = caret.standing();
      if (!laidHere(kept, standing) || unplaced(first.kind, standing))
        return acted ? null : standing;
      behind = behind.slice(1);
      act(first, standing);
      acted = true;
      heldOn = standsAt();
    }
    return null;
  };

  const left = () => {
    caughtUp();
    dropped();
  };

  const pressed = (repeated: boolean) => {
    again = repeated;
  };

  return { hold, dropped, caughtUp, left, pressed };
};
