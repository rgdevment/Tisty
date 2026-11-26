import { colonsKept } from "./pageColons";
import { type Edit, landedAfter, type Span } from "./pageEdits";
import type { Laid } from "./pageLate";
import type { Laying } from "./pageLaying";
import { fenceBreakAt } from "./pageLists";
import { replaced, replacedOverCode } from "./pageMarks";
import { type Mend, openOf, shutOf } from "./pageMends";
import type { Writer } from "./pageWriter";

export const mendedOver = (text: string, one: Span, insert: string, mend: Mend): Laid => {
  const shut = shutOf(text, one, insert, mend);
  const { edit, caret } = replacedOverCode(text, one, insert + shut);
  return {
    edit: { ...edit, insert: edit.insert + openOf(text, edit, mend) },
    caret: caret - shut.length,
  };
};

export const laidOver = (
  text: string,
  one: Span,
  insert: string,
  coded: boolean,
  mend: Mend | null = null,
): Laid => {
  if (coded) {
    const lineBreak = insert && !insert.endsWith("\n") ? fenceBreakAt(text, one.to) : "";
    return { edit: { ...one, insert: insert + lineBreak }, caret: one.from + insert.length };
  }
  const laid = mend ? mendedOver(text, one, insert, mend) : replaced(text, one, insert);
  const edit = colonsKept(text, laid.edit);
  return { edit, caret: landedAfter(laid.edit, edit, laid.caret) };
};

export type Putting = ReturnType<typeof puttingOn>;

export const puttingOn = (writer: Writer, laying: Laying) => {
  const { hands } = writer;

  const put = (wanted: Edit, at = wanted.from + wanted.insert.length, whole = false) => {
    const asked = colonsKept(hands.text(), wanted);
    laying.put(asked, landedAfter(wanted, asked, at), whole);
  };

  const putOver = (one: Span, insert: string, coded: boolean, mend: Mend | null = null) => {
    const laid = laidOver(hands.text(), one, insert, coded, mend);
    laying.put(laid.edit, laid.caret);
  };

  return { put, putOver };
};
