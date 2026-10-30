import { useRef, useState } from "react";
import { fenced, nameOf, STARTS, startOf } from "tisty-diagrams/read";
import { known } from "tisty-editor/read";
import type { Filed } from "../core";
import { locale, t } from "../locales";
import { spawned } from "../making";
import { docCard } from "../markdown";
import Asking from "./Asking";
import DiagramFinder from "./DiagramFinder";
import Glyphs from "./Glyphs";
import Papers from "./Papers";
import type { Asked } from "./slashing";

export type Picked = {
  readonly kind: Asked;
  readonly at: { x: number; y: number };
  readonly put: (text: string) => boolean;
  readonly file?: string;
};

// The icons keep the room they have everywhere else, so they read the same here as in a list or a folder.
const ROOM = {
  narrow: { wide: 272, tall: 340 },
  icons: { wide: 400, tall: 430 },
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
  onLost: (doc: { file: string; title: string }) => void;
  onError: (problem: unknown) => void;
  onDone: () => void;
  onDrawn?: (source: string, write: () => boolean) => void;
}) {
  const { kind, at, put } = picked;
  const [finding, setFinding] = useState(kind === "diagramFrom");
  const { wide, tall } = finding ? ROOM.finder : kind === "icon" ? ROOM.icons : ROOM.narrow;
  const below = at.y + 6 + tall <= window.innerHeight;
  const placed = {
    width: `${wide}px`,
    left: Math.max(8, Math.min(at.x, window.innerWidth - wide - 18)),
    ...(below ? { top: at.y + 6 } : { bottom: Math.max(8, window.innerHeight - at.y + 24) }),
  };
  const making = useRef(false);
  const done = (text?: string): boolean => {
    onDone();
    return text ? put(text) : true;
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
          if (e.key !== "Escape") return;
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
                  if (!done(born.said)) onLost({ file: born.id, title: name });
                })
                .catch((problem) => {
                  done();
                  onError(problem);
                });
            }}
          />
        )}
        {(kind === "page" || kind === "doc") && (
          <Papers
            all={kind === "page" ? pages : others}
            onPick={(one) => {
              const title = one.title || t("untitledDoc");
              if (!done(docCard(one.file, title))) onLost({ file: one.file, title });
            }}
          />
        )}
        {finding && <DiagramFinder onPick={(source) => done(fenced(source))} onError={onError} />}
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
                  const write = () => done(fenced(source));
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
              done(!known(key) ? key : hue ? `::${key} ${hue}::` : `::${key}::`)
            }
          />
        )}
      </div>
    </>
  );
}
