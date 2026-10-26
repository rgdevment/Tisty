import { listen } from "@tauri-apps/api/event";
import { toldAside } from "./coming";
import {
  type Carried,
  type Carrying,
  folderAstir,
  syncNow,
  syncState,
  whatWentAmiss,
} from "./core";
import { settledElsewhere } from "./deciding";
import { knock, settled } from "./knocking";
import { t } from "./locales";
import { saidPlainly } from "./refusal";
import { lastingCause } from "./syncSaid";
import { lookAgain } from "./syncSeen";

const AFTER_A_CHANGE = 4_000;
const EVERY_SO_OFTEN = 15 * 60_000;
const TAKING_LONG = 60_000;
const A_GLANCE = 30_000;

type Way = "push" | "pull" | undefined;

export type Awry =
  | { why: "slow" }
  | { why: "busy" }
  | { why: "broke"; said: string }
  | { why: "amiss"; said: string }
  | { why: "away"; said: string };

// A cloud reached through its API has no folder to name, and is carried with all the same.
const chosenPlace = (state: Carrying): string | undefined =>
  state.chosen ?? (state.mode === "cloud" ? `cloud:${state.cloud ?? ""}` : undefined);

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
        // A round that did nothing, here or in the cloud another round held, still owes what was asked.
        const busy = answer.carried === "busy" || answer.deferred?.reason === "busy";
        if (busy) {
          owed = owing(owed, way);
          stalled = true;
        }
        if (!busy && answer.owed && !answer.deferred && way !== "push") owed = owing(owed, "push");
        if (!busy && !answer.deferred) toldAside(null);
        if (mine !== round) return busy ? "busy" : answer.carried;
        if (answer.undecided.length) atOdds(answer.undecided);
        // A history turned away is the whole point of signing it, and the round that finds it is
        // the one nobody is watching: saying nothing here is how it goes unnoticed for months.
        if (answer.unconfirmed?.length) void knock();
        else if (!busy && way !== "push") settled();
        const amiss = whatWentAmiss({ ...answer, unconfirmed: [] });
        const away = answer.deferred?.reason === "offline" ? lastingCause(answer.deferred) : null;
        awry(
          busy
            ? { why: "busy" }
            : amiss
              ? { why: "amiss", said: t(amiss) }
              : away
                ? { why: "away", said: away }
                : null,
        );
        return busy ? "busy" : answer.carried;
      })
      .catch((problem) => {
        const ours = (problem as { code?: string } | null)?.code === "cloudSealingHere";
        if (!gone && mine === round)
          awry(ours ? null : { why: "broke", said: saidPlainly(problem) });
        return undefined;
      })
      .finally(() => {
        if (mine === round) {
          clearTimeout(expire);
          expire = undefined;
          running = false;
        }
        if (!gone && mine === round && !stalled) {
          void lookAgain();
          if (way !== "push") void settledElsewhere();
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
        folder = chosenPlace(state);
        then?.(was);
      })
      .catch(() => {});

  let astir: string | undefined;
  // The provider also says when this machine's own upload landed, so its word is weighed like a glance.
  const stirred = () => {
    if (gone) return;
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
          // Read again once the round is done, so our own push is absorbed and the next stir is still heard.
          return folderAstir()
            .then((after) => {
              if (!gone) astir = after;
            })
            .catch(() => {
              astir = undefined;
            });
        });
      })
      .catch(() => {});
  };

  const glance = () => {
    if (folder === undefined || document.visibilityState !== "visible") return;
    stirred();
  };

  const seen = () => {
    if (document.visibilityState === "visible") void pull();
  };

  void settings(() => void pull());

  window.addEventListener("focus", pull);
  document.addEventListener("visibilitychange", seen);
  const told = listen("told", stirred);
  const beat = setInterval(both, EVERY_SO_OFTEN);
  const glancing = setInterval(glance, A_GLANCE);

  return {
    now: both,
    changed() {
      clearTimeout(soon);
      soon = setTimeout(() => go("push"), AFTER_A_CHANGE);
    },
    recheck() {
      void lookAgain();
      void settings((was) => {
        if (was !== folder) void both();
      });
    },
    stop() {
      gone = true;
      clearTimeout(soon);
      clearTimeout(expire);
      clearTimeout(later);
      clearInterval(beat);
      clearInterval(glancing);
      void told.then((stop) => stop());
      window.removeEventListener("focus", pull);
      document.removeEventListener("visibilitychange", seen);
    },
  };
}
