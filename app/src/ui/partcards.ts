import { useEffect, useState } from "react";
import { type Task, taskOf } from "../core";
import { whenLabel } from "../format";
import { TASK } from "../markdown";
import type { PartCarded } from "./partcarding";

const named = new RegExp(`${TASK.replace("/", "\\/")}([0-9A-Za-z]+)`, "g");

const cardOf = (whole: string, known: ReadonlyMap<string, Task | null> | null) => (id: string) => {
  if (!known) return { kind: "waiting" } satisfies PartCarded;
  const one = known.get(id);
  if (!one || one.part_of !== whole) return { kind: "loose" } satisfies PartCarded;
  if (one.status !== "open")
    return {
      kind: "closed",
      title: one.title,
      dropped: one.status === "dropped",
    } satisfies PartCarded;
  const day = one.date ?? one.deadline;
  return {
    kind: "open",
    title: one.title,
    done: one.volume?.steps_done ?? 0,
    steps: one.volume?.steps ?? 0,
    when: day ? whenLabel(day) : undefined,
  } satisfies PartCarded;
};

const saidOf = (one: Task | null): string =>
  one
    ? [
        one.id,
        one.status,
        one.title,
        one.part_of,
        one.volume?.steps_done,
        one.volume?.steps,
        JSON.stringify(one.date ?? one.deadline ?? null),
      ].join(":")
    : "-";

// Each card asks for its own task, so a part dropped or hidden still reads as what it is.
export const usePartCards = (whole: Task, text: string) => {
  const ids = [...new Set(Array.from(text.matchAll(named), (found) => found[1]))].sort().join(",");
  const [known, setKnown] = useState<ReadonlyMap<string, Task | null> | null>(null);
  useEffect(() => {
    let live = true;
    const asked = ids ? ids.split(",") : [];
    Promise.all(
      asked.map((id) =>
        taskOf(id).then(
          (one) => [id, one] as const,
          () => [id, null] as const,
        ),
      ),
    ).then((all) => {
      if (live) setKnown(new Map(all));
    });
    return () => {
      live = false;
    };
  }, [ids, whole]);
  const said = known ? Array.from(known.values(), saidOf).join("|") : "";
  return { part: cardOf(whole.id, known), said };
};
