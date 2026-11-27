import { listen } from "@tauri-apps/api/event";
import { type MutableRefObject, useEffect, useRef } from "react";
import { cameBack } from "./back";
import { carrying } from "./carrying";
import { heard, play } from "./chime";
import { type Afoot, type DoorDue, doorDue, keptForTheEnd, syncState } from "./core";
import { decideAll, putOffSaid } from "./deciding";
import { departing } from "./departing";
import { fireAll } from "./dropLater";
import { CALENDAR, caughtBy, whenFilesLand } from "./dropped";
import type { Freshness } from "./freshness";
import { keptAll } from "./kept";
import { t } from "./locales";
import { gathering, type Refresh } from "./refreshing";
import { saidPlainly } from "./refusal";
import { attachedAll } from "./reshaping";
import { resentThen } from "./saving";
import { lookAgain } from "./syncSeen";
import { CALENDAR_DROPPED } from "./ui/Calendars";
import type { Chosen } from "./views";

const TURNS_OVER = 60 * 1000;

type Setter<T> = (next: T | ((was: T) => T)) => void;

interface Hands {
  latest: MutableRefObject<() => void>;
  lookPapers: () => void;
  chosen: Chosen;
  greeted: number;
  returning: string | null;
  setReturning: (at: string | null) => void;
  carried: Freshness;
  setError: (text: string | null) => void;
  noted: (text: string, fades?: number) => void;
  carries: MutableRefObject<ReturnType<typeof carrying> | null>;
  setGreet: (greet: boolean) => void;
  setLeaving: (leaving: boolean) => void;
  setStarring: Setter<boolean>;
  setOffering: Setter<DoorDue | null>;
  setAfoot: Setter<Afoot | null>;
}

export function useListening({
  latest,
  lookPapers,
  chosen,
  greeted,
  returning,
  setReturning,
  carried,
  setError,
  noted,
  carries,
  setGreet,
  setLeaving,
  setStarring,
  setOffering,
  setAfoot,
}: Hands) {
  const wasAwry = useRef<string | null>(null);
  const refreshed = useRef<(asked: Refresh) => void>(() => {});
  refreshed.current = ({ tasks, papers, fresh, seen }) => {
    if (tasks) latest.current();
    if (papers) lookPapers();
    if (fresh) carried.landed();
    if (seen) void lookAgain();
  };
  const round = useRef<ReturnType<typeof gathering> | null>(null);
  round.current ??= gathering((asked) => refreshed.current(asked));
  useEffect(() => () => round.current?.stop(), []);
  useEffect(() => {
    const again = cameBack(() => {
      latest.current();
      lookPapers();
      doorDue()
        .then((due) => {
          if (due?.why === "knocked") setOffering(due);
        })
        .catch(() => {});
    });
    const shownAgain = () => {
      if (document.visibilityState === "visible") again();
    };
    window.addEventListener("focus", again);
    document.addEventListener("visibilitychange", shownAgain);
    return () => {
      window.removeEventListener("focus", again);
      document.removeEventListener("visibilitychange", shownAgain);
    };
  }, [latest, lookPapers, setOffering]);

  useEffect(() => {
    let day = new Date().getDate();
    const turned = setInterval(() => {
      const now = new Date().getDate();
      if (now === day) return;
      day = now;
      latest.current();
    }, TURNS_OVER);
    return () => clearInterval(turned);
  }, [latest]);
  useEffect(() => {
    const carrier = carrying(
      () => round.current?.ask({ tasks: true, papers: true, fresh: true }),
      (ids) => {
        decideAll(ids)
          .then(({ said, putOff }) => {
            if (said) setError(said);
            if (putOff.length) noted(putOffSaid(putOff.length), 8000);
          })
          .catch((problem) => setError(saidPlainly(problem)))
          .finally(() => latest.current());
      },
      (why) => {
        const now = why?.why ?? null;
        if (now === wasAwry.current) return;
        wasAwry.current = now;
        if (why?.why === "broke" || why?.why === "amiss" || why?.why === "away") {
          noted(why.said, 6000);
        }
      },
    );
    carries.current = carrier;
    return () => carrier.stop();
  }, [noted, latest, lookPapers, carries, setError]);

  useEffect(() => {
    syncState()
      .then((state) => setGreet(!state.asked))
      .catch(() => {});
  }, [setGreet]);

  useEffect(() => {
    const off = [
      listen("parting", () => void departing()),
      listen<number>("ending", ({ payload }) => resentThen(() => keptForTheEnd(payload))()),
    ];
    return () => {
      for (const one of off) void one.then((stop) => stop());
    };
  }, []);

  useEffect(() => {
    if (!returning) return;
    document
      .querySelector<HTMLElement>(`[data-task="${returning}"], [data-row="${returning}"]`)
      ?.focus();
    setReturning(null);
  }, [returning, setReturning]);

  useEffect(() => {
    const stop = listen("closing", () => setLeaving(true));
    // A hidden window has nobody to undo with, so what waits to be deleted goes now instead of in the dark.
    const gone = listen("withdrawn", () => {
      fireAll();
      setStarring(false);
      setOffering(null);
    });
    const caught = listen("captured", () => latest.current());
    const jotted = listen("jotted", () => round.current?.ask({ papers: true, fresh: true }));
    // The snapshot carries no documents, so a paper written from outside would go unseen till relaunch.
    const stirred = listen("stirred", () =>
      round.current?.ask({ tasks: true, papers: true, fresh: true, seen: true }),
    );
    const landed = listen<string>("carried", (far) =>
      round.current?.ask({
        tasks: true,
        papers: far.payload === "papers",
        fresh: true,
        seen: true,
      }),
    );
    const sound = listen<unknown>("chime", (rung) => {
      if (heard(rung.payload)) play(rung.payload);
    });
    const along = listen<Afoot>("carrying", (step) => {
      setAfoot((was) => (was ? step.payload : was));
    });
    return () => {
      stop.then((off) => off()).catch(() => {});
      gone.then((off) => off()).catch(() => {});
      caught.then((off) => off()).catch(() => {});
      jotted.then((off) => off()).catch(() => {});
      stirred.then((off) => off()).catch(() => {});
      landed.then((off) => off()).catch(() => {});
      sound.then((off) => off()).catch(() => {});
      along.then((off) => off()).catch(() => {});
    };
  }, [lookPapers, setLeaving, setAfoot, setOffering, setStarring, latest]);

  const where = useRef(chosen);
  where.current = chosen;

  useEffect(() => {
    setStarring(false);
    setOffering((was) => (was?.why === "knocked" ? was : null));
  }, [chosen, setStarring, setOffering]);

  useEffect(() => {
    doorDue()
      .then((due) => setOffering((was) => (due?.why === "knocked" ? due : (was ?? due))))
      .catch(() => {});
  }, [greeted, setOffering]);

  useEffect(
    () =>
      whenFilesLand((target, paths, at) => {
        setError(null);
        const landing = caughtBy(target, at);
        attachedAll(paths)
          .then((all) => keptAll(all, setError))
          .then((written) => {
            if (written && !landing?.(written)) setError(t("attachmentLost"));
          }, setError);
      }, calendarsDropped),
    [setError],
  );
}

// Away from a document a calendar is not an attachment: it brings tasks.
const calendarsDropped = (paths: string[]) => {
  const calendars = paths.filter((one) => CALENDAR.test(one));
  if (calendars.length)
    window.dispatchEvent(new CustomEvent(CALENDAR_DROPPED, { detail: calendars }));
};
