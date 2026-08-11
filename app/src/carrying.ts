import { syncNow, syncState } from "./core";

const AFTER_A_CHANGE = 4_000;
const EVERY_SO_OFTEN = 15 * 60_000;

/**
 * Never blocks a local write and never interrupts: a folder that is not there
 * is retried in silence, and what happened is read in the maintenance panel.
 */
export function carrying(brought: () => void) {
  let on = false;
  let running = false;
  let soon: ReturnType<typeof setTimeout> | undefined;

  const go = (way?: "push" | "pull") => {
    if (!on || running) return;
    running = true;
    syncNow(way)
      .then((came) => came && brought())
      .catch(() => {})
      .finally(() => {
        running = false;
      });
  };

  const pull = () => go("pull");
  const both = () => go();

  syncState()
    .then((state) => {
      on = state.chosen !== undefined;
      if (on) pull();
    })
    .catch(() => {});

  window.addEventListener("focus", pull);
  const beat = setInterval(both, EVERY_SO_OFTEN);

  return {
    /** Debounced: a burst of edits is one push, not one per keystroke. */
    changed() {
      clearTimeout(soon);
      soon = setTimeout(() => go("push"), AFTER_A_CHANGE);
    },
    /** The panel can turn it on or off while the window stays open. */
    recheck() {
      syncState()
        .then((state) => {
          on = state.chosen !== undefined;
        })
        .catch(() => {});
    },
    stop() {
      clearTimeout(soon);
      clearInterval(beat);
      window.removeEventListener("focus", pull);
    },
  };
}
