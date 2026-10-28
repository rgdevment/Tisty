import { useMemo, useState } from "react";
import { kindOf } from "./framing";
import { type Changed, ideaAt, type Path, pathOfWords } from "./minding";
import { type Idea, type Mind, readMind, writeMind } from "./mindmap";
import { useSteps } from "./useSteps";

export interface Minding {
  readonly mind: Mind;
  readonly chosen: Path;
  readonly idea: Idea;
  // `merged` folds the change into the last one, as naming an idea just added.
  readonly change: (made: Changed, merged?: boolean) => void;
  readonly choose: (path: Path) => void;
  readonly undo: (() => void) | null;
  readonly redo: (() => void) | null;
}

interface Chosen {
  readonly path: Path;
  readonly words: string;
}

/** The text read as a mind map, while it is one the window can keep; every change is written back into the text. */
export const useMind = (text: string, setText: (text: string) => void): Minding | null => {
  const mind = useMemo(() => (kindOf(text) === "mindmap" ? readMind(text) : null), [text]);
  const [chosen, setChosen] = useState<Chosen>({ path: [], words: "" });
  const { stepped, undo, redo } = useSteps(text, setText);
  if (!mind) return null;
  // Text typed by hand can move the chosen idea; it is found again by its words, or the root stands in.
  const still = ideaAt(mind, chosen.path)?.text === chosen.words;
  const here = still ? chosen.path : (pathOfWords(mind, chosen.words) ?? []);
  const keep = (into: Mind, path: Path) =>
    setChosen({ path, words: ideaAt(into, path)?.text ?? "" });
  return {
    mind,
    chosen: here,
    idea: ideaAt(mind, here) ?? mind.root,
    change: (made, merged) => {
      if (made.mind !== mind) stepped(writeMind(made.mind), merged);
      keep(made.mind, made.chosen);
    },
    choose: (path) => keep(mind, path),
    undo,
    redo,
  };
};
