import { StateEffect, StateField } from "@codemirror/state";
import { Decoration, type DecorationSet, EditorView } from "@codemirror/view";

// The words an agent just wrote into an open editor, marked so they are seen, then let go.
export const markLately = StateEffect.define<{ from: number; to: number } | null>();

const fresh = Decoration.mark({ class: "cm-lately" });

export const latelyMarked = StateField.define<DecorationSet>({
  create: () => Decoration.none,
  update: (was, done) => {
    let now = was.map(done.changes);
    for (const one of done.effects) {
      if (!one.is(markLately)) continue;
      now =
        one.value && one.value.to > one.value.from
          ? Decoration.set(fresh.range(one.value.from, one.value.to))
          : Decoration.none;
    }
    return now;
  },
  provide: (field) => EditorView.decorations.from(field),
});
