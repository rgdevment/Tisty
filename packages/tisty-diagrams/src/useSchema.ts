import { useMemo, useState } from "react";
import { kindOf } from "./framing";
import { readSchema, type Schema, writeSchema } from "./schema";
import { type Altered, type Picked, relationsOf, tableOf } from "./scheming";
import { useSteps } from "./useSteps";

export interface Scheming {
  readonly schema: Schema;
  readonly picked: Picked;
  readonly change: (made: Altered) => void;
  readonly pick: (picked: Picked) => void;
  readonly undo: (() => void) | null;
  readonly redo: (() => void) | null;
}

// What makes a choice that one, whatever place it has: a table's name, a column's, a relation's ends and words.
const signOf = (schema: Schema, picked: Picked): string | null => {
  if (!picked) return null;
  if (picked.kind === "table") return tableOf(schema, picked.name) ? picked.name : null;
  if (picked.kind === "tables")
    return picked.names.every((name) => tableOf(schema, name))
      ? `tables\n${picked.names.join("\n")}`
      : null;
  if (picked.kind === "column") {
    const column = tableOf(schema, picked.table)?.columns[picked.index];
    return column ? `${picked.table}\n${column.name}` : null;
  }
  return relationSigns(schema)[picked.index] ?? null;
};

// Two relations alike in ends and words are told apart by which of them it is.
const relationSigns = (schema: Schema): string[] => {
  const seen = new Map<string, number>();
  return relationsOf(schema).map((one) => {
    const said = `${one.from}\n${one.to}\n${one.label}`;
    const nth = seen.get(said) ?? 0;
    seen.set(said, nth + 1);
    return `${said}\n${nth}`;
  });
};

// Text typed by hand can move what was chosen; it is found again by what it is, or nothing is chosen.
const foundAgain = (schema: Schema, picked: Picked, sign: string | null): Picked => {
  if (!picked || sign === null) return null;
  if (signOf(schema, picked) === sign) return picked;
  if (picked.kind === "table" || picked.kind === "tables") return null;
  if (picked.kind === "column") {
    const index = tableOf(schema, picked.table)?.columns.findIndex(
      (one) => `${picked.table}\n${one.name}` === sign,
    );
    return index !== undefined && index >= 0 ? { ...picked, index } : null;
  }
  const index = relationSigns(schema).indexOf(sign);
  return index >= 0 ? { kind: "relation", index } : null;
};

/** The text read as a schema, while it is one the window can keep; every change is written back into the text. */
export const useSchema = (text: string, setText: (text: string) => void): Scheming | null => {
  const schema = useMemo(() => (kindOf(text) === "erDiagram" ? readSchema(text) : null), [text]);
  const [chosen, setChosen] = useState<{ picked: Picked; sign: string | null }>({
    picked: null,
    sign: null,
  });
  const { stepped, undo, redo } = useSteps(text, setText);
  if (!schema) return null;
  const keep = (into: Schema, picked: Picked) => setChosen({ picked, sign: signOf(into, picked) });
  return {
    schema,
    picked: foundAgain(schema, chosen.picked, chosen.sign),
    change: (made) => {
      if (made.schema !== schema) stepped(writeSchema(made.schema));
      keep(made.schema, made.picked);
    },
    pick: (picked) => keep(schema, picked),
    undo,
    redo,
  };
};
