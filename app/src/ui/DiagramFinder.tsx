import { useEffect, useState } from "react";
import { useAttended } from "../attended";
import { diagramsEverywhere, type FoundDiagram } from "../drawing";
import { matched } from "../finding";
import { t } from "../locales";
import { svgFor } from "./drawing/exporting";
import { kindOf } from "./drawing/framing";
import { firstWords, kindName } from "./drawing/starting";
import { darkNow } from "./viewer/acting";

const titled = (one: FoundDiagram) => one.title.trim() || t("untitledDoc");

// Only the diagram being looked at is drawn: drawing every one at once would hold the list back.
const usePreview = (source: string | undefined) => {
  const [shot, setShot] = useState<string | null>(null);
  useEffect(() => {
    if (source === undefined) return setShot(null);
    let live = true;
    const font = getComputedStyle(document.body).fontFamily || "sans-serif";
    svgFor(source, darkNow(), false, font)
      .then((drawn) => {
        if (live) setShot(`data:image/svg+xml;charset=utf-8,${encodeURIComponent(drawn.svg)}`);
      })
      .catch(() => {
        if (live) setShot(null);
      });
    return () => {
      live = false;
    };
  }, [source]);
  return shot;
};

export default function DiagramFinder({
  onPick,
  onError,
}: Readonly<{ onPick: (source: string) => void; onError?: (problem: unknown) => void }>) {
  const [found, setFound] = useState<FoundDiagram[] | null>(null);
  const [word, setWord] = useState("");
  const [chosen, setChosen] = useState(0);
  const held = useAttended<HTMLInputElement>();

  useEffect(() => {
    diagramsEverywhere()
      .then(setFound)
      .catch((problem) => {
        setFound([]);
        onError?.(problem);
      });
  }, [onError]);

  const shown = (found ?? []).filter((one) =>
    matched(`${titled(one)} ${kindName(kindOf(one.source))}`, word),
  );
  const current = shown[Math.min(chosen, shown.length - 1)];
  const shot = usePreview(current?.source);

  const keyed = (e: React.KeyboardEvent) => {
    const moves: Record<string, number> = { ArrowDown: 1, ArrowUp: -1 };
    if (e.key in moves) {
      e.preventDefault();
      setChosen((at) => Math.max(0, Math.min(shown.length - 1, at + moves[e.key])));
    }
    if (e.key === "Enter" && current) {
      e.preventDefault();
      onPick(current.source);
    }
  };

  return (
    <div className="flex flex-col">
      <input
        ref={held}
        value={word}
        aria-label={t("diagramFind")}
        placeholder={t("diagramFind")}
        onChange={(e) => {
          setWord(e.target.value);
          setChosen(0);
        }}
        onKeyDown={keyed}
        className="mb-1 w-full rounded-md bg-hover px-2.5 py-1.5 outline-none placeholder:text-faint"
      />
      <div className="flex min-h-0 gap-2">
        <div
          role="listbox"
          aria-label={t("diagramExisting")}
          className="scroller h-[300px] w-[260px] shrink-0"
        >
          {found === null && <p className="px-2.5 py-1.5 text-faint">{t("opening")}</p>}
          {found !== null && shown.length === 0 && (
            <p className="px-2.5 py-1.5 text-faint">
              {found.length ? t("noneHere") : t("diagramNoneYet")}
            </p>
          )}
          {shown.map((one, at) => (
            <button
              key={`${one.file}-${one.nth}`}
              type="button"
              role="option"
              aria-selected={one === current}
              onMouseEnter={() => setChosen(at)}
              onClick={() => onPick(one.source)}
              className={`flex w-full flex-col rounded-md px-2.5 py-1.5 text-left ${one === current ? "bg-hover" : ""}`}
            >
              <span className="truncate text-ink">{titled(one)}</span>
              <span className="truncate text-[11.5px] text-faint">
                {`${kindName(kindOf(one.source))} · ${firstWords(one.source)}`}
              </span>
            </button>
          ))}
        </div>
        <div className="flex h-[300px] min-w-0 flex-1 items-center justify-center rounded-md bg-desk p-2">
          {shot && <img src={shot} alt="" className="max-h-full max-w-full object-contain" />}
        </div>
      </div>
    </div>
  );
}
