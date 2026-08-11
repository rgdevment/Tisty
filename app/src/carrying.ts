import { syncNow, syncState } from "./core";

const AFTER_A_CHANGE = 4_000;
const EVERY_SO_OFTEN = 15 * 60_000;
/** A hung share must not silence the carrier for the rest of the session. */
const GIVE_UP_AFTER = 60_000;

/**
 * Never blocks a local write and never interrupts: a folder that is not there
 * is retried in silence, and what happened is read in the maintenance panel.
 */
export function carrying(brought: () => void) {
  let on = false;
  let gone = false;
  let running = false;
  let owed = false;
  let soon: ReturnType<typeof setTimeout> | undefined;

  const go = (way?: "push" | "pull") => {
    if (gone || !on) return;
    // A change made mid-carry is remembered, or it would never go up.
    if (running) {
      owed = true;
      return;
    }
    running = true;

    let expire: ReturnType<typeof setTimeout>;
    const patience = new Promise<boolean>((resolve) => {
      expire = setTimeout(() => resolve(false), GIVE_UP_AFTER);
    });

    Promise.race([syncNow(way), patience])
      .then((came) => came && !gone && brought())
      .catch(() => {})
      .finally(() => {
        clearTimeout(expire);
        running = false;
        if (owed && !gone) {
          owed = false;
          go("push");
        }
      });
  };

  const pull = () => go("pull");
  const both = () => go();

  const settings = (then?: () => void) =>
    syncState()
      .then((state) => {
        if (gone) return;
        on = state.chosen != null;
        then?.();
      })
      .catch(() => {});

  settings(pull);

  window.addEventListener("focus", pull);
  const beat = setInterval(both, EVERY_SO_OFTEN);

  return {
    /** Debounced: a burst of edits is one push, not one per keystroke. */
    changed() {
      clearTimeout(soon);
      soon = setTimeout(() => go("push"), AFTER_A_CHANGE);
    },
    /** Turning it on has to bring the others in now, not in fifteen minutes. */
    recheck() {
      const was = on;
      settings(() => !was && on && go());
    },
    stop() {
      gone = true;
      clearTimeout(soon);
      clearInterval(beat);
      window.removeEventListener("focus", pull);
    },
  };
}
