import { type Carried, folderAstir, syncNow, syncState } from "./core";
import { saidPlainly } from "./refusal";

const AFTER_A_CHANGE = 4_000;
const EVERY_SO_OFTEN = 15 * 60_000;
const TAKING_LONG = 60_000;
const A_GLANCE = 30_000;

type Way = "push" | "pull" | undefined;

export type Awry = { why: "slow" } | { why: "busy" } | { why: "broke"; said: string };

const owing = (held: Way | null, next: Way): Way =>
  held === null || held === next ? next : undefined;

export function carrying(
  brought: () => void,
  atOdds: (ids: string[]) => void = () => {},
  awry: (why: Awry | null) => void = () => {},
) {
  let folder: string | undefined;
  let gone = false;
  let running = false;
  let owed: Way | null = null;
  let round = 0;
  let soon: ReturnType<typeof setTimeout> | undefined;
  let expire: ReturnType<typeof setTimeout> | undefined;
  let later: ReturnType<typeof setTimeout> | undefined;

  const go = (way: Way): Promise<Carried | undefined> => {
    if (gone || folder === undefined) return Promise.resolve(undefined);
    if (running) {
      owed = owing(owed, way);
      return Promise.resolve("busy");
    }
    running = true;
    let stalled = false;
    const mine = ++round;
    // A round against a cloud folder can hang as long as that folder likes, and holding the door
    // shut on it left every later one queued behind one that never came back.
    expire = setTimeout(() => {
      if (gone || mine !== round) return;
      awry({ why: "slow" });
      running = false;
    }, TAKING_LONG);

    return syncNow(way)
      .then((answer) => {
        if (gone) return undefined;
        if (answer.carried === "came" || answer.carried === "both") brought();
        // Busy is a round that did nothing, so what was asked for is still owed.
        if (answer.carried === "busy") {
          owed = owing(owed, way);
          stalled = true;
        }
        if (mine !== round) return answer.carried;
        if (answer.undecided.length) atOdds(answer.undecided);
        awry(answer.carried === "busy" ? { why: "busy" } : null);
        return answer.carried;
      })
      .catch((problem) => {
        if (!gone && mine === round) awry({ why: "broke", said: saidPlainly(problem) });
        return undefined;
      })
      .finally(() => {
        if (mine === round) {
          clearTimeout(expire);
          expire = undefined;
          running = false;
        }
        if (owed === null || gone || running) return;
        const again = owed;
        owed = null;
        if (!stalled) return void go(again);
        clearTimeout(later);
        later = setTimeout(() => go(again), AFTER_A_CHANGE);
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

  let astir: string | undefined;
  const glance = () => {
    if (gone || folder === undefined || document.visibilityState !== "visible") return;
    folderAstir()
      .then((mark) => {
        if (gone) return undefined;
        const was = astir;
        if (was === undefined) {
          astir = mark;
          return undefined;
        }
        if (was === mark) return undefined;
        // The mark only moves on once a round actually took it: a busy or broken one would
        // otherwise swallow what stirred and wait for the next thing to move.
        return pull().then((how) => {
          if (!how || how === "busy") return;
          // Our own push moves the folder too, so the mark is read again rather than kept.
          astir = undefined;
        });
      })
      .catch(() => {});
  };

  const seen = () => {
    if (document.visibilityState === "visible") pull();
  };

  settings(() => pull());

  window.addEventListener("focus", pull);
  document.addEventListener("visibilitychange", seen);
  const beat = setInterval(both, EVERY_SO_OFTEN);
  const glancing = setInterval(glance, A_GLANCE);

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
      clearTimeout(later);
      clearInterval(beat);
      clearInterval(glancing);
      window.removeEventListener("focus", pull);
      document.removeEventListener("visibilitychange", seen);
    },
  };
}
