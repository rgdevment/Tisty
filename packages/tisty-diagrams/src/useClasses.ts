import { useMemo, useState } from "react";
import { type Classes, readClasses, writeClasses } from "./classes";
import type { ClassAltered, ClassPicked } from "./classing";
import { kindOf } from "./framing";

export interface Classing {
  readonly diagram: Classes;
  readonly picked: ClassPicked;
  readonly change: (made: ClassAltered) => void;
  readonly pick: (picked: ClassPicked) => void;
}

// What makes a choice that one whatever place it has: a class's name, a member's words, a relation's ends.
const signOf = (diagram: Classes, picked: ClassPicked): string | null => {
  if (!picked) return null;
  if (picked.kind === "class")
    return diagram.classes.some((one) => one.name === picked.name) ? picked.name : null;
  if (picked.kind === "member") {
    const member = diagram.classes.find((one) => one.name === picked.name)?.members[picked.index];
    return member ? `${picked.name}\n${member.text}` : null;
  }
  const link = diagram.links[picked.index];
  return link ? `${link.left}\n${link.right}\n${link.arrow}` : null;
};

// Text typed by hand can move what was chosen; it is found again by what it is, or nothing is chosen.
const foundAgain = (diagram: Classes, picked: ClassPicked, sign: string | null): ClassPicked => {
  if (!picked || sign === null) return null;
  if (signOf(diagram, picked) === sign) return picked;
  if (picked.kind === "member") {
    const index = diagram.classes
      .find((one) => one.name === picked.name)
      ?.members.findIndex((one) => `${picked.name}\n${one.text}` === sign);
    return index !== undefined && index >= 0 ? { ...picked, index } : null;
  }
  if (picked.kind !== "link") return null;
  const index = diagram.links.findIndex(
    (one) => `${one.left}\n${one.right}\n${one.arrow}` === sign,
  );
  return index >= 0 ? { kind: "link", index } : null;
};

/** The text read as a class diagram, while it is one the window can keep; every change is written back into the text. */
export const useClasses = (text: string, setText: (text: string) => void): Classing | null => {
  const diagram = useMemo(
    () => (kindOf(text) === "classDiagram" ? readClasses(text) : null),
    [text],
  );
  const [chosen, setChosen] = useState<{ picked: ClassPicked; sign: string | null }>({
    picked: null,
    sign: null,
  });
  if (!diagram) return null;
  const keep = (into: Classes, picked: ClassPicked) =>
    setChosen({ picked, sign: signOf(into, picked) });
  return {
    diagram,
    picked: foundAgain(diagram, chosen.picked, chosen.sign),
    change: (made) => {
      const written = writeClasses({ ...diagram, body: made.body });
      if (written !== text) setText(written);
      keep(readClasses(written) ?? diagram, made.picked);
    },
    pick: (picked) => keep(diagram, picked),
  };
};
