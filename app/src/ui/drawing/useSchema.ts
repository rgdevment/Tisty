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

// Text typed by hand can take away what was chosen; then nothing is.
const stillThere = (schema: Schema, picked: Picked): Picked => {
  if (!picked) return null;
  if (picked.kind === "table") return tableOf(schema, picked.name) ? picked : null;
  if (picked.kind === "column")
    return tableOf(schema, picked.table)?.columns[picked.index] ? picked : null;
  return relationsOf(schema)[picked.index] ? picked : null;
};

/** The text read as a schema, while it is one the window can keep; every change is written back into the text. */
export const useSchema = (text: string, setText: (text: string) => void): Scheming | null => {
  const schema = useMemo(() => (kindOf(text) === "erDiagram" ? readSchema(text) : null), [text]);
  const [picked, setPicked] = useState<Picked>(null);
  if (!schema) return null;
  return {
    schema,
    picked: stillThere(schema, picked),
    change: (made) => {
      if (made.schema !== schema) setText(writeSchema(made.schema));
      setPicked(made.picked);
    },
    pick: setPicked,
  };
};
