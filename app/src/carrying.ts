import { syncNow, syncState, type Carried } from "./core";

const AFTER_A_CHANGE = 4_000;
const EVERY_SO_OFTEN = 15 * 60_000;
/** A hung share must not silence the carrier for the rest of the session. */
const GIVE_UP_AFTER = 60_000;

type Way = "push" | "pull" | undefined;

const owing = (held: Way | null, next: Way): Way =>
  held === null || held === next ? next : undefined;

/**
 * Never blocks a local write and never interrupts: a folder that is not there
 * is retried in silence, and what happened is read in the maintenance panel.
 */
export function carrying(brought: () => void) {
  let folder: string | undefined;
  let gone = false;
  let running = false;
  let owed: Way | null = null;
  let soon: ReturnType<typeof setTimeout> | undefined;
  let expire: ReturnType<typeof setTimeout> | undefined;

  const go = (way: Way) => {
    if (gone || folder === undefined) return;
    // Remembered as what it was: relaunching a pull as a push strands theirs.
    if (running) {
      owed = owing(owed, way);
      return;
    }
    running = true;

    const patience = new Promise<Carried>((resolve) => {
      expire = setTimeout(() => resolve("same"), GIVE_UP_AFTER);
    });

    Promise.race([syncNow(way), patience])
      .then((answer) => answer === "came" && !gone && brought())
      .catch(() => {})
      .finally(() => {
        clearTimeout(expire);
        expire = undefined;
        running = false;
        if (owed !== null && !gone) {
          const again = owed;
          owed = null;
          go(again);
        }
      });
  };

  const pull = () => go("pull");
  const both = () => go(undefined);

  const settings = (then?: (was: string | undefined) => void) =>
    syncState()
      .then((state) => {
        if (gone) return;
        const was = folder;
        folder = state.chosen ?? undefined;
        then?.(was);
      })
      .catch(() => {});

  settings(() => pull());

  window.addEventListener("focus", pull);
  const beat = setInterval(both, EVERY_SO_OFTEN);

  return {
    /** Debounced: a burst of edits is one push, not one per keystroke. */
    changed() {
      clearTimeout(soon);
      soon = setTimeout(() => go("push"), AFTER_A_CHANGE);
    },
    /** A folder just turned on, or swapped for another, has to be read now. */
    recheck() {
      settings((was) => was !== folder && both());
    },
    stop() {
      gone = true;
      clearTimeout(soon);
      clearTimeout(expire);
      clearInterval(beat);
      window.removeEventListener("focus", pull);
    },
  };
}
