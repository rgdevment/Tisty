import type { Filed } from "../core";
import { known } from "../glyphs";
import { t } from "../locales";
import { spawned } from "../making";
import { docCard } from "../markdown";
import Asking from "./Asking";
import Glyphs from "./Glyphs";
import Papers from "./Papers";
import type { Asked } from "./slashing";

export type Picked = {
  readonly kind: Asked;
  readonly at: { x: number; y: number };
  readonly put: (text: string) => void;
};

// The icons keep the room they have everywhere else, so they read the same here as in a list or a folder.
const ROOM = { narrow: { wide: 272, tall: 340 }, icons: { wide: 400, tall: 430 } };

// What the / menu asks the window for: a name for a new page, a document to point at, an icon.
export default function Picking({
  picked,
  own,
  pages,
  others,
  onMade,
  onDone,
}: {
  picked: Picked;
  own?: Filed;
  pages: Filed[];
  others: Filed[];
  onMade: (doc: { id: string; title: string }) => void;
  onDone: () => void;
}) {
  const { kind, at, put } = picked;
  const { wide, tall } = kind === "icon" ? ROOM.icons : ROOM.narrow;
  const below = at.y + 6 + tall <= window.innerHeight;
  const placed = {
    width: `${wide}px`,
    left: Math.max(8, Math.min(at.x, window.innerWidth - wide - 18)),
    ...(below ? { top: at.y + 6 } : { bottom: Math.max(8, window.innerHeight - at.y + 24) }),
  };
  const done = (text?: string) => {
    onDone();
    if (text) put(text);
  };

  return (
    <>
      <span aria-hidden="true" className="fixed inset-0 z-30" onMouseDown={() => done()} />
      <div
        role="dialog"
        style={placed}
        onKeyDown={(e) => {
          if (e.key !== "Escape") return;
          e.preventDefault();
          e.stopPropagation();
          done();
        }}
        className={`fixed z-40 flex flex-col rounded-[10px] border border-hair bg-rail shadow-xl ${kind === "icon" ? "p-3" : "p-1.5"}`}
      >
        {(kind === "newPage" || kind === "newDoc") && (
          <Asking
            leaf={kind === "newPage"}
            onName={(name) => {
              const leaf = kind === "newPage";
              spawned(name, own?.folder ?? undefined, leaf ? own?.id : undefined)
                .then((born) => {
                  onMade({ id: born.id, title: name });
                  done(born.said);
                })
                .catch(() => done());
            }}
          />
        )}
        {(kind === "page" || kind === "doc") && (
          <Papers
            all={kind === "page" ? pages : others}
            onPick={(one) => done(docCard(one.file, one.title || t("untitledDoc")))}
          />
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
