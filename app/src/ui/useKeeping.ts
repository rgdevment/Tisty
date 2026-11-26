import {
  type Dispatch,
  type RefObject,
  type SetStateAction,
  useCallback,
  useEffect,
  useMemo,
  useRef,
  useState,
} from "react";
import { docOpened, noteTrouble } from "../core";
import { t } from "../locales";
import { type Backoff, backoff, beat } from "../pacing";
import { saidPlainly } from "../refusal";
import { allUnanswered, codeOf, refusedForGood, stuck, turned, unansweredWords } from "../saving";
import { type Clash, type Read, remembered, savedOnce } from "../weaving";

export const SETTLES = 700;
export const AT_MOST = 2_000;

export type Held = { id: string; body: string };

type Kept = { id: string; title: string };

interface Hands {
  shaped: RefObject<string>;
  typed: RefObject<number>;
  lastRead: RefObject<Map<string, Read>>;
  onScreen: RefObject<string | null>;
  held: RefObject<Held | null>;
  // A round that came while a save was in flight is read once the save lands, not dropped.
  missed: RefObject<boolean>;
  setClashed: Dispatch<SetStateAction<Clash | null>>;
  onKept: (doc: { id: string; title: string }) => void;
  onError: (problem: unknown) => void;
  onSent: (body: string) => void;
  onWoven: (sent: string, woven: string) => void;
}

const sameClash = (was: Clash | null, now: Clash) =>
  was !== null && was.torn === now.torn && was.front === now.front;

// The core names the file when it answers late; a save this window called off itself does not.
const lateFromCore = (problem: unknown) =>
  codeOf(problem) === "saveLate" && Boolean((problem as { name?: string } | null)?.name);

export function useKeeping(hands: Hands) {
  const { shaped, typed, lastRead, onScreen, held, missed, setClashed } = hands;
  const { onKept, onError, onSent, onWoven } = hands;
  const [saving, setSaving] = useState(false);
  const [waiting, setWaiting] = useState(false);
  const [heldOpen, setHeldOpen] = useState<string | null>(null);
  const [saved, setSaved] = useState(0);
  const [behind, setBehind] = useState(0);
  const flushing = useRef<() => unknown>(() => undefined);
  const resending = useRef<(id: string) => unknown>(() => undefined);
  const retaking = useRef<(id: string, text: string, mineStands: boolean) => void>(() => {});
  const pace = useMemo(() => beat(() => flushing.current(), SETTLES, AT_MOST), []);
  // Each document held open waits on its own, so one that saves does not call off another's retries.
  const retries = useRef(new Map<string, Backoff>());
  const retryOf = useCallback((id: string) => {
    const known = retries.current.get(id);
    if (known) return known;
    const made = backoff(() => resending.current(id));
    retries.current.set(id, made);
    return made;
  }, []);
  // Who chose «mine» still means it when the retry goes, or it would clash again.
  const standing = useRef(new Set<string>());
  const retaken = useRef(new Set<string>());
  // Telling a save reloads every paper and nudges the sync, so saves made mid-typing are told at the pause.
  const told = useRef(new Map<string, string>());
  const owed = useRef(new Map<string, Kept>());
  const owing = useRef<ReturnType<typeof setTimeout> | null>(null);
  const telling = useRef(onKept);
  telling.current = onKept;
  const tellOwed = useCallback(() => {
    owing.current = null;
    const typing = Date.now() - typed.current < SETTLES;
    for (const [file, doc] of owed.current) {
      if (typing && told.current.get(file) === doc.title) continue;
      owed.current.delete(file);
      told.current.set(file, doc.title);
      telling.current(doc);
    }
    if (owed.current.size) owing.current = setTimeout(tellOwed, SETTLES);
  }, [typed]);
  const tellKept = useCallback(
    (doc: Kept, file: string) => {
      owed.current.set(file, doc);
      if (owing.current) clearTimeout(owing.current);
      tellOwed();
    },
    [tellOwed],
  );
  // A save still out when the view goes answers into nothing, so it arms no retry behind it.
  const live = useRef(true);
  useEffect(() => {
    live.current = true;
    const waits = retries.current;
    const tells = owed.current;
    return () => {
      live.current = false;
      for (const one of waits.values()) one.reset();
      if (owing.current) clearTimeout(owing.current);
      for (const doc of tells.values()) telling.current(doc);
      tells.clear();
    };
  }, []);

  const stalled = useCallback(
    (id: string) => {
      noteTrouble("saveStuck", id).catch(() => {});
      const last = lastRead.current.get(id);
      if (last) remembered(lastRead.current, id, { ...last, unwoven: true });
      if (held.current || onScreen.current !== id) return;
      held.current = { id, body: shaped.current };
      setWaiting(true);
    },
    [shaped, lastRead, onScreen, held],
  );

  const keep = useCallback(
    (id: string, text: string, mineStands = standing.current.has(id)) => {
      setSaving(true);
      if (mineStands) standing.current.add(id);
      const wrote_at = typed.current;
      const newest = () =>
        typed.current === wrote_at || (held.current?.id === id && held.current.body === text);
      const mine = savedOnce(id, text, mineStands, () => lastRead.current.get(id))
        .then(({ fresh, woven }) => {
          if (held.current?.id === id && held.current.body === text) held.current = null;
          retries.current.get(id)?.reset();
          retries.current.delete(id);
          standing.current.delete(id);
          retaken.current.delete(id);
          setHeldOpen((was) => (was === id ? null : was));
          setClashed(null);
          setSaved((many) => many + 1);
          if (fresh) {
            remembered(lastRead.current, id, { text, print: fresh.print });
            tellKept(fresh, id);
            return;
          }
          if (!woven) return;
          tellKept({ id, title: woven.title }, id);
          // Words typed since are woven by their own save, over what this one sent; the old print keeps it refused.
          if (typed.current !== wrote_at) {
            const was = lastRead.current.get(id)?.print;
            remembered(lastRead.current, id, { text, print: was, unwoven: true });
            return;
          }
          remembered(lastRead.current, id, { text: woven.body, print: woven.print });
          if (onScreen.current === id && woven.body !== text) onWoven(text, woven.body);
        })
        .catch((e) => {
          const code = codeOf(e);
          if (code === "saveLate") {
            if (lateFromCore(e) && newest()) retaking.current(id, text, mineStands);
            return;
          }
          if (refusedForGood(e) && held.current?.id === id && held.current.body === text)
            held.current = null;
          // Words refused for good are kept aside, and that bar, not the clash, says where they are.
          if (refusedForGood(e)) {
            standing.current.delete(id);
            setClashed(null);
          }
          if (code === "documentHeldOpen") {
            setHeldOpen(id);
            // «Mine» was already chosen: the file being held is all that is left to say.
            if (mineStands) setClashed(null);
            if (live.current) retryOf(id).later();
          } else if (stuck(e)) stalled(id);
          else if (code === "documentMoved" && onScreen.current === id) {
            // Each refused autosave tells the same clash again: the bar stays put and only its words move.
            const now = { torn: (e as Clash).torn, front: (e as Clash).front };
            setClashed((was) => (sameClash(was, now) ? was : now));
          } else onError(saidPlainly(e));
        })
        .finally(() => {
          setSaving(false);
          if (held.current) return;
          setWaiting(false);
          if (!missed.current) return;
          missed.current = false;
          setBehind((many) => many + 1);
        });
      return mine;
    },
    [
      typed,
      lastRead,
      onScreen,
      held,
      missed,
      setClashed,
      onError,
      tellKept,
      onWoven,
      stalled,
      retryOf,
    ],
  );

  // A window reloaded while the core stayed lost its turn: it takes it again once; a second refusal is told.
  retaking.current = (id: string, text: string, mineStands: boolean) => {
    if (onScreen.current === id && !held.current) held.current = { id, body: text };
    if (retaken.current.has(id)) {
      noteTrouble("saveLate", id).catch(() => {});
      onError(t("saveTurnLost"));
      return;
    }
    retaken.current.add(id);
    docOpened(id, turned())
      .then(() => keep(id, text, mineStands))
      .catch((e) => onError(saidPlainly(e)));
  };

  const flush = useCallback(() => {
    pace.stop();
    const waiting = held.current;
    if (!waiting) return undefined;
    onSent(waiting.body);
    return keep(waiting.id, waiting.body);
  }, [keep, onSent, pace, held]);
  flushing.current = flush;

  const again = useCallback(() => {
    const sent = flush();
    for (const [id, words] of allUnanswered()) {
      if (id !== held.current?.id) keep(id, words);
    }
    return sent;
  }, [flush, keep, held]);

  resending.current = (id: string) => {
    if (held.current?.id === id) return flush();
    const words = unansweredWords(id);
    if (words !== undefined) return keep(id, words);
  };

  const hold = useCallback(
    (one: Held) => {
      held.current = one;
      setWaiting(true);
      pace.soon();
    },
    [pace, held],
  );

  const drop = useCallback(() => {
    pace.stop();
    const dropped = held.current?.id;
    held.current = null;
    setWaiting(false);
    if (dropped === undefined) return;
    standing.current.delete(dropped);
    retaken.current.delete(dropped);
    retries.current.get(dropped)?.reset();
    setHeldOpen((was) => (was === dropped ? null : was));
  }, [pace, held]);

  const mineStands = useCallback(
    (file: string) => {
      pace.stop();
      const waiting = held.current;
      keep(file, waiting?.id === file ? waiting.body : shaped.current, true);
    },
    [keep, pace, shaped, held],
  );

  return {
    saving,
    waiting,
    setWaiting,
    heldOpen,
    saved,
    behind,
    keep,
    flush,
    again,
    hold,
    drop,
    mineStands,
  };
}
