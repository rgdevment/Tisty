import { listen } from "@tauri-apps/api/event";
import { type MutableRefObject, useEffect, useRef } from "react";
import { carrying } from "./carrying";
import { heard, play } from "./chime";
import { type Afoot, attach, doorDue, type Filed, parted, syncState } from "./core";
import { decideAll } from "./deciding";
import { handTo, whenFilesLand } from "./dropped";
import { fill, t } from "./locales";
import { saidPlainly } from "./refusal";
import { settled } from "./saving";
import type { Chosen } from "./views";

const TURNS_OVER = 60 * 1000;

type Setter<T> = (next: T | ((was: T) => T)) => void;

interface Hands {
  latest: MutableRefObject<() => void>;
  lookPapers: () => void;
  docs: Filed[];
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
  setOffering: Setter<boolean>;
  setAfoot: Setter<Afoot | null>;
}

export function useListening({
  latest,
  lookPapers,
  docs,
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
  }, [latest, lookPapers]);

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
  const papersNow = useRef(docs);
  papersNow.current = docs;
  useEffect(() => {
    const carrier = carrying(
      () => {
        setCarried((was) => was + 1);
        latest.current();
        lookPapers();
      },
      (ids) => {
        decideAll(ids)
          .then((shut) => {
            if (!shut.length) return;
            const named = shut
              .map((one) => papersNow.current.find((doc) => doc.file === one))
              .map((one) => `«${one?.title?.trim() || t("untitledDoc")}»`)
              .join(", ");
            setError(fill("someLockedAtOdds", named));
          })
          .catch((problem) => setError(saidPlainly(problem)))
          .finally(() => latest.current());
      },
      (why) => {
        const now = why?.why ?? null;
        if (now === wasAwry.current) return;
        wasAwry.current = now;
        if (why?.why === "broke" || why?.why === "amiss") {
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
      setOffering(false);
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
    setOffering(false);
  }, [chosen, setStarring, setOffering]);

  useEffect(() => {
    doorDue()
      .then((due) => setOffering((was) => was || due))
      .catch(() => {});
  }, [greeted, setOffering]);

  useEffect(
    () =>
      whenFilesLand((target, paths, at) => {
        setError(null);
        Promise.all(paths.map((one) => attach(one, undefined, where.current.named === "docs")))
          .then((written) => {
            const put = handTo(target, written.join("\n\n"), at);
            if (!put) setError(t("attachmentLost"));
          })
          .catch((e) => setError(saidPlainly(e)));
      }),
    [setError],
  );
}
