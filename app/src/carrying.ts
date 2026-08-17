import { type Settled, syncNow, syncState } from "./core";

const AFTER_A_CHANGE = 4_000;
const EVERY_SO_OFTEN = 15 * 60_000;
const GIVE_UP_AFTER = 60_000;

type Way = "push" | "pull" | undefined;

const owing = (held: Way | null, next: Way): Way =>
  held === null || held === next ? next : undefined;

export function carrying(brought: () => void, atOdds: (ids: string[]) => void = () => {}) {
  let folder: string | undefined;
  let gone = false;
  let running = false;
  let owed: Way | null = null;
  let soon: ReturnType<typeof setTimeout> | undefined;
  let expire: ReturnType<typeof setTimeout> | undefined;

  const go = (way: Way) => {
    if (gone || folder === undefined) return;
    if (running) {
      owed = owing(owed, way);
      return;
    }
    running = true;

    const patience = new Promise<Settled>((resolve) => {
      expire = setTimeout(
        () => resolve({ carried: "same", undecided: [], unreadable: [], astray: [], joined: [] }),
        GIVE_UP_AFTER,
      );
    });

    Promise.race([syncNow(way), patience])
      .then((answer) => {
        if (gone) return;
        if (answer.undecided.length) atOdds(answer.undecided);
        if (answer.carried === "came" || answer.carried === "both") brought();
      })
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
    changed() {
      clearTimeout(soon);
      soon = setTimeout(() => go("push"), AFTER_A_CHANGE);
    },
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
