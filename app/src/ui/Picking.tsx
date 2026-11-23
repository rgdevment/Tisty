import { useRef, useState } from "react";
import { fenced, nameOf, STARTS, startOf } from "tisty-diagrams/read";
import type { Asked } from "tisty-editor/edit";
import { known } from "tisty-editor/read";
import type { Filed } from "../core";
import { locale, t, type Word } from "../locales";
import { spawned } from "../making";
import { docCard, taskCard } from "../markdown";
import Asking from "./Asking";
import DiagramFinder from "./DiagramFinder";
import Glyphs from "./Glyphs";
import Papers from "./Papers";
import TaskMaking from "./TaskMaking";
import TaskPick from "./TaskPick";

export type Picked = {
  readonly kind: Asked;
  readonly at: { x: number; y: number };
  readonly put: (text: string) => boolean;
  readonly file?: string;
  readonly before?: string;
  readonly whole?: string;
};

// The icons keep the room they have everywhere else, so they read the same here as in a list or a folder.
const ROOM = {
  narrow: { wide: 272, tall: 340 },
  icons: { wide: 400, tall: 430 },
  making: { wide: 480, tall: 330 },
  finder: { wide: 600, tall: 370 },
};

export default function Picking({
  picked,
  own,
  pages,
  others,
  onMade,
  onLost,
  onError,
  onDone,
  onDrawn,
}: {
  picked: Picked;
  own?: Filed;
  pages: Filed[];
  others: Filed[];
  onMade: (doc: { id: string; title: string }) => void;
  onLost: (doc: Filed) => boolean;
  onError: (problem: unknown) => void;
  onDone: () => void;
  onDrawn?: (source: string, write: () => boolean) => void;
}) {
  const { kind, at, put } = picked;
  const [finding, setFinding] = useState(kind === "diagramFrom");
  const { wide, tall } = finding
    ? ROOM.finder
    : kind === "icon"
      ? ROOM.icons
      : kind === "newTask"
        ? ROOM.making
        : ROOM.narrow;
  const below = at.y + 6 + tall <= window.innerHeight;
  const placed = {
    width: `${wide}px`,
    left: Math.max(8, Math.min(at.x, window.innerWidth - wide - 18)),
    ...(below ? { top: at.y + 6 } : { bottom: Math.max(8, window.innerHeight - at.y + 24) }),
  };
  const making = useRef(false);
  const told = useRef({ onLost, onError });
  told.current = { onLost, onError };
  const done = (text?: string): boolean => {
    onDone();
    return text ? put(text) : true;
  };
  const landed = (text: string, lost: Word) => {
    const laid = done(text);
    if (!laid) told.current.onError(t(lost));
    return laid;
  };
  const carded = (file: string, title: string, card: string, lost: Word) => {
    const doc = { id: file, file, title, folder: null, archived: false, away: false };
    if (!done(card) && !told.current.onLost(doc)) told.current.onError(t(lost));
  };
  // A name already on its way to disk is not cancelled: its card has to land where it was asked for.
  const dismissed = () => {
    if (!making.current) done();
  };

  return (
    <>
      <span
        aria-hidden="true"
        data-keeps-writing
        className="fixed inset-0 z-30"
        onMouseDown={dismissed}
      />
      <div
        role="dialog"
        data-keeps-writing
        style={placed}
        onKeyDown={(e) => {
          if (e.key !== "Escape" || e.defaultPrevented) return;
          e.preventDefault();
          e.stopPropagation();
          dismissed();
        }}
        className={`fixed z-40 flex flex-col rounded-[10px] border border-hair bg-rail shadow-xl ${kind === "icon" ? "p-3" : "p-1.5"}`}
      >
        {(kind === "newPage" || kind === "newDoc") && (
          <Asking
            leaf={kind === "newPage"}
            onName={(name) => {
              if (making.current) return;
              making.current = true;
              const leaf = kind === "newPage";
              spawned(name, own?.folder ?? undefined, leaf ? own?.id : undefined)
                .then((born) => {
                  onMade({ id: born.id, title: name });
                  // The block may have closed while the name was typed; the card then goes at the end.
                  carded(born.id, name, born.said, "docMadeCardLost");
                })
                .catch((problem) => {
                  done();
                  told.current.onError(problem);
                });
            }}
          />
        )}
        {(kind === "page" || kind === "doc") && (
          <Papers
            all={kind === "page" ? pages : others}
            onPick={(one) => {
              const title = one.title || t("untitledDoc");
              carded(one.file, title, docCard(one.file, title), "docCardLost");
            }}
          />
        )}
        {kind === "newTask" && (
          <TaskMaking
            born={
              picked.whole ? { whole: picked.whole } : { doc: picked.file, before: picked.before }
            }
            onMaking={(now) => {
              making.current = now;
            }}
            onMade={(task) => {
              landed(taskCard(task.id, task.title), "taskMadeCardLost");
            }}
          />
        )}
        {kind === "task" && (
          <TaskPick
            onPick={(one) => {
              landed(taskCard(one.id, one.title), "taskCardLost");
            }}
            onError={onError}
          />
        )}
        {finding && (
          <DiagramFinder
            onPick={(source) => landed(fenced(source), "choiceLost")}
            onError={onError}
          />
        )}
        {kind === "diagram" && !finding && (
          <div role="menu" aria-label={t("newDiagram")} className="flex flex-col">
            {STARTS.map((one) => (
              <button
                key={one}
                type="button"
                role="menuitem"
                className="rounded-md px-2.5 py-1.5 text-left text-[13px] hover:bg-hover"
                onClick={() => {
                  const source = startOf(one, locale());
                  const write = () => landed(fenced(source), "choiceLost");
                  if (onDrawn) onDrawn(source, write);
                  else write();
                }}
              >
                {nameOf(one, locale())}
              </button>
            ))}
            <button
              type="button"
              role="menuitem"
              className="mt-1 rounded-md border-t border-hair px-2.5 py-1.5 text-left text-[13px] hover:bg-hover"
              onClick={() => setFinding(true)}
            >
              {t("diagramExisting")}
            </button>
          </div>
        )}
        {kind === "icon" && (
          <Glyphs
            onPick={(key, hue) =>
              landed(!known(key) ? key : hue ? `::${key} ${hue}::` : `::${key}::`, "choiceLost")
            }
          />
        )}
      </div>
    </>
  );
}
