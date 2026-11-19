import { listen } from "@tauri-apps/api/event";
import { useEffect, useRef } from "react";
import { type Neuron, OPEN_ASKED } from "../brainsty";
import { taskOf } from "../core";
import { type Desk, seeing, useDesk } from "../desk";
import { saidPlainly } from "../refusal";

type Hands = Pick<Desk, "setChosen" | "choose" | "opening" | "setError">;

export const openedHere = (one: Neuron, { setChosen, choose, opening, setError }: Hands) => {
  if (one.kind === "doc" || one.kind === "page") setChosen({ named: "docs", doc: one.id });
  else if (one.kind === "tag") choose({ named: "tags", tags: [one.title] });
  else if (one.kind === "task")
    taskOf(one.id).then(
      (task) => {
        setChosen(seeing(task));
        opening(task);
      },
      (e: unknown) => setError(saidPlainly(e)),
    );
};

export default function BrainstyAsked() {
  const desk = useDesk();
  const hands = useRef<Hands>(desk);
  hands.current = desk;
  useEffect(() => {
    let drop: (() => void) | undefined;
    let gone = false;
    listen<Neuron>(OPEN_ASKED, ({ payload }) => openedHere(payload, hands.current))
      .then((off) => {
        if (gone) off();
        else drop = off;
      })
      .catch(() => undefined);
    return () => {
      gone = true;
      drop?.();
    };
  }, []);
  return null;
}
