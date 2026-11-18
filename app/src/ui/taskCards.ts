import { useEffect, useState } from "react";
import type { TaskCarded } from "tisty-editor/read";
import { type Cited, taskCards } from "../citing";
import { dateOf, whenLabel } from "../format";
import { fill } from "../locales";
import { TASK } from "../markdown";

const named = new RegExp(`${TASK.replace("/", "\\/")}([0-9A-Za-z]+)`, "g");

export const citedIn = (text: string): string[] =>
  [...new Set(Array.from(text.matchAll(named), (found) => found[1].toUpperCase()))].sort(
    (one, other) => one.localeCompare(other),
  );

const closedOn = (iso?: string): string | undefined => {
  const at = iso ? Date.parse(iso) : Number.NaN;
  return Number.isNaN(at) ? undefined : fill("taskCardClosedOn", dateOf(at / 1000));
};

// A part on its own whole's page sits in the whole's list, so naming the list there says nothing.
const aboutOf = (one: Cited, here?: string): string | undefined => {
  const list = one.part_of && one.part_of === here ? undefined : one.list;
  const days =
    one.status === "open"
      ? [
          one.date && whenLabel(one.date),
          one.deadline && fill("taskCardDue", whenLabel(one.deadline)),
        ]
      : [closedOn(one.completed_at)];
  return [list, ...days].filter(Boolean).join(" · ") || undefined;
};

export const cardOf = (one: Cited | undefined, here?: string): TaskCarded => {
  if (!one) return { kind: "gone" };
  const shown = { title: one.title, about: aboutOf(one, here), steps: one.steps, parts: one.parts };
  return one.status === "open"
    ? { kind: "open", ...shown }
    : { kind: "closed", dropped: one.status === "dropped", ...shown };
};

type Known = { readonly asked: string; readonly cited: ReadonlyMap<string, Cited> };

const NONE: Known = { asked: "", cited: new Map() };

// Asked again whenever the window reloads its tasks, so a card never shows what the task no longer says.
export const useTaskCards = (text: string, heard: unknown, here?: string) => {
  const ids = citedIn(text).join(",");
  const [known, setKnown] = useState<Known>(NONE);
  useEffect(() => {
    if (!ids) return setKnown((was) => (was.asked === "" ? was : NONE));
    let live = true;
    taskCards(ids.split(","))
      .then((all) => {
        const cited = Array.isArray(all) ? all : [];
        if (live) setKnown({ asked: ids, cited: new Map(cited.map((one) => [one.id, one])) });
      })
      .catch(() => {});
    return () => {
      live = false;
    };
  }, [ids, heard]);
  const asked = known.asked.split(",");
  const task = (id: string): TaskCarded => {
    const key = id.toUpperCase();
    if (!asked.includes(key)) return { kind: "waiting" };
    return cardOf(known.cited.get(key), here);
  };
  const said = JSON.stringify([known.asked, ...known.cited.values()]);
  return { task, said };
};
