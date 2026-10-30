import { useEffect, useMemo, useState } from "react";
import { type Facts, fields, type Head, type Leaf, overgrown, type Panel } from "tisty-editor/read";
import type { Filed, Folded } from "../core";
import { type DocFacts, docFacts, type Paper } from "../core";
import { weigh } from "../format";
import { waysOut } from "../leaving";
import { t } from "../locales";
import Menu, { type Choice } from "./Menu";
import { dated, held, holdsOf } from "./paperFacts";
import SidePanels from "./SidePanels";
import { often } from "./tagging";

export const trailed = (folders: Folded[], at: string | null): Folded[] => {
  const by = new Map(folders.map((one) => [one.id, one]));
  const walk: Folded[] = [];
  const seen = new Set<string>();
  for (let up = at; up && !seen.has(up); ) {
    seen.add(up);
    const one = by.get(up);
    if (!one) break;
    walk.unshift(one);
    up = one.parent;
  }
  return walk;
};

// An outline this short leaves room, so the details open with the document.
const SHORT_OUTLINE = 6;

interface Props {
  title: string;
  paper: string;
  body: string;
  tags: readonly string[];
  kept: number;
  heads: Head[];
  pages: Filed[];
  /** The document a page belongs to, so its column can take the person back. */
  up?: Filed;
  onPage?: (doc: Filed) => void;
  leaf: Paper;
  onLeaf: (leaf: Paper) => void;
  sheets?: boolean;
  landscape?: boolean;
  onSheets?: (on: boolean) => void;
  onLandscape?: (turned: boolean) => void;
  making: boolean;
  onCopy: () => void;
  onTakeOut: () => void;
  onShut: () => void;
  onApart: () => void;
  trail?: Folded[];
  onFolder?: (id: string | null) => void;
}

const leafOf = (one: Filed, paper: string, onPage?: (doc: Filed) => void): Leaf => ({
  key: one.id,
  title: one.title,
  current: one.file === paper,
  away: one.away,
  archived: one.archived,
  flagged: Boolean(one.flagged),
  onPick: () => onPage?.(one),
});

// Read once per version of the text and its facts: the column is drawn again far more often than either changes.
const useFacts = (props: Props, facts: DocFacts | null): Facts => {
  const { body, tags, heads } = props;
  const onFront = heads.find((one) => one.level === 0)?.go;
  return useMemo(() => {
    const times = often(body);
    const holds = holdsOf(body, facts).join(" · ");
    return {
      body,
      author: facts?.author ?? undefined,
      born: facts?.born ?? undefined,
      editor: facts?.editor ?? undefined,
      made: facts ? dated(facts.made) : undefined,
      wrote: facts ? dated(facts.wrote) : undefined,
      weighs: facts ? weigh(facts.bytes) : undefined,
      holds: holds || undefined,
      fields: fields(body),
      overgrown: overgrown(body),
      onFront,
      tags: tags.map((name) => ({ name, times: times.get(name) ?? 0 })),
    };
  }, [body, facts, tags, onFront]);
};

export default function Beside(props: Props) {
  const { paper, kept, heads, pages, up, onPage, trail = [], onFolder } = props;
  const outline = heads.filter((one) => one.level > 0);
  const short = outline.length + pages.length <= SHORT_OUTLINE;
  const [facts, setFacts] = useState<DocFacts | null>(null);
  const [out, setOut] = useState<{ x: number; y: number } | null>(null);

  useEffect(() => {
    let gone = false;
    docFacts(paper)
      .then((now) => {
        if (!gone) setFacts(now);
      })
      .catch(() => {});
    return () => {
      gone = true;
    };
  }, [paper, kept]);

  const details = useFacts(props, facts);
  const ways: Choice[] = [
    ...waysOut(false, props.making),
    { key: "copy", icon: "⧉", label: t("copyPlain"), apart: true, onPick: props.onCopy },
    { key: "save", icon: "⇩", label: t("saveAsMarkdown"), onPick: props.onTakeOut },
  ];
  const sheets = props.sheets ?? false;
  const { onSheets, onLandscape } = props;
  const panels: Panel[] = [
    {
      kind: "title",
      title: props.title || t("untitledDoc"),
      trail: [
        { key: "all", label: t("everyPaper"), onPick: () => onFolder?.(null) },
        ...trail.map((one) => ({ key: one.id, label: one.name, onPick: () => onFolder?.(one.id) })),
      ],
      onApart: props.onApart,
      onShut: props.onShut,
    },
    {
      kind: "paper",
      leaf: props.leaf,
      onLeaf: props.onLeaf,
      sheets: onSheets && { on: sheets, onTurn: onSheets },
      landscape:
        sheets && onLandscape ? { on: props.landscape ?? false, onTurn: onLandscape } : undefined,
      exporting: props.making,
      onExport: (at) => {
        const box = at.getBoundingClientRect();
        setOut((was) => (was ? null : { x: box.right - 230, y: box.bottom + 4 }));
      },
    },
    {
      kind: "index",
      heads,
      book: {
        up: up && { title: up.title, onPick: () => onPage?.(up) },
        said: held(pages.length),
        pages: pages.map((one) => leafOf(one, paper, onPage)),
      },
    },
    { kind: "details", facts: details, open: short },
    { kind: "shared", gist: t("sharedSoon"), said: t("sharedWill") },
  ];

  return (
    <aside
      data-beside=""
      aria-label={t("aboutPaper")}
      className="absolute top-11 right-3 bottom-3 flex w-[320px] flex-col"
    >
      <SidePanels key={paper} panels={panels} className="min-h-0 flex-1" />
      {out && <Menu at={out} choices={ways} label={t("exportIt")} onClose={() => setOut(null)} />}
    </aside>
  );
}
