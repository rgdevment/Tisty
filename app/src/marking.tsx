import { useEffect, useRef, useState } from "react";
import { complete, owed, type Task } from "./core";
import { fill } from "./locales";
import { saidPlainly } from "./refusal";
import Owed from "./ui/Owed";
import type { Chosen } from "./views";

interface Hands {
  chosen: Chosen;
  act: (work: Promise<Task>) => void;
  say: (words: string) => void;
  setError: (text: string | null) => void;
  lookForAStar: () => void;
  cheer?: (task: Task) => void;
}

export function useMarking({ chosen, act, say, setError, lookForAStar, cheer }: Hands) {
  const [asking, setAsking] = useState<{
    id: string;
    title: string;
    days: string[];
    summary?: string;
  } | null>(null);
  const asked = useRef(0);

  useEffect(() => {
    /// A slow answer must not open a strip over the view the person moved on to.
    asked.current += 1;
    setAsking(null);
  }, [chosen]);

  const marking = (id: string, title: string, summary?: string) => {
    setError(null);
    const mine = ++asked.current;
    owed(id)
      .then((days) => {
        if (!days.length) {
          say(fill("saidDone", title));
          const done = complete(id, undefined, summary);
          act(done);
          done
            .then((task) => {
              if (!task.part_of && !task.repeat) cheer?.(task);
            })
            .catch(() => {});
          lookForAStar();
          return;
        }
        // A slow answer must not open a strip over the task the person moved on to.
        if (mine === asked.current) setAsking({ id, title, days, summary });
      })
      .catch((e) => setError(saidPlainly(e)));
  };

  const strip = asking ? (
    <Owed
      days={asking.days}
      onConfirm={(days) => {
        say(fill("saidDone", asking.title));
        act(complete(asking.id, days, asking.summary));
        setAsking(null);
        lookForAStar();
      }}
    />
  ) : null;

  return { asking, marking, strip };
}
