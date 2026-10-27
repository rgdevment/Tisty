import { useMemo, useState } from "react";
import { kindOf } from "./framing";
import { type Changed, ideaAt, type Path, pathOfWords } from "./minding";
import { type Idea, type Mind, readMind, writeMind } from "./mindmap";

export interface Minding {
  readonly mind: Mind;
  readonly chosen: Path;
  readonly idea: Idea;
  readonly change: (made: Changed) => void;
  readonly choose: (path: Path) => void;
}

interface Chosen {
  readonly path: Path;
  readonly words: string;
}

/** The text read as a mind map, while it is one the window can keep; every change is written back into the text. */
export const useMind = (text: string, setText: (text: string) => void): Minding | null => {
  const mind = useMemo(() => (kindOf(text) === "mindmap" ? readMind(text) : null), [text]);
  const [chosen, setChosen] = useState<Chosen>({ path: [], words: "" });
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
    change: (made) => {
      if (made.mind !== mind) setText(writeMind(made.mind));
      keep(made.mind, made.chosen);
    },
    choose: (path) => keep(mind, path),
  };
};
