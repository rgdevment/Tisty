import type { Asked } from "tisty-editor/edit";

/** Where a document lives, as far as what may be put in it goes. */
export type Lodged = {
  /** Inside the folder of Agents, at any depth. */
  readonly agents: boolean;
  /** One of the diary's days. */
  readonly diary: boolean;
  /** An agent's own file, kept byte for byte for the agent that reads it. */
  readonly agentFile: boolean;
};

/** What a folder lets be made or put in a document of it. */
export type Allows = {
  /** A new document or page made from inside it. */
  readonly newDocs: boolean;
  /** A card of a task, a document, a page or a diagram, which only Tisty can draw. */
  readonly cards: boolean;
};

// The one place that tells what each folder allows, until the core's own rules answer it.
export const folderAllows = (lodged: Lodged): Allows => ({
  newDocs: !lodged.agents && !lodged.diary,
  cards: !lodged.agentFile,
});

const NEW_DOCS: readonly Asked[] = ["newPage", "newDoc"];
const CARDS: readonly Asked[] = [
  "page",
  "doc",
  "newPage",
  "newDoc",
  "task",
  "newTask",
  "diagram",
  "diagramFrom",
];

/** The kinds of thing still offered once what the folder allows is taken into account. */
export const allowedKinds = (kinds: readonly Asked[], allows: Allows): Asked[] =>
  kinds.filter(
    (kind) =>
      (allows.newDocs || !NEW_DOCS.includes(kind)) && (allows.cards || !CARDS.includes(kind)),
  );
