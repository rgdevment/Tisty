import { useMemo, useState } from "react";
import { kindOf } from "./framing";
import { type Changed, ideaAt, type Path } from "./minding";
import { type Idea, type Mind, readMind, writeMind } from "./mindmap";

export interface Minding {
  readonly mind: Mind;
  readonly chosen: Path;
  readonly idea: Idea;
  readonly change: (made: Changed) => void;
  readonly choose: (path: Path) => void;
}

/** The text read as a mind map, while it is one the window can keep; every change is written back into the text. */
export const useMind = (text: string, setText: (text: string) => void): Minding | null => {
  const mind = useMemo(() => (kindOf(text) === "mindmap" ? readMind(text) : null), [text]);
  const [chosen, setChosen] = useState<Path>([]);
  if (!mind) return null;
  // Text typed by hand can take the chosen idea away; the root is always there.
  const here = ideaAt(mind, chosen) ? chosen : [];
  return {
    mind,
    chosen: here,
    idea: ideaAt(mind, here) ?? mind.root,
    change: (made) => {
      if (made.mind !== mind) setText(writeMind(made.mind));
      setChosen(made.chosen);
    },
    choose: setChosen,
  };
};
