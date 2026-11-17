import { begun, matched } from "tisty-editor/read";
import type { Filed, Found, List, Papers, Task } from "./core";
import { trail } from "./folders";
import { latest } from "./lately";
import { t } from "./locales";
import { shown } from "./tongues";
import type { Pickable } from "./ui/Picker";

const RECENT = 8;
const TITLED = 8;
const FOLDERS = 4;
const TASKS = 6;
const INSIDE = 5;

export interface Going {
  doc: (file: string) => void;
  folder: (id: string) => void;
  task: (task: Task) => void;
}

const named = (doc: Filed): string => doc.title.trim() || t("untitledDoc");

const stamp = (doc: Filed): number => Date.parse(doc.wrote ?? "") || 0;

// Sister pages are called January, February, March: without their document the rows read alike.
const underOf = (doc: Filed, all: Filed[]): string | undefined => {
  const book = doc.pageOf ? all.find((one) => one.id === doc.pageOf) : undefined;
  return book ? named(book) : undefined;
};

const docRow = (doc: Filed, all: Filed[], group: string, go: Going): Pickable => ({
  key: `doc-${doc.id}`,
  glyph: "▤",
  label: named(doc),
  detail: underOf(doc, all),
  group,
  onPick: () => go.doc(doc.file),
});

const taskRow = (task: Task, lists: List[], go: Going): Pickable => {
  const list = lists.find((one) => one.id === task.list)?.name;
  const closed = task.status === "open" ? undefined : t("paletteArchived");
  return {
    key: `task-${task.id}`,
    glyph: task.status === "open" ? "○" : "✓",
    label: task.title,
    detail: [list, closed].filter(Boolean).join(" · ") || undefined,
    group: t("paletteTasks"),
    onPick: () => go.task(task),
  };
};

// The documents written last when nothing is asked; asked, titles first, then folders, tasks and what the text holds.
export function jumps(
  papers: Papers,
  lists: List[],
  asked: string,
  found: Found | null,
  go: Going,
): Pickable[] {
  const docs = shown(papers.docs).filter((one) => !one.away && !one.gone);
  if (!asked.trim()) {
    return latest(papers, RECENT).map((doc) => docRow(doc, docs, t("paletteRecent"), go));
  }
  const titled = docs
    .filter((one) => matched(named(one), asked))
    .sort(
      (one, two) =>
        Number(begun(named(two), asked)) - Number(begun(named(one), asked)) ||
        stamp(two) - stamp(one),
    )
    .slice(0, TITLED);
  const folders = papers.folders
    .filter((one) => !one.away && matched(one.name, asked))
    .slice(0, FOLDERS)
    .map(
      (folder): Pickable => ({
        key: `folder-${folder.id}`,
        glyph: "▭",
        label: folder.name,
        detail: folder.parent ? trail(papers.folders, folder.parent) : undefined,
        group: t("paletteFolders"),
        onPick: () => go.folder(folder.id),
      }),
    );
  const tasks = [...(found?.tasks ?? [])]
    .sort((one, two) => Number(two.status === "open") - Number(one.status === "open"))
    .slice(0, TASKS)
    .map((task) => taskRow(task, lists, go));
  const seen = new Set(titled.map((one) => one.file));
  const inside = (found?.papers ?? [])
    .filter((one) => !seen.has(one.id))
    .slice(0, INSIDE)
    .map(
      (one): Pickable => ({
        key: `inside-${one.id}`,
        glyph: "▤",
        label: one.title.trim() || t("untitledDoc"),
        detail: one.line || undefined,
        group: t("paletteInText"),
        onPick: () => go.doc(one.id),
      }),
    );
  return [
    ...titled.map((doc) => docRow(doc, docs, t("paletteDocs"), go)),
    ...folders,
    ...tasks,
    ...inside,
  ];
}
