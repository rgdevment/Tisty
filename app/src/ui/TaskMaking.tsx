import { emit } from "@tauri-apps/api/event";
import { useEffect, useRef, useState } from "react";
import {
  type Born,
  type Counted,
  capture,
  type Edits,
  type List,
  snapshot,
  type Task,
} from "../core";
import { every, turnOf } from "../examples";
import { t } from "../locales";
import CaptureField from "./CaptureField";

type Known = { lists: List[]; tags: Counted[] };

export default function TaskMaking({
  born,
  onMaking,
  onMade,
}: {
  born: Born;
  onMaking: (making: boolean) => void;
  onMade: (task: Task) => void;
}) {
  const [known, setKnown] = useState<Known>({ lists: [], tags: [] });
  const [invite] = useState(() => turnOf(every(), Math.random()));
  const [problem, setProblem] = useState<string | null>(null);
  const asked = useRef<Promise<Task> | null>(null);

  useEffect(() => {
    let live = true;
    snapshot({ most: 0 })
      .then((fresh) => {
        if (live) setKnown({ lists: fresh.lists, tags: fresh.tags });
      })
      .catch(() => {});
    return () => {
      live = false;
    };
  }, []);

  const made = (text: string, edits: Edits): Promise<Task> => {
    if (asked.current) return asked.current;
    if (!text.trim()) return Promise.reject({ code: "untitled" });
    setProblem(null);
    onMaking(true);
    const asking = capture(text, {}, edits, born)
      .then((task) => {
        void emit("captured");
        onMade(task);
        return task;
      })
      .finally(() => {
        asked.current = null;
        onMaking(false);
      });
    asked.current = asking;
    return asking;
  };

  return (
    <section aria-label={t("newTaskHere")} className="p-1.5">
      <CaptureField
        invite={invite}
        lists={known.lists}
        tags={known.tags}
        onCapture={made}
        onError={setProblem}
      />
      {problem && (
        <p role="alert" className="px-1 pt-1.5 text-[11.5px] text-urgent">
          {problem}
        </p>
      )}
    </section>
  );
}
