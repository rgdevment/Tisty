import { lazy, Suspense, useEffect, useRef, useState } from "react";
import { copied, type DocFacts, type Paper } from "../../core";
import { fill, t } from "../../locales";
import PaperFacts from "../PaperFacts";
import { LEAVES, leafWord, PAPER, TALL } from "../sheets";
import type { Head } from "../writing";
import { button, Segments } from "./Glyphs";

const Editor = lazy(() => import("../Editor"));

interface Props {
  body: string;
  title: string;
  paper?: string;
  stranger: boolean;
  side: boolean;
  raw: boolean;
  zoom: number;
  leaf: Paper;
  facts: DocFacts | null;
  finding: boolean;
  onLeaf: (leaf: Paper) => void;
  onRaw: (raw: boolean) => void;
  onFound: () => void;
  onDoc: (file: string) => void;
}

const heading = "mx-2 mb-2 text-[10.5px] font-semibold uppercase tracking-[0.06em] text-faint";

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

function Finder({ onClose }: { onClose: () => void }) {
  const field = useRef<HTMLInputElement>(null);
  useEffect(() => field.current?.focus(), []);
  const seek = (said: string, back: boolean) => {
    const finds = (window as unknown as { find?: (...args: unknown[]) => boolean }).find;
    if (said.trim()) finds?.call(window, said, false, back, true);
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

export default function Reader(props: Props) {
  const { body, title, stranger, side, raw, zoom, leaf, finding } = props;
  const [heads, setHeads] = useState<Head[]>([]);
  const wide = PAPER[leaf] + 40;

  const go = (one: Head) => {
    if (raw) props.onRaw(false);
    requestAnimationFrame(() => requestAnimationFrame(one.go));
  };

  return (
    <div className="flex min-h-0 flex-1">
      {side && (
        <nav className="flex w-[264px] shrink-0 flex-col overflow-hidden border-r border-hair bg-panel px-2 py-3.5">
          <h6 className={heading}>{t(stranger ? "viewInFile" : "viewInDoc")}</h6>
          <div className="scroller flex min-h-0 flex-1 flex-col">
            {heads.map((one) => (
              <button
                type="button"
                key={one.key}
                onClick={() => go(one)}
                className="shrink-0 truncate rounded-md py-1 pr-2 text-left text-[12.5px] text-soft hover:bg-hover hover:text-ink"
                style={{ paddingLeft: 8 + Math.max(0, one.level - 1) * 12 }}
              >
                {one.text}
              </button>
            ))}
          </div>
          <div className="mt-3 shrink-0 border-t border-hair pt-3.5">
            <h6 className={heading}>{t(stranger ? "aboutFile" : "aboutPaper")}</h6>
            <div className="mx-2 mb-4">
              <PaperFacts facts={props.facts} body={body} />
            </div>
            <h6 className={heading}>{t("viewPage")}</h6>
            <div className="mx-1">
              <Segments
                wide
                value={leaf}
                options={LEAVES.map((one) => ({ key: one, label: t(leafWord(one)) }))}
                onPick={props.onLeaf}
              />
            </div>
            <div className="mx-1.5 mt-2.5 flex items-center gap-2.5 text-[11.5px] leading-snug text-faint">
              <span
                className="w-[26px] shrink-0 rounded-md border-[1.5px] border-faint bg-sheet transition-[height]"
                style={{ height: Math.round(26 * TALL[leaf]) }}
              />
              <span>{fill("viewPageAbout", String(PAPER[leaf]))}</span>
            </div>
          </div>
        </nav>
      )}
      <div className="viewed-doc relative flex min-w-0 flex-1 flex-col">
        {finding && <Finder onClose={props.onFound} />}
        {raw ? (
          <Raw body={body} wide={wide} />
        ) : (
          <div
            className="mx-auto flex min-h-0 w-full flex-1 flex-col px-5 transition-[max-width]"
            style={{ maxWidth: wide, zoom }}
          >
            <Suspense fallback={<p className="text-[12.5px] text-faint">{t("opening")}</p>}>
              <Editor
                value={body}
                reading
                stranger={stranger}
                label={title}
                paper={props.paper}
                onWrite={() => {}}
                onOutline={setHeads}
                onDoc={props.onDoc}
              />
            </Suspense>
          </div>
        )}
      </div>
    </div>
  );
}
