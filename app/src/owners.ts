import { useEffect, useState } from "react";
import type { Task, Whole } from "./core";
import { stored } from "./stored";
import { agentNamed } from "./who";

export type Owner = "person" | "agent";

export const ownerOf = (task: Task): Owner =>
  agentNamed(task.created_by) && !task.for_person ? "agent" : "person";

export const confirmable = (task: Task): boolean =>
  task.status === "open" && Boolean(task.resolved) && !task.resolved?.drop && !task.repeat;

const PROOF_LINES = 2;

export const proofOf = (task: Task): string[] => {
  const said = task.resolved;
  if (!said) return [];
  const body = task.log?.find((entry) => entry.id === said.entry)?.body ?? "";
  return body
    .split("\n")
    .map((line) =>
      line
        .replace(/^\s*(#{1,6}|[-*+]|\d+[.)]|>)\s+/, "")
        .replace(/[*_`]/g, "")
        .trim(),
    )
    .filter(Boolean)
    .slice(0, PROOF_LINES);
};

export type Companion = { key: "waitsOn" | "yourPartOf"; title: string };

export const companionOf = (
  task: Task,
  wholes: Record<string, Whole> | undefined,
): Companion | undefined => {
  if (!task.for_person || !task.part_of) return undefined;
  const whole = wholes?.[task.part_of];
  if (!whole) return undefined;
  return { key: whole.working ? "waitsOn" : "yourPartOf", title: whole.title };
};

const met = stored<Partial<Record<Owner, boolean>>>("tisty.owners", (said) => ({
  person: said.person === true,
  agent: said.agent === true,
}));

export const useFirstMeeting = (owner: Owner): boolean => {
  const [first] = useState(() => !met.current()[owner]);
  useEffect(() => {
    if (!met.current()[owner]) met.put({ ...met.current(), [owner]: true });
  }, [owner]);
  return first;
};
