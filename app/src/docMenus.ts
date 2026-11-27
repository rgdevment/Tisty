import { open as pick } from "@tauri-apps/plugin-dialog";
import { agentsHideChoice } from "./agentsHiding";
import { movedOnceSure } from "./agentsMoving";
import {
  AGENTS_CHANGED,
  type AgentsStage,
  type AgentsState,
  agentsRoot,
  agentsStage,
  agentsState,
  agentsTurn,
  bringMoreAgents,
  inAgents,
  ofAnother,
  rescanAgents,
  syncAgents,
} from "./agentsSync";
import { asLink, asPlain } from "./copying";
import {
  DEEPEST,
  docAway,
  docCopy,
  docExport,
  docFile,
  docLock,
  docPage,
  type Filed,
  type Folded,
  folderAway,
  folderFile,
  type Papers,
} from "./core";
import { deep, destinations } from "./folders";
import { agentsKeys, BRAINSTORM, hideChoice } from "./hiding";
import { waysOut } from "./leaving";
import { fill, t, type Word } from "./locales";
import { measureChoice, widens } from "./measureMenu";
import { folderPlace, type Place, placeOf, rulesOf } from "./places";
import { tongueChoices } from "./tongueMenus";
import type { Choice } from "./ui/Menu";
import { shortcut } from "./ui/macKeys";

export interface Hands {
  papers: Papers;
  showing: string | null;
  newDoc: (folder?: string, pageOf?: string) => void;
  newDiagram: (folder?: string) => void;
  bringIn: (folder?: string) => void;
  makeFolderIn: (folder?: string) => void;
  rename: (folder: Folded) => void;
  parcels: {
    takeParcel: () => unknown;
    packUp: (which: string[], named: string) => unknown;
  };
  changed: () => void;
  fail: (problem: unknown) => void;
  failWith: (text: string) => void;
  noted: (text: string) => void;
  open: (doc: string) => void;
  dropFolder: (folder: Folded) => void;
  dropDoc: (doc: Filed) => void;
  bringBack: (doc: Filed) => void;
  hangIt: (doc: string, pageOf: string) => void;
}

export const NEW_DOC_KEY = shortcut("N");
export const deleteKey = (): string => t("keyDelete");

const byAnother = (doc: Filed) => doc.away && !doc.archived;

// A choice that does not apply stays in its place and says why, so a menu keeps one shape.
const unless = (h: Hands, choice: Choice, why: string | null): Choice =>
  why === null ? choice : { ...choice, into: undefined, hint: why, onPick: () => h.failWith(why) };

const IDEAS_SAY: Record<Rule, Word> = {
  pages: "ideasHoldNoPages",
  moves: "ideasStayPut",
  copies: "ideasAreOne",
  goes: "ideasStay",
};

type Rule = "moves" | "copies" | "pages" | "goes";

const placeWhy = (place: Place, rule: Rule): string | null => {
  if (rulesOf(place)[rule]) return null;
  const said: Record<Place, Word | null> = {
    plain: null,
    agents: null,
    ideas: IDEAS_SAY[rule],
    diary:
      rule === "pages" ? "diaryHoldsOnlyDays" : rule === "moves" ? "diaryStaysPut" : "diaryIsOne",
  };
  const word = said[place];
  return word ? t(word) : null;
};

const moreToMake = (h: Hands, folder?: string, off: string | null = null): Choice =>
  unless(
    h,
    {
      key: "more",
      icon: "+",
      label: t("moreToMake"),
      into: {
        label: t("moreToMake"),
        choices: [
          {
            key: "newDiagram",
            icon: "+",
            label: t("newDiagram"),
            onPick: () => h.newDiagram(folder),
          },
          { key: "import", icon: "↧", label: t("importDoc"), onPick: () => h.bringIn(folder) },
        ],
      },
    },
    off,
  );

const newDocChoice = (h: Hands, folder: string | undefined, label: Word = "newDoc"): Choice => ({
  key: "newDoc",
  icon: "+",
  label: t(label),
  keys: NEW_DOC_KEY,
  onPick: () => h.newDoc(folder),
});

// The rules of AGENTS: a new document there is a rule, and other files come in through «Bring another file».
const agentsHere = (h: Hands, folder: string | undefined, off = false): Choice[] => [
  { ...newDocChoice(h, folder, "agentsNewRule"), off },
  { key: "agentsMore", icon: "↧", label: t("agentsMoreTitle"), off, onPick: bringMoreAgents },
];

export const hereChoices = (h: Hands, folder?: string): Choice[] => {
  if (folderPlace(folder, h.papers) === "agents") return agentsHere(h, folder);
  return [
    newDocChoice(h, folder),
    { key: "newFolder", icon: "+", label: t("newFolder"), onPick: () => h.makeFolderIn(folder) },
    { key: "newDiagram", icon: "+", label: t("newDiagram"), onPick: () => h.newDiagram(folder) },
    {
      key: "import",
      icon: "↧",
      label: t("importDoc"),
      apart: true,
      onPick: () => h.bringIn(folder),
    },
    { key: "unpack", icon: "↧", label: t("unpackIt"), onPick: () => h.parcels.takeParcel() },
    {
      key: "packAll",
      icon: "⇪",
      label: t("packAll"),
      apart: true,
      onPick: () => h.parcels.packUp([], "tisty"),
    },
  ];
};

// Only the switch moves: no agent file is touched and no document goes.
const turnSync = (h: Hands, assistant: string | null, on: boolean) =>
  agentsTurn(assistant, on)
    .then(() => {
      h.changed();
      window.dispatchEvent(new Event(AGENTS_CHANGED));
    })
    .catch(h.fail);

export const stopSyncing = (h: Hands) => turnSync(h, null, false);

const syncChoice = (): Choice => ({
  key: "agentsSync",
  icon: "↻",
  label: t("agentsSync"),
  onPick: syncAgents,
});

const rescanChoice = (): Choice => ({
  key: "agentsRescan",
  icon: "⟳",
  label: t("agentsRescan"),
  onPick: rescanAgents,
});

const unsyncChoice = (h: Hands): Choice => ({
  key: "agentsUnsync",
  icon: "⏸",
  label: t("agentsUnsync"),
  onPick: () => stopSyncing(h),
});

/// Whether Agents syncs lives with this computer, so its own menu asks before it opens.
export const stageFor = (
  folder: Folded,
  papers: Papers,
  then: (stage: AgentsStage | null, state: AgentsState | null) => void,
) => {
  const root = agentsRoot(papers.folders)?.id ?? null;
  if (root !== folder.id && !inAgents(folder.id, papers.folders)) return then(null, null);
  // Unknown counts as syncing: stopping twice is harmless, hiding a folder that writes is not.
  agentsState()
    .then((state) => then(agentsStage(root, state), state))
    .catch(() => then("syncs", null));
};

const agentsRootChoices = (folder: Folded, h: Hands, stage: AgentsStage | null): Choice[] => {
  const hiding = { ...agentsHideChoice(agentsKeys(folder.id), stage, h), apart: false };
  if (stage === "unbrought") return [hiding];
  if (stage === "brought") return [syncChoice(), rescanChoice(), hiding];
  return [unsyncChoice(h), rescanChoice(), hiding];
};

const underRoot = (folder: Folded, folders: readonly Folded[], root: string | undefined) => {
  let at: Folded | undefined = folder;
  while (at && at.parent !== root && !at.agents?.room) {
    const parent: string | null = at.parent;
    at = folders.find((one) => one.id === parent);
  }
  return at;
};

type Assistant = AgentsState["assistants"][number];

// Global and a folder of the person's own are no assistant's; pausing one only means something while Agents syncs.
const assistantOf = (
  folder: Folded,
  h: Hands,
  stage: AgentsStage | null,
  state: AgentsState | null,
): Assistant | undefined => {
  if (stage !== "syncs" || !state) return undefined;
  const top = underRoot(folder, h.papers.folders, agentsRoot(h.papers.folders)?.id);
  if (!top || ofAnother(top) || top.agents?.computer) return undefined;
  return state.assistants.find((one) => one.name === top.name);
};

const turnChoice = (h: Hands, assistant: Assistant, off: boolean): Choice => ({
  key: "agentsTurn",
  icon: assistant.on ? "⏸" : "↻",
  label: fill(assistant.on ? "agentPauseSync" : "agentResumeSync", assistant.name),
  off,
  onPick: () => turnSync(h, assistant.id, !assistant.on),
});

const awayChoice = (folder: Folded, h: Hands): Choice => ({
  // Only the folder that was shelved answers for itself; one below it comes back with it.
  key: "away",
  icon: folder.archived ? "▢" : "▣",
  label: folder.archived ? t("bringBack") : t("putAway"),
  off: folder.away && !folder.archived,
  onPick: () => folderAway(folder.id, !folder.archived).then(h.changed).catch(h.fail),
});

const dropChoice = (folder: Folded, h: Hands): Choice => ({
  key: "drop",
  icon: "✕",
  label: t("deleteSome"),
  keys: deleteKey(),
  off: folder.away,
  danger: true,
  onPick: () => h.dropFolder(folder),
});

// Inside Agents a folder's name says which assistant it writes to, so nothing there renames, moves or adds folders.
const agentsFolderChoices = (
  folder: Folded,
  h: Hands,
  stage: AgentsStage | null,
  state: AgentsState | null,
): Choice[] => {
  if (ofAnother(folder)) return [agentsHideChoice([folder.id], stage, h, folder.away)];
  const assistant = assistantOf(folder, h, stage, state);
  return [
    ...agentsHere(h, folder.id, folder.away),
    ...(assistant ? [turnChoice(h, assistant, folder.away)] : []),
    agentsHideChoice([folder.id], stage, h, folder.away),
    ...(folder.archived ? [awayChoice(folder, h)] : []),
    { ...dropChoice(folder, h), apart: true },
  ];
};

const filesUnder = (folder: string, papers: Papers): string[] => {
  const tree = new Set([folder]);
  for (let grew = true; grew; ) {
    grew = false;
    for (const one of papers.folders) {
      if (one.parent && tree.has(one.parent) && !tree.has(one.id)) {
        tree.add(one.id);
        grew = true;
      }
    }
  }
  return papers.docs
    .filter((one) => !one.pageOf && !one.away && one.folder && tree.has(one.folder))
    .map((one) => one.file);
};

/// On Agents itself only syncing and hiding; an unknown stage counts as syncing.
export const folderChoices = (
  folder: Folded,
  h: Hands,
  stage: AgentsStage | null = null,
  state: AgentsState | null = null,
): Choice[] => {
  if (agentsRoot(h.papers.folders)?.id === folder.id) return agentsRootChoices(folder, h, stage);
  if (inAgents(folder.id, h.papers.folders)) return agentsFolderChoices(folder, h, stage, state);
  const shelved = folder.away ? t("inArchiveSaid") : null;
  const full = deep(h.papers.folders, folder.id) >= DEEPEST ? t("tooDeepSaid") : null;
  const files = filesUnder(folder.id, h.papers);
  return [
    unless(h, newDocChoice(h, folder.id), shelved),
    unless(
      h,
      {
        key: "newFolder",
        icon: "+",
        label: t("newFolder"),
        onPick: () => h.makeFolderIn(folder.id),
      },
      shelved ?? full,
    ),
    moreToMake(h, folder.id, shelved),
    unless(
      h,
      {
        key: "rename",
        icon: "✎",
        label: t("rename"),
        keys: "F2",
        apart: true,
        onPick: () => h.rename(folder),
      },
      shelved,
    ),
    unless(
      h,
      {
        key: "move",
        icon: "⇢",
        label: t("moveTo"),
        into: {
          label: t("moveHere"),
          choices: destinations(
            h.papers.folders,
            folder.id,
            (parent) =>
              movedOnceSure("folder", folder.id, parent, () =>
                folderFile(folder.id, parent).then(h.changed),
              ).catch(h.fail),
            folder,
          ),
        },
      },
      shelved,
    ),
    unless(
      h,
      {
        key: "packFolder",
        icon: "⇪",
        label: t("packFolder"),
        onPick: () => h.parcels.packUp(files, folder.name),
      },
      files.length ? null : t("folderEmptySaid"),
    ),
    { ...hideChoice(folder.id, folder.away), apart: true },
    { ...awayChoice(folder, h), apart: false },
    dropChoice(folder, h),
  ];
};

const moveChoice = (doc: Filed, h: Hands, place: Place): Choice => {
  const own: Choice = {
    key: "ownDoc",
    icon: "⇤",
    label: t("ownDoc"),
    onPick: () => docPage(doc.id).then(h.changed).catch(h.fail),
  };
  const books = h.papers.docs.filter(
    (one) => one.id !== doc.id && !one.pageOf && !one.away && !one.locked,
  );
  const asPage: Choice = unless(
    h,
    {
      key: "pageOf",
      icon: "⇥",
      label: t("asPageOf"),
      into: {
        label: t("pageOfWhich"),
        choices: books
          .filter((one) => rulesOf(placeOf(one, h.papers)).pages)
          .map((one) => ({
            key: one.id,
            icon: "▤",
            label: one.title || t("untitledDoc"),
            onPick: () => h.hangIt(doc.id, one.id),
          })),
      },
    },
    doc.locked
      ? t("lockedSaid")
      : h.papers.docs.some((one) => one.pageOf === doc.id)
        ? t("holdsPages")
        : null,
  );
  const into = doc.pageOf
    ? [own]
    : [
        ...destinations(h.papers.folders, doc.folder, (folder) =>
          movedOnceSure("doc", doc.id, folder, () => docFile(doc.id, folder).then(h.changed)).catch(
            h.fail,
          ),
        ),
        { ...asPage, apart: true },
      ];
  return unless(
    h,
    { key: "move", icon: "⇢", label: t("moveTo"), into: { label: t("moveHere"), choices: into } },
    byAnother(doc) ? t("folderIsAway") : placeWhy(place, "moves"),
  );
};

// Duplicating makes a copy too, so it heads the copies and the menu keeps room for the rest.
const copyChoice = (doc: Filed, h: Hands, place: Place): Choice => ({
  key: "copyAs",
  icon: "⌘",
  label: t("copyAs"),
  apart: true,
  into: {
    label: t("copyAs"),
    choices: [
      unless(
        h,
        {
          key: "copy",
          icon: "⧉",
          label: t("duplicate"),
          onPick: () =>
            docCopy(doc.id)
              .then((made) => {
                h.changed();
                if (!doc.away) h.open(made.id);
              })
              .catch(h.fail),
        },
        ofAnother(h.papers.folders.find((one) => one.id === doc.folder))
          ? t("notHereSaid")
          : placeWhy(place, "copies"),
      ),
      {
        apart: true,
        key: "asPlain",
        icon: "⌘",
        label: t("copyPlain"),
        onPick: () =>
          asPlain(doc.file)
            .then(() => h.noted(t("copied")))
            .catch(h.fail),
      },
      {
        key: "asLink",
        icon: "↗",
        label: t("copyLink"),
        onPick: () =>
          asLink(doc.file)
            .then(() => h.noted(t("linkCopied")))
            .catch(h.fail),
      },
    ],
  },
});

const takenOut = (doc: Filed, h: Hands) => () =>
  pick({ directory: true })
    .then((at) => (typeof at === "string" ? docExport(doc.file, at) : null))
    .then((took) => {
      if (took === null) return;
      if (took.missed > 0) {
        h.failWith(took.missed === 1 ? t("takenShort") : fill("takenShorter", String(took.missed)));
        return;
      }
      if (took.left > 0) {
        h.failWith(took.left === 1 ? t("takenLess") : fill("takenLesser", String(took.left)));
        return;
      }
      h.noted(took.files ? fill("takenOut", String(took.files)) : t("takenOutAlone"));
    })
    .catch(h.fail);

// PDF, HTML and Word are drawn from the open page, so they wait until it is open and say so.
const exportChoice = (doc: Filed, h: Hands): Choice => ({
  key: "exportAs",
  icon: "⇩",
  label: t("exportAs"),
  into: {
    label: t("exportAs"),
    choices: [
      { key: "takeOut", icon: "⇪", label: t("takeOut"), onPick: takenOut(doc, h) },
      {
        key: "packIt",
        icon: "⇪",
        label: t("packIt"),
        onPick: () => h.parcels.packUp([doc.file], doc.title || doc.file),
      },
      ...waysOut(false).map((one) =>
        unless(h, one, h.showing === doc.file ? null : t("openToExport")),
      ),
    ],
  },
});

const languagesChoice = (doc: Filed, h: Hands, place: Place): Choice => {
  const all = tongueChoices(doc, h);
  return unless(
    h,
    {
      key: "languages",
      icon: "◌",
      label: t("languagesOf"),
      into: {
        label: t("languagesOf"),
        choices: all.map((one) =>
          unless(h, { ...one, off: false }, one.off ? t("notHereSaid") : null),
        ),
      },
    },
    placeWhy(place, "copies"),
  );
};

const everyChoice = (doc: Filed, h: Hands, place: Place): Choice[] => [
  unless(
    h,
    { key: "newPage", icon: "+", label: t("newPage"), onPick: () => h.newDoc(undefined, doc.id) },
    doc.away
      ? t("inArchiveSaid")
      : doc.pageOf
        ? t("pageOfPage")
        : (placeWhy(place, "pages") ?? (doc.locked ? t("lockedSaid") : null)),
  ),
  moveChoice(doc, h, place),
  copyChoice(doc, h, place),
  exportChoice(doc, h),
  languagesChoice(doc, h, place),
  measureChoice(doc),
  unless(
    h,
    {
      key: "lock",
      icon: doc.locked ? "◉" : "○",
      label: doc.locked ? t("unlockIt") : t("lockIt"),
      apart: true,
      onPick: () => docLock(doc.id, !doc.locked).then(h.changed).catch(h.fail),
    },
    byAnother(doc) ? t("folderIsAway") : doc.pageOf ? t("pageOfPage") : null,
  ),
  unless(
    h,
    {
      // Only the folder comes back for what it put away, and its own mark is what it recovers.
      key: "away",
      icon: doc.archived ? "▢" : "▣",
      label: doc.archived ? t("bringBack") : t("putAway"),
      onPick: () => {
        if (doc.archived) return h.bringBack(doc);
        docAway(doc.id, true).then(h.changed).catch(h.fail);
      },
    },
    byAnother(doc) ? t("folderIsAway") : doc.archived ? null : placeWhy(place, "goes"),
  ),
  unless(
    h,
    {
      key: "drop",
      icon: "✕",
      label: t("deleteIt"),
      keys: deleteKey(),
      danger: true,
      onPick: () => h.dropDoc(doc),
    },
    doc.locked ? t("lockedSaid") : byAnother(doc) ? t("folderIsAway") : placeWhy(place, "goes"),
  ),
];

// Ideas is Tisty's own page of ideas, and a diary keeps its days: each shows only what it takes.
const OWN_MENUS: Partial<Record<Place, Set<string>>> = {
  ideas: new Set(["away", "drop"]),
  diary: new Set(["copyAs", "exportAs", "away"]),
};

// Another computer's agent file is read, moved and taken out here; what it says and whether it stands is that computer's.
const OF_ANOTHER = new Set(["move", "copyAs", "takeOut"]);

// An agent reads its one file as it stands: a page, a language, a second copy or a printed form of it reaches no agent.
const FOR_AGENTS = new Set(["move", "copyAs", "away", "drop"]);

const opening = (choices: Choice[]): Choice[] =>
  choices.map((one, at) => ({ ...one, apart: at > 0 && one.apart }));

const forAgents = (all: Choice[], doc: Filed): Choice[] =>
  all.flatMap((one): Choice[] => {
    if (one.key === "copyAs" && one.into) {
      const copies = opening(one.into.choices.filter((each) => each.key !== "copy"));
      return [{ ...one, into: { ...one.into, choices: copies } }];
    }
    if (one.key === "exportAs")
      return (one.into?.choices ?? []).filter((each) => each.key === "takeOut");
    // A lock set before can still be lifted.
    if (one.key === "lock") return doc.locked ? [one] : [];
    return FOR_AGENTS.has(one.key) ? [one] : [];
  });

export const docChoices = (doc: Filed, h: Hands, board = false): Choice[] => {
  const place = board ? "ideas" : placeOf(doc, h.papers);
  const every = everyChoice(doc, h, place).filter((one) => one.key !== "measure" || widens(place));
  const all = place === "agents" ? forAgents(every, doc) : every;
  if (ofAnother(h.papers.folders.find((one) => one.id === doc.folder)))
    return all.filter((one) => OF_ANOTHER.has(one.key));
  const own = OWN_MENUS[place];
  if (!own) return all;
  const kept = all
    .filter((one) => own.has(one.key))
    .map((one, at) => ({ ...one, apart: at > 0 && one.apart }));
  return board
    ? [
        ...kept.filter((one) => one.key !== "drop"),
        hideChoice(BRAINSTORM),
        ...kept.filter((one) => one.key === "drop"),
      ]
    : kept;
};
