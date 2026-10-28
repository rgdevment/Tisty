import { useMemo, useState } from "react";
import { kindOf } from "./framing";
import { readSchema, type Schema, writeSchema } from "./schema";
import { type Altered, type Picked, relationsOf, tableOf } from "./scheming";

export interface Scheming {
  readonly schema: Schema;
  readonly picked: Picked;
  readonly change: (made: Altered) => void;
  readonly pick: (picked: Picked) => void;
}

// What makes a choice that one, whatever place it has: a table's name, a column's, a relation's ends and words.
const signOf = (schema: Schema, picked: Picked): string | null => {
  if (!picked) return null;
  if (picked.kind === "table") return tableOf(schema, picked.name) ? picked.name : null;
  if (picked.kind === "column") {
    const column = tableOf(schema, picked.table)?.columns[picked.index];
    return column ? `${picked.table}\n${column.name}` : null;
  }
  const relation = relationsOf(schema)[picked.index];
  return relation ? `${relation.from}\n${relation.to}\n${relation.label}` : null;
};

// Text typed by hand can move what was chosen; it is found again by what it is, or nothing is chosen.
const foundAgain = (schema: Schema, picked: Picked, sign: string | null): Picked => {
  if (!picked || sign === null) return null;
  if (signOf(schema, picked) === sign) return picked;
  if (picked.kind === "table") return null;
  if (picked.kind === "column") {
    const index = tableOf(schema, picked.table)?.columns.findIndex(
      (one) => `${picked.table}\n${one.name}` === sign,
    );
    return index !== undefined && index >= 0 ? { ...picked, index } : null;
  }
  const index = relationsOf(schema).findIndex(
    (one) => `${one.from}\n${one.to}\n${one.label}` === sign,
  );
  return index >= 0 ? { kind: "relation", index } : null;
};

/** The text read as a schema, while it is one the window can keep; every change is written back into the text. */
export const useSchema = (text: string, setText: (text: string) => void): Scheming | null => {
  const schema = useMemo(() => (kindOf(text) === "erDiagram" ? readSchema(text) : null), [text]);
  const [chosen, setChosen] = useState<{ picked: Picked; sign: string | null }>({
    picked: null,
    sign: null,
  });
  if (!schema) return null;
  const keep = (into: Schema, picked: Picked) => setChosen({ picked, sign: signOf(into, picked) });
  return {
    schema,
    picked: foundAgain(schema, chosen.picked, chosen.sign),
    change: (made) => {
      if (made.schema !== schema) setText(writeSchema(made.schema));
      keep(made.schema, made.picked);
    },
    pick: (picked) => keep(schema, picked),
  };
};
