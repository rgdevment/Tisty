import { lazy, Suspense, useEffect, useMemo, useRef, useState } from "react";
import {
  acrossOf,
  type Facts,
  fields,
  type Head,
  PAPER,
  type Panel,
  sheetFor,
} from "tisty-editor/read";
import { copied, type DocFacts, type Paper } from "../../core";
import { weigh } from "../../format";
import { t } from "../../locales";
import { dated, holdsOf } from "../PaperFacts";
import SidePanels from "../SidePanels";
import { usePictures } from "../usePictures";
import { button } from "./Glyphs";

const Page = lazy(() => import("../Page"));

interface Props {
  body: string;
  title: string;
  under: string;
  stranger: boolean;
  side: boolean;
  raw: boolean;
  zoom: number;
  leaf: Paper;
  sheets: boolean;
  landscape: boolean;
  facts: DocFacts | null;
  finding: boolean;
  onLeaf: (leaf: Paper) => void;
  onSheets: (on: boolean) => void;
  onLandscape: (across: boolean) => void;
  onRaw: (raw: boolean) => void;
  onEdit: () => void;
  onExport: (at: { x: number; y: number }) => void;
  onFound: () => void;
  onDoc: (file: string) => void;
  onOpen?: (reference: string) => void;
  /** The page as it is drawn, for whoever prints it the way it is seen. */
  onPage?: (page: Window | null) => void;
}

// What the file and its front matter say about it, in the window's words for dates.
const factsOf = (body: string, facts: DocFacts | null): Facts => ({
  body,
  author: facts?.author ?? undefined,
  born: facts?.born ?? undefined,
  editor: facts?.editor ?? undefined,
  made: facts?.made ? dated(facts.made) : undefined,
  wrote: facts?.wrote ? dated(facts.wrote) : undefined,
  weighs: facts ? weigh(facts.bytes) : undefined,
  holds: holdsOf(body, facts).join(" · ") || undefined,
  fields: fields(body),
});

function Raw({ body, wide }: { body: string; wide: number }) {
  const [done, setDone] = useState(false);
  const copy = () =>
    copied(body)
      .then(() => {
        setDone(true);
        setTimeout(() => setDone(false), 1200);
      })
      .catch(() => {});
  return (
    <div className="scroller min-h-0 flex-1 px-5 pt-1 pb-10">
      <div
        className="relative mx-auto rounded-[10px] bg-sheet py-6 pr-7 pl-4 font-mono text-[13px] leading-[1.65] shadow-[0_1px_2px_rgb(0_0_0/0.06)]"
        style={{ maxWidth: wide }}
      >
        <button type="button" onClick={copy} className={`absolute top-3 right-3.5 ${button}`}>
          {t(done ? "viewCopied" : "viewCopy")}
        </button>
        {body.split("\n").map((line, at) => (
          // biome-ignore lint/suspicious/noArrayIndexKey: a line of one text has no identity but its place
          <div key={at} className="flex whitespace-pre-wrap break-words">
            <span className="mr-4 w-[30px] shrink-0 select-none pt-[2px] text-right text-[11.5px] text-faint">
              {at + 1}
            </span>
            <span className={/^\s{0,3}#/.test(line) ? "text-accent" : undefined}>
              {line || " "}
            </span>
          </div>
        ))}
      </div>
    </div>
  );
}

type Finding = { find?: (...args: unknown[]) => boolean };

function Finder({ onClose, within }: { onClose: () => void; within: () => Window | null }) {
  const field = useRef<HTMLInputElement>(null);
  useEffect(() => field.current?.focus(), []);
  const seek = (said: string, back: boolean) => {
    const place = within() ?? window;
    const finds = (place as unknown as Finding).find;
    if (said.trim()) finds?.call(place, said, false, back, true);
  };
  return (
    <div className="absolute top-2 right-4 z-10 flex items-center gap-1.5 rounded-[10px] border border-line bg-sheet px-2 py-1.5 shadow-[0_10px_30px_rgb(0_0_0/0.16)]">
      <input
        ref={field}
        type="search"
        placeholder={t("viewFind")}
        aria-label={t("viewFind")}
        className="w-52 bg-transparent text-[12.5px] outline-none placeholder:text-faint"
        onKeyDown={(e) => {
          if (e.key === "Enter") seek(e.currentTarget.value, e.shiftKey);
          if (e.key === "Escape") onClose();
        }}
      />
    </div>
  );
}

export default function Reader(props: Readonly<Props>) {
  const { body, title, stranger, side, raw, zoom, leaf, sheets, landscape, finding } = props;
  const [heads, setHeads] = useState<Head[]>([]);
  const pictures = usePictures({ open: props.onOpen });
  const shown = useRef<Window | null>(null);
  const wide = sheets ? acrossOf(leaf, landscape) : PAPER[leaf] + 40;
  const sheet = useMemo(
    () => (sheets ? sheetFor(leaf, landscape, t("sheetsBreak")) : undefined),
    [sheets, landscape, leaf],
  );

  const sought = useRef<string | null>(null);
  const outlined = (found: Head[]) => {
    setHeads(found);
    const pending = found.find((one) => one.key === sought.current);
    sought.current = null;
    pending?.go();
  };

  // The panels are drawn again only when what they show changes, so their presses ask for the window's latest.
  const latest = useRef(props);
  latest.current = props;

  const go = (one: Head) => {
    if (!latest.current.raw) return one.go();
    sought.current = one.key;
    latest.current.onRaw(false);
  };

  // biome-ignore lint/correctness/useExhaustiveDependencies: `go` and the presses read the window's latest through `latest`
  const panels = useMemo(
    (): Panel[] => [
      { kind: "title", title, under: props.under },
      {
        kind: "paper",
        leaf,
        onLeaf: (one) => latest.current.onLeaf(one),
        sheets: { on: sheets, onTurn: (on) => latest.current.onSheets(on) },
        landscape: sheets
          ? { on: landscape, onTurn: (on) => latest.current.onLandscape(on) }
          : undefined,
        raw: { on: raw, onTurn: (on) => latest.current.onRaw(on) },
      },
      { kind: "index", heads: heads.map((one) => ({ ...one, go: () => go(one) })) },
      { kind: "details", facts: factsOf(body, props.facts) },
      {
        kind: "acts",
        acts: [
          {
            key: "edit",
            label: t("viewEditIn"),
            main: true,
            onPick: () => latest.current.onEdit(),
          },
          {
            key: "export",
            label: `${t("exportIt")} ▾`,
            onPick: (at) => {
              const box = at.getBoundingClientRect();
              latest.current.onExport({ x: box.left, y: box.bottom + 4 });
            },
          },
        ],
      },
    ],
    [title, props.under, leaf, sheets, landscape, raw, heads, body, props.facts],
  );

  return (
    <div className="flex min-h-0 flex-1">
      <div className="viewed-doc relative flex min-w-0 flex-1 flex-col">
        {finding && <Finder onClose={props.onFound} within={() => shown.current} />}
        {raw ? (
          <Raw body={body} wide={wide} />
        ) : (
          <Suspense fallback={<p className="text-[12.5px] text-faint">{t("opening")}</p>}>
            <Page
              body={body}
              label={title}
              stranger={stranger}
              wide={wide}
              zoom={zoom}
              sheet={sheet}
              onOutline={outlined}
              onDoc={props.onDoc}
              onOpen={props.onOpen}
              onPicture={pictures.view}
              onShown={(at) => {
                shown.current = at;
                props.onPage?.(at);
              }}
            />
          </Suspense>
        )}
        {pictures.viewer}
      </div>
      {side && (
        <SidePanels panels={panels} className="w-[300px] shrink-0 overflow-y-auto py-3.5 pr-3.5" />
      )}
    </div>
  );
}
