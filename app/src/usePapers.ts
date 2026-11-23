import { listen } from "@tauri-apps/api/event";
import { open as pick } from "@tauri-apps/plugin-dialog";
import { type MutableRefObject, useCallback, useEffect, useRef, useState } from "react";
import type { Start } from "tisty-diagrams/read";
import { dropAsked } from "./agentsDropping";
import { agentsDrop, agentsReach, inAgents, type Reach } from "./agentsSync";
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
import { letAsideGo } from "./keptAside";
import { leftBehind } from "./leftBehind";
import { fill, t } from "./locales";
import { drafted, placeOf } from "./making";
import { saidPlainly } from "./refusal";
import { cut } from "./saving";
import { sure } from "./sure";
import { dropAllAsked, filesGoing, type Going, goingWith } from "./tidying";
import { tonguesNamed } from "./tongues";
import type { Chosen } from "./views";

// An old copy of a task's page would take away what the task holds now, so the person says so first.
export const pageHome = (at: string): Promise<string | null | undefined> =>
  taskPageImport(at).catch(async (e: unknown) => {
    if ((e as { code?: string } | null)?.code !== "taskPageBehind") throw e;
    return (await sure(t("taskPageBehindSure"))) ? taskPageImport(at, true) : undefined;
  });

const gone = (file: string) => {
  cut(file);
  letAsideGo(file);
};

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
  const chosenNow = useRef(chosen);
  chosenNow.current = chosen;

  const newDoc = (folder?: string, pageOf?: string) =>
    docNew(folder, pageOf, true)
      .then((made) => {
        papersChanged();
        setChosen({ named: "docs", doc: made.id });
        if (!pageOf) lookForAStar();
      })
      .catch((e) => setError(saidPlainly(e)));

  const newDiagram = (name: string, kind: Start, into?: string) =>
    drafted(name, kind, into)
      .then((made) => {
        papersChanged();
        setChosen({ named: "docs", doc: made.id });
        lookForAStar();
        const place = placeOf(papers.folders, made.folder);
        const paper = { id: "", file: made.id, title: made.title, place };
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

  const wentAway = (one: Going) => {
    const files = filesGoing([one]);
    for (const file of files) gone(file);
    const open = chosenNow.current.doc;
    if (open && files.includes(open)) setChosen({ named: "docs" });
  };

  const returnTo = (doc: Filed) => setReturning(doc.pageOf ?? doc.folder ?? "unfiled");

  // What a document of Agents wrote into an agent's folder is asked about in the one question that deletes it.
  const reached = async (going: Going[]): Promise<Reach | null> => {
    if (!going.some((one) => inAgents(one.doc.folder, papers.folders))) return null;
    const reach = await agentsReach(going.map((one) => one.doc.id));
    const loose = reach.at.length + reach.changed.length + reach.paused.length;
    return loose > 0 ? reach : null;
  };

  const dropFromAgents = async (going: Going[], reach: Reach, said: string) => {
    const answer = await dropAsked({ said, reach });
    if (!answer) return;
    const done = await agentsDrop(
      going.map((one) => one.doc.id),
      answer === "away",
      reach.at,
    );
    const went = going.filter((one) => done.gone.includes(one.doc.id));
    for (const one of went) wentAway(one);
    if (went[0]) returnTo(went[0].doc);
    const titled = (id: string) =>
      going.find((one) => one.doc.id === id)?.doc.title || t("untitledDoc");
    const trouble = [
      ...done.refused.map((one) => fill("dropRefused", titled(one.id), saidPlainly(one.refusal))),
      ...done.untaken.map((one) => fill("agentsDropUntaken", one.at, saidPlainly(one.refusal))),
      ...done.unmarked.map((one) => fill("agentsDropUnmarked", one.at, saidPlainly(one.refusal))),
    ];
    if (trouble.length > 0) setError(trouble.join(" · "));
    papersChanged();
  };

  // Nothing is deleted when Agents cannot say who keeps a copy: the error is shown and the question ends there.
  const droppedFromAgents = async (going: Going[], said: string): Promise<boolean> => {
    try {
      const reach = await reached(going);
      if (!reach) return false;
      await dropFromAgents(going, reach, said);
    } catch (e) {
      setError(saidPlainly(e));
    }
    return true;
  };

  const dropDoc = async (doc: Filed) => {
    // The languages beside it are the same row, so they go with it, and the question says so.
    const going = goingWith([doc], papers.docs);
    const [{ tongues, pages }] = going;
    const asked =
      tongues.length > 1
        ? fill("dropTonguesSure", doc.title || t("untitledDoc"), tonguesNamed(tongues))
        : fill(pages.length ? "dropPagesSure" : "dropDocSure", doc.title || t("untitledDoc"));
    if (await droppedFromAgents(going, asked)) return;
    return sure(asked)
      .then((yes) => {
        if (!yes) return;
        return docDrop(doc.id).then(() => {
          wentAway(going[0]);
          returnTo(doc);
          papersChanged();
        });
      })
      .catch((e) => setError(saidPlainly(e)));
  };

  const dropDocs = async (all: Filed[]) => {
    const going = goingWith(all, papers.docs);
    const asked = dropAllAsked(going);
    if (await droppedFromAgents(going, asked)) return;
    if (!(await sure(asked))) return;
    const went: Filed[] = [];
    const refused: string[] = [];
    for (const one of going) {
      await docDrop(one.doc.id)
        .then(() => {
          wentAway(one);
          went.push(one.doc);
        })
        .catch((e: unknown) => {
          refused.push(fill("dropRefused", one.doc.title || t("untitledDoc"), saidPlainly(e)));
        });
    }
    if (went[0]) returnTo(went[0]);
    if (refused.length > 0) setError(refused.join(" · "));
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
  useEffect(() => {
    const off = listen<string>("showdoc", ({ payload }) => opening.current(payload));
    return () => {
      void off.then((stop) => stop());
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
