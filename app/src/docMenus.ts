import { open as pick } from "@tauri-apps/plugin-dialog";
import { asPlain } from "./copying";
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
import { fill, t } from "./locales";
import type { Choice } from "./ui/Menu";

export interface Hands {
  papers: Papers;
  showing: string | null;
  newDoc: (folder?: string, pageOf?: string) => void;
  bringIn: (folder?: string) => void;
  makeFolder: () => void;
  makeFolderIn: (folder: string) => void;
  rename: (folder: Folded) => void;
  parcels: {
    takeParcel: () => unknown;
    packUp: (which: string[], named: string) => unknown;
    takeOutAll: () => unknown;
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

const byAnother = (doc: Filed) => doc.away && !doc.archived;

export const hereChoices = (h: Hands): Choice[] => [
  { key: "newDoc", icon: "+", label: t("newDoc"), onPick: () => h.newDoc(undefined) },
  {
    key: "newFolder",
    icon: "+",
    label: t("newFolder"),
    onPick: h.makeFolder,
  },
  {
    key: "import",
    icon: "↧",
    label: t("importDoc"),
    apart: true,
    onPick: () => h.bringIn(undefined),
  },
  { key: "unpack", icon: "↧", label: t("unpackIt"), onPick: () => h.parcels.takeParcel() },
  {
    key: "packAll",
    icon: "⇪",
    label: t("packAll"),
    onPick: () => h.parcels.packUp([], "tisty"),
  },
  {
    key: "takeOutAll",
    icon: "⇪",
    label: t("takeOutAll"),
    onPick: () => h.parcels.takeOutAll(),
  },
];

export const folderChoices = (folder: Folded, h: Hands): Choice[] => [
  {
    key: "newDoc",
    icon: "+",
    label: t("newDoc"),
    off: folder.away,
    onPick: () => h.newDoc(folder.id),
  },
  {
    key: "newFolder",
    icon: "+",
    label: t("newFolder"),
    off: folder.away || deep(h.papers.folders, folder.id) >= DEEPEST,
    onPick: () => h.makeFolderIn(folder.id),
  },
  {
    key: "rename",
    icon: "✎",
    label: t("rename"),
    off: folder.away,
    apart: true,
    onPick: () => h.rename(folder),
  },
  {
    key: "move",
    icon: "⇢",
    label: t("moveTo"),
    off: folder.away,
    into: {
      label: t("moveHere"),
      choices: destinations(
        h.papers.folders,
        folder.id,
        (parent) => folderFile(folder.id, parent).then(h.changed).catch(h.fail),
        folder,
      ),
    },
  },
  {
    key: "import",
    icon: "↧",
    label: t("importDoc"),
    off: folder.away,
    onPick: () => h.bringIn(folder.id),
  },
  {
    key: "unpack",
    icon: "↧",
    label: t("unpackIt"),
    off: folder.away,
    onPick: () => h.parcels.takeParcel(),
  },
  {
    key: "packAll",
    icon: "⇪",
    label: t("packAll"),
    onPick: () => h.parcels.packUp([], "tisty"),
  },
  {
    key: "takeOutAll",
    icon: "⇪",
    label: t("takeOutAll"),
    onPick: () => h.parcels.takeOutAll(),
  },
  {
    // Only the folder that was shelved answers for itself; one below it comes back with it.
    key: "away",
    icon: folder.archived ? "▢" : "▣",
    label: folder.archived ? t("bringBack") : t("putAway"),
    off: folder.away && !folder.archived,
    apart: true,
    onPick: () => folderAway(folder.id, !folder.archived).then(h.changed).catch(h.fail),
  },
  {
    key: "drop",
    icon: "✕",
    label: t("deleteIt"),
    off: folder.away,
    danger: true,
    apart: true,
    onPick: () => h.dropFolder(folder),
  },
];

export const docChoices = (doc: Filed, h: Hands): Choice[] => [
  {
    key: "newPage",
    icon: "+",
    label: t("newPage"),
    off: doc.away || !!doc.pageOf,
    onPick: () => h.newDoc(undefined, doc.id),
  },
  {
    key: "pageOf",
    icon: "⇥",
    label: t("pageOf"),
    off:
      doc.away || doc.locked || !!doc.pageOf || h.papers.docs.some((one) => one.pageOf === doc.id),
    into: {
      label: t("pageOfWhich"),
      choices: h.papers.docs
        .filter((one) => one.id !== doc.id && !one.pageOf && !one.away && !one.locked)
        .map((one) => ({
          key: one.id,
          icon: "▤",
          label: one.title || t("untitledDoc"),
          onPick: () => h.hangIt(doc.id, one.id),
        })),
    },
  },
  {
    key: "ownDoc",
    icon: "⇤",
    label: t("ownDoc"),
    off: !doc.pageOf || byAnother(doc),
    onPick: () => docPage(doc.id).then(h.changed).catch(h.fail),
  },
  {
    key: "move",
    icon: "⇢",
    label: t("moveTo"),
    off: byAnother(doc) || !!doc.pageOf,
    into: {
      label: t("moveHere"),
      choices: destinations(h.papers.folders, doc.folder, (folder) =>
        docFile(doc.id, folder).then(h.changed).catch(h.fail),
      ),
    },
  },
  {
    key: "asPlain",
    icon: "⌘",
    label: t("copyPlain"),
    apart: true,
    onPick: () =>
      asPlain(doc.file)
        .then(() => {
          h.noted(t("copied"));
        })
        .catch(h.fail),
  },
  {
    key: "takeOut",
    icon: "⇪",
    label: t("takeOut"),
    onPick: () =>
      pick({ directory: true })
        .then((at) => (typeof at === "string" ? docExport(doc.file, at) : null))
        .then((took) => {
          if (took === null) return;
          if (took.missed > 0) {
            h.failWith(
              took.missed === 1 ? t("takenShort") : fill("takenShorter", String(took.missed)),
            );
            return;
          }
          if (took.left > 0) {
            h.failWith(took.left === 1 ? t("takenLess") : fill("takenLesser", String(took.left)));
            return;
          }
          h.noted(took.files ? fill("takenOut", String(took.files)) : t("takenOutAlone"));
        })
        .catch(h.fail),
  },
  {
    key: "packIt",
    icon: "⇪",
    label: t("packIt"),
    onPick: () => h.parcels.packUp([doc.file], doc.title || doc.file),
  },
  {
    key: "seePdf",
    icon: "▤",
    label: t("seePdf"),
    off: h.showing !== doc.file,
    onPick: () => window.dispatchEvent(new CustomEvent("tisty:see-pdf")),
  },
  {
    key: "asPdf",
    icon: "⇩",
    label: t("toPdf"),
    off: h.showing !== doc.file,
    onPick: () => window.dispatchEvent(new CustomEvent("tisty:to-pdf")),
  },
  {
    key: "copy",
    icon: "⧉",
    label: t("duplicate"),
    apart: true,
    onPick: () =>
      docCopy(doc.id)
        .then((made) => {
          h.changed();
          if (!doc.away) h.open(made.id);
        })
        .catch(h.fail),
  },
  {
    key: "lock",
    icon: doc.locked ? "◉" : "○",
    label: doc.locked ? t("unlockIt") : t("lockIt"),
    off: !!doc.pageOf || byAnother(doc),
    apart: true,
    onPick: () => docLock(doc.id, !doc.locked).then(h.changed).catch(h.fail),
  },
  {
    // What the folder put away has no door of its own: only the folder comes back, and
    // its own mark is what it recovers when it does.
    key: "away",
    icon: doc.archived ? "▢" : "▣",
    label: doc.archived ? t("bringBack") : t("putAway"),
    off: byAnother(doc),
    apart: true,
    onPick: () => {
      if (doc.archived) return h.bringBack(doc);
      docAway(doc.id, true).then(h.changed).catch(h.fail);
    },
  },
  {
    key: "drop",
    icon: "✕",
    label: t("deleteIt"),
    off: doc.locked || byAnother(doc),
    danger: true,
    onPick: () => h.dropDoc(doc),
  },
];
