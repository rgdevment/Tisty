import { useEffect, useId, useState } from "react";
import { useAttended } from "../attended";
import { search, type Task } from "../core";
import { t } from "../locales";
import { optionOf, PickList, usePicker } from "./Picker";

const leftOpen = () => {};

const SETTLES = 150;
const SHOWN = 30;

// Typing asks on every key; only the answer to the words still written is shown.
const useFound = (word: string, onError?: (problem: unknown) => void): Task[] | null => {
  const [found, setFound] = useState<{ word: string; tasks: Task[] } | null>(null);
  useEffect(() => {
    if (!word.trim()) return;
    let live = true;
    const asking = window.setTimeout(() => {
      search(word, "either")
        .then((all) => {
          if (live) setFound({ word, tasks: all.tasks.slice(0, SHOWN) });
        })
        .catch((problem) => {
          if (live) onError?.(problem);
        });
    }, SETTLES);
    return () => {
      live = false;
      window.clearTimeout(asking);
    };
  }, [word, onError]);
  if (!word.trim()) return [];
  return found?.word === word ? found.tasks : null;
};

export default function TaskPick({
  onPick,
  onError,
}: {
  onPick: (task: Task) => void;
  onError?: (problem: unknown) => void;
}) {
  const [word, setWord] = useState("");
  const found = useFound(word, onError);
  const rows = (found ?? []).map((task) => ({
    key: task.id,
    glyph: task.status === "open" ? "○" : "✓",
    label: task.title,
    detail: task.status === "open" ? undefined : t("paletteArchived"),
    onPick: () => onPick(task),
  }));
  const { on, setAt, keyed } = usePicker(rows, word, leftOpen);
  const list = useId();
  const held = useAttended<HTMLInputElement>();

  return (
    <>
      <input
        ref={held}
        value={word}
        role="combobox"
        aria-expanded="true"
        aria-controls={list}
        aria-activedescendant={optionOf(list, on)}
        aria-label={t("pickATaskToCite")}
        placeholder={t("pickATaskToCite")}
        onChange={(e) => setWord(e.target.value)}
        onKeyDown={(e) => {
          if (e.key !== "Escape" && !e.nativeEvent.isComposing) keyed(e);
        }}
        className="mb-1 w-full rounded-md bg-hover px-2.5 py-1.5 outline-none placeholder:text-faint"
      />
      <div className="scroller max-h-[168px]">
        {!word.trim() && <p className="px-2.5 py-1.5 text-faint">{t("pickATaskHint")}</p>}
        {word.trim() && found === null && (
          <p className="px-2.5 py-1.5 text-faint">{t("opening")}</p>
        )}
        {word.trim() && found?.length === 0 && (
          <p className="px-2.5 py-1.5 text-faint">{t("noneHere")}</p>
        )}
        {rows.length > 0 && (
          <PickList id={list} label={t("paletteTasks")} rows={rows} on={on} onHover={setAt} />
        )}
      </div>
    </>
  );
}
