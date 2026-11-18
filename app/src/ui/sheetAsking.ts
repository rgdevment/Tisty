import { type Asking, stagesIn } from "tisty-editor/edit";

// Every field the window asks with reaches the editor, so a new one is never left behind on the way.
export const editorAsking = (
  asking: Asking | undefined,
  failed: (problem: unknown) => void,
  text: () => string,
): Asking => ({
  ...asking,
  kinds: asking?.kinds ?? [],
  failed,
  ask: (kind, at, put, first, before) => asking?.ask(kind, at, put, first, before),
  steps: () => stagesIn(text()),
});
