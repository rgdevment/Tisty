import { listen } from "@tauri-apps/api/event";
import { type MutableRefObject, useEffect, useRef } from "react";
import { carrying } from "./carrying";
import { heard, play } from "./chime";
import { type Afoot, attach, type DoorDue, doorDue, parted, syncState } from "./core";
import { decideAll, putOffSaid } from "./deciding";
import { CALENDAR, handTo, whenFilesLand } from "./dropped";
import { keptAll } from "./kept";
import { t } from "./locales";
import { saidPlainly } from "./refusal";
import { settled } from "./saving";
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
  setCarried: Setter<number>;
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
  setCarried,
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
  useEffect(() => {
    const again = () => {
      latest.current();
      lookPapers();
      doorDue()
        .then((due) => {
          if (due?.why === "knocked") setOffering(due);
        })
        .catch(() => {});
    };
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
      () => {
        setCarried((was) => was + 1);
        latest.current();
        lookPapers();
      },
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
  }, [noted, setCarried, latest, lookPapers, carries, setError]);

  useEffect(() => {
    syncState()
      .then((state) => setGreet(!state.asked))
      .catch(() => {});
  }, [setGreet]);

  useEffect(() => {
    const off = listen("parting", () => {
      void settled().finally(() => void parted());
    });
    return () => {
      void off.then((stop) => stop());
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
    const gone = listen("withdrawn", () => {
      setStarring(false);
      setOffering(null);
    });
    const caught = listen("captured", () => latest.current());
    // The snapshot carries no documents, so a paper written from outside would go unseen till relaunch.
    const stirred = listen("stirred", () => {
      latest.current();
      lookPapers();
      setCarried((was) => was + 1);
    });
    const landed = listen<string>("carried", (far) => {
      latest.current();
      if (far.payload === "papers") lookPapers();
      setCarried((was) => was + 1);
    });
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
      stirred.then((off) => off()).catch(() => {});
      landed.then((off) => off()).catch(() => {});
      sound.then((off) => off()).catch(() => {});
      along.then((off) => off()).catch(() => {});
    };
  }, [lookPapers, setLeaving, setAfoot, setOffering, setStarring, setCarried, latest]);

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
        keptAll(
          paths.map((one) => attach(one)),
          setError,
        ).then((written) => {
          if (written && !handTo(target, written, at)) setError(t("attachmentLost"));
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
