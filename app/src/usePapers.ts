import { listen } from "@tauri-apps/api/event";
import { open as pick } from "@tauri-apps/plugin-dialog";
import { type MutableRefObject, useCallback, useEffect, useRef, useState } from "react";
import type { carrying } from "./carrying";
import {
  docAway,
  docDrop,
  docImport,
  docNew,
  docPage,
  docs,
  docsCatchUp,
  type Filed,
  type Folded,
  folderDrop,
  noteTrouble,
  type Papers,
  taskPageImport,
} from "./core";
import { viewDiagram } from "./drawing";
import { leftBehind } from "./leftBehind";
import { fill, t } from "./locales";
import { drafted } from "./making";
import { saidPlainly } from "./refusal";
import { sure } from "./sure";
import { dropAllAsked, goingWith } from "./tidying";
import type { Start } from "./ui/drawing/starting";
import type { Chosen } from "./views";

// An old copy of a task's page would take away what the task holds now, so the person says so first.
export const pageHome = (at: string): Promise<string | null | undefined> =>
  taskPageImport(at).catch(async (e: unknown) => {
    if ((e as { code?: string } | null)?.code !== "taskPageBehind") throw e;
    return (await sure(t("taskPageBehindSure"))) ? taskPageImport(at, true) : undefined;
  });

export const steady = <T>(was: T, found: T): T =>
  JSON.stringify(was) === JSON.stringify(found) ? was : found;

interface Hands {
  chosen: Chosen;
  setChosen: (chosen: Chosen) => void;
  here: string | null | undefined;
  setHere: (here: string | null | undefined) => void;
  setReturning: (at: string | null) => void;
  setBacking: (doc: Filed | null) => void;
  setError: (text: string | null) => void;
  openTask: (id: string) => void;
  noted: (text: string, fades?: number) => void;
  lookForAStar: () => void;
  carries: MutableRefObject<ReturnType<typeof carrying> | null>;
  paging: MutableRefObject<((page: Filed) => boolean) | null>;
}

export function usePapers({
  chosen,
  setChosen,
  here,
  setHere,
  setReturning,
  setBacking,
  setError,
  openTask,
  noted,
  lookForAStar,
  carries,
  paging,
}: Hands) {
  const [papers, setPapers] = useState<Papers>({ folders: [], docs: [] });

  const newDoc = (folder?: string, pageOf?: string) =>
    docNew(folder, pageOf)
      .then((made) => {
        papersChanged();
        setChosen({ named: "docs", doc: made.id });
        if (!pageOf) lookForAStar();
      })
      .catch((e) => setError(saidPlainly(e)));

  const newDiagram = (name: string, kind: Start, into?: string) =>
    drafted(name, kind, papers.folders, into)
      .then((made) => {
        papersChanged();
        setChosen({ named: "docs", doc: made.id });
        lookForAStar();
        const paper = { id: "", file: made.id, title: made.title, place: made.place };
        return viewDiagram(paper, made.at, 0, made.source);
      })
      .catch((e) => setError(saidPlainly(e)));

  const bringIn = (folder?: string) =>
    pick({
      multiple: false,
      filters: [
        { name: "Markdown", extensions: ["md", "markdown", "txt", "html", "htm", "docx"] },
        { name: t("anyFile"), extensions: ["*"] },
      ],
    })
      .then(async (at) => {
        if (typeof at !== "string") return null;
        // A task's own page coming back goes to its task, never into the documents.
        const task = await pageHome(at);
        if (task === undefined) return null;
        if (task) {
          openTask(task);
          return null;
        }
        return docImport(at, folder);
      })
      .then((made) => {
        if (!made) return;
        papersChanged();
        setChosen({ named: "docs", doc: made.id });
        const left = leftBehind(made);
        if (left) noted(left, 9_000);
      })
      .catch((e) => setError(saidPlainly(e)));

  const dropFolder = (folder: Folded) =>
    sure(fill("dropFolderSure", folder.name))
      .then((yes) => {
        if (!yes) return;
        if (here === folder.id) setHere(undefined);
        setReturning(folder.parent ?? "unfiled");
        return folderDrop(folder.id).then(papersChanged);
      })
      .catch((e) => setError(saidPlainly(e)));

  const dropDoc = (doc: Filed) =>
    sure(
      fill(
        papers.docs.some((one) => one.pageOf === doc.id) ? "dropPagesSure" : "dropDocSure",
        doc.title || t("untitledDoc"),
      ),
    )
      .then((yes) => {
        if (!yes) return;
        const going = [
          doc.file,
          ...papers.docs.filter((one) => one.pageOf === doc.id).map((one) => one.file),
        ];
        if (chosen.doc && going.includes(chosen.doc)) setChosen({ named: "docs" });
        setReturning(doc.pageOf ?? doc.folder ?? "unfiled");
        return docDrop(doc.id).then(papersChanged);
      })
      .catch((e) => setError(saidPlainly(e)));

  const dropDocs = async (all: Filed[]) => {
    const { going, pages } = goingWith(all, papers.docs);
    if (!(await sure(dropAllAsked(going, pages)))) return;
    const files = new Set([...all.map((one) => one.file), ...pages.map((one) => one.file)]);
    if (chosen.doc && files.has(chosen.doc)) setChosen({ named: "docs" });
    const first = going[0];
    if (first) setReturning(first.pageOf ?? first.folder ?? "unfiled");
    try {
      for (const one of going) await docDrop(one.id);
    } catch (e) {
      setError(saidPlainly(e));
    }
    papersChanged();
  };

  const bringBack = (doc: Filed) => {
    if (doc.pageOf) {
      docAway(doc.id, false)
        .then(papersChanged)
        .catch((e) => setError(saidPlainly(e)));
      return;
    }
    setBacking(doc);
  };

  const openDoc = (paper: string) => {
    if (papers.docs.some((one) => one.file === paper)) {
      return setChosen({ named: "docs", doc: paper });
    }
    docs()
      .then((found) => {
        setPapers((was) => steady(was, found ?? { folders: [], docs: [] }));
        if (found?.docs.some((one) => one.file === paper)) {
          setChosen({ named: "docs", doc: paper });
        } else {
          void noteTrouble("goneDoc", paper);
          setError(t("goneDoc"));
        }
      })
      .catch((e) => setError(saidPlainly(e)));
  };

  const opening = useRef(openDoc);
  opening.current = openDoc;
  const changing = useRef<() => void>(() => {});
  useEffect(() => {
    const off = listen<string>("showdoc", ({ payload }) => opening.current(payload));
    const fresh = listen("papered", () => changing.current());
    return () => {
      void off.then((stop) => stop());
      void fresh.then((stop) => stop());
    };
  }, []);

  const caught = useRef(false);
  const lookPapers = useCallback(() => {
    docs()
      .then((found) => {
        const now = found ?? { folders: [], docs: [] };
        setPapers((was) => steady(was, now));
        if (caught.current || now.docs.every((one) => one.told !== false)) return;
        caught.current = true;
        return docsCatchUp()
          .then((all) => {
            setPapers((was) => steady(was, { folders: was.folders, docs: all }));
            if (all.some((one) => one.told === false && !one.gone)) caught.current = false;
          })
          .catch(() => {
            caught.current = false;
          });
      })
      .catch(() => {});
  }, []);
  useEffect(lookPapers, [lookPapers]);
  const papersChanged = useCallback(() => {
    lookPapers();
    carries.current?.changed();
  }, [lookPapers, carries]);
  changing.current = papersChanged;

  const hangIt = async (doc: string, pageOf: string) => {
    const named = (id: string) =>
      papers.docs.find((one) => one.id === id)?.title || t("untitledDoc");
    if (!(await sure(fill("pageOfSure", named(doc), named(pageOf))))) return;
    const under = papers.docs.find((one) => one.id === pageOf);
    const page = papers.docs.find((one) => one.id === doc);
    docPage(doc, pageOf)
      .then(() => {
        // The line goes in through the editor that holds the book, so its own save carries it and
        // nothing is written behind it. A book that is not open leaves the page in the loose half.
        if (page && under && chosen.doc === under.file) {
          const put = paging.current;
          if (!put) setError(t("leafWaitsInIndex"));
          else if (put(page) === false) setError(t("leafNeedsTitle"));
        }
      })
      .then(papersChanged)
      .catch((e) => setError(saidPlainly(e)));
  };

  return {
    papers,
    lookPapers,
    papersChanged,
    newDoc,
    newDiagram,
    bringIn,
    dropFolder,
    dropDoc,
    dropDocs,
    bringBack,
    openDoc,
    hangIt,
  };
}
