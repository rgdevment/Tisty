import { ask, open as pick } from "@tauri-apps/plugin-dialog";
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
} from "./core";
import { fill, t } from "./locales";
import { saidPlainly } from "./refusal";
import type { Chosen } from "./views";

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

  const bringIn = (folder?: string) =>
    pick({
      multiple: false,
      filters: [
        { name: "Markdown", extensions: ["md", "markdown", "txt"] },
        { name: t("anyFile"), extensions: ["*"] },
      ],
    })
      .then((at) => (typeof at === "string" ? docImport(at, folder) : null))
      .then((made) => {
        if (!made) return;
        papersChanged();
        setChosen({ named: "docs", doc: made.id });
      })
      .catch((e) => setError(saidPlainly(e)));

  const dropFolder = (folder: Folded) =>
    ask(fill("dropFolderSure", folder.name), { kind: "warning" })
      .then((yes) => {
        if (!yes) return;
        if (here === folder.id) setHere(undefined);
        setReturning(folder.parent ?? "unfiled");
        return folderDrop(folder.id).then(papersChanged);
      })
      .catch((e) => setError(saidPlainly(e)));

  const dropDoc = (doc: Filed) =>
    ask(
      fill(
        papers.docs.some((one) => one.pageOf === doc.id) ? "dropPagesSure" : "dropDocSure",
        doc.title || t("untitledDoc"),
      ),
      { kind: "warning" },
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

  const hangIt = async (doc: string, pageOf: string) => {
    const named = (id: string) =>
      papers.docs.find((one) => one.id === id)?.title || t("untitledDoc");
    if (!(await ask(fill("pageOfSure", named(doc), named(pageOf)), { kind: "warning" }))) return;
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
    bringIn,
    dropFolder,
    dropDoc,
    bringBack,
    openDoc,
    hangIt,
  };
}
