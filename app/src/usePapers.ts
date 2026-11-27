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
  noteTrouble,
  type Papers,
  taskPageImport,
} from "./core";
import { viewDiagram } from "./drawing";
import { dropLater } from "./dropLater";
import { folderDropAsked, folderDropped } from "./folderDropping";
import { letAsideGo } from "./keptAside";
import { leftBehind } from "./leftBehind";
import { fill, t } from "./locales";
import { drafted, placeOf } from "./making";
import { saidPlainly } from "./refusal";
import { cut } from "./saving";
import { sure } from "./sure";
import { dropPickedAsked, filesGoing, type Going, goingWith } from "./tidying";
import { usePinned } from "./usePinned";
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

  const treeOf = (folder: string): Set<string> => {
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
    return tree;
  };

  // An empty folder takes the plain question; one that holds something asks where it goes.
  const dropFolder = async (folder: Folded) => {
    const docsIn = papers.docs.filter((one) => one.folder === folder.id && !one.pageOf);
    const kids = papers.folders.filter((one) => one.parent === folder.id);
    // The core keeps a folder of Agents that still holds files; saying so now beats ten seconds of waiting for it.
    if (docsIn.length + kids.length > 0 && inAgents(folder.id, papers.folders)) {
      setError(t("agentsFolderHolds"));
      return;
    }
    try {
      const answer =
        docsIn.length + kids.length === 0
          ? (await sure(fill("dropFolderSure", folder.name)))
            ? { contents: "up" as const }
            : null
          : await folderDropAsked({
              folder,
              up: papers.folders.find((one) => one.id === folder.parent) ?? null,
              docs: docsIn.length,
              folders: kids.length,
              handsOnly: inAgents(folder.id, papers.folders),
            });
      if (!answer) return;
      const tree = treeOf(folder.id);
      const lost =
        answer.contents === "drop"
          ? papers.docs.filter((one) => one.folder && tree.has(one.folder))
          : [];
      // The core keeps the whole folder when one document inside cannot go: say so now, not in ten seconds.
      const kept = lost.find((one) => stays(one));
      if (kept)
        return setError(
          kept.locked ? fill("folderKeepsLocked", kept.title || t("untitledDoc")) : stays(kept),
        );
      if (here && tree.has(here)) setHere(undefined);
      setReturning(folder.parent ?? "unfiled");
      leaving(lost);
      dropLater({
        hides: answer.contents === "drop" ? [...tree, ...lost.map((one) => one.id)] : [folder.id],
        moves:
          answer.contents === "drop"
            ? undefined
            : {
                from: folder.id,
                to: answer.contents === "up" ? (folder.parent ?? null) : (answer.into ?? null),
              },
        said: fill("folderGone", folder.name),
        run: () =>
          folderDropped(folder.id, answer)
            .then(() => {
              for (const one of lost) gone(one.file);
            })
            .catch((e) => setError(saidPlainly(e)))
            .finally(papersChanged),
      });
    } catch (e) {
      setError(saidPlainly(e));
    }
  };

  // The page closes as soon as its row hides, so nothing is written into what is about to go.
  const leaving = (docs: Filed[]) => {
    const open = chosenNow.current.doc;
    if (open && docs.some((one) => one.file === open)) setChosen({ named: "docs" });
  };

  const board = usePinned(papers)?.file ?? null;

  // As its menu: a locked document, one its archived folder holds, or Ideas says why instead of going.
  const stays = (doc: Filed): string | null => {
    if (doc.locked) return t("lockedSaid");
    if (doc.away && !doc.archived) return t("folderIsAway");
    return doc.file === board ? t("ideasStay") : null;
  };

  const goingIds = (going: Going[]) =>
    going.flatMap(({ doc, tongues, pages }) => [doc, ...tongues, ...pages].map((one) => one.id));

  const goingDocs = (going: Going[]) =>
    going.flatMap(({ doc, tongues, pages }) => [doc, ...tongues, ...pages]);

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
    const why = stays(doc);
    if (why) return setError(why);
    const going = goingWith([doc], papers.docs);
    const asked = dropPickedAsked(going);
    if (await droppedFromAgents(going, asked)) return;
    if (!(await sure(asked).catch(() => false))) return;
    leaving(goingDocs(going));
    returnTo(doc);
    dropLater({
      hides: goingIds(going),
      said: fill("docGone", doc.title || t("untitledDoc")),
      run: () =>
        docDrop(doc.id)
          .then(() => wentAway(going[0]))
          .catch((e) => setError(saidPlainly(e)))
          .finally(papersChanged),
    });
  };

  const dropDocs = async (picked: Filed[]) => {
    const all = picked.filter((one) => !stays(one));
    const kept = picked.find((one) => stays(one));
    if (kept) setError(stays(kept));
    if (!all.length) return;
    const going = goingWith(all, papers.docs);
    const asked = dropPickedAsked(going);
    if (await droppedFromAgents(going, asked)) return;
    if (!(await sure(asked))) return;
    leaving(goingDocs(going));
    if (going[0]) returnTo(going[0].doc);
    dropLater({
      hides: goingIds(going),
      said:
        going.length === 1
          ? fill("docGone", going[0].doc.title || t("untitledDoc"))
          : fill("docsGone", String(going.length)),
      run: async () => {
        const refused: string[] = [];
        for (const one of going) {
          await docDrop(one.doc.id)
            .then(() => wentAway(one))
            .catch((e: unknown) => {
              refused.push(fill("dropRefused", one.doc.title || t("untitledDoc"), saidPlainly(e)));
            });
        }
        if (refused.length > 0) setError(refused.join(" · "));
        papersChanged();
      },
    });
  };

  const bringBack = (doc: Filed) => {
    // A page goes back with its document, and the diary keeps its own place, so neither asks where.
    if (doc.pageOf || doc.diary) {
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
    board,
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
