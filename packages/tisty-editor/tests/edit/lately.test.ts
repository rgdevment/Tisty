import { EditorState } from "@codemirror/state";
import { describe, expect, it } from "vitest";
import { latelyMarked, markLately } from "../../src/edit/lately";

const marks = (state: EditorState) => {
  const found: [number, number][] = [];
  state.field(latelyMarked).between(0, state.doc.length, (from, to) => {
    found.push([from, to]);
  });
  return found;
};

describe("the words an agent just wrote into an open editor", () => {
  it("marks what came in and follows it as the person writes before it", () => {
    let state = EditorState.create({ doc: "uno dos", extensions: [latelyMarked] });
    state = state.update({
      changes: { from: 7, insert: " tres" },
      effects: markLately.of({ from: 7, to: 12 }),
    }).state;
    expect(marks(state)).toEqual([[7, 12]]);
    state = state.update({ changes: { from: 0, insert: "cero " } }).state;
    expect(marks(state)).toEqual([[12, 17]]);
  });

  it("lets the mark go, and marks nothing for words only taken out", () => {
    let state = EditorState.create({ doc: "uno dos", extensions: [latelyMarked] });
    state = state.update({ effects: markLately.of({ from: 0, to: 3 }) }).state;
    state = state.update({ effects: markLately.of(null) }).state;
    expect(marks(state)).toEqual([]);
    state = state.update({ effects: markLately.of({ from: 2, to: 2 }) }).state;
    expect(marks(state)).toEqual([]);
  });
});
