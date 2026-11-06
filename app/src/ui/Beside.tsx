import { useEffect, useMemo, useRef, useState } from "react";
import { type Facts, fields, type Head, type Leaf, overgrown, type Panel } from "tisty-editor/read";
import type { Filed, Folded } from "../core";
import { type DocFacts, docFacts, type Paper } from "../core";
import { weigh } from "../format";
import { waysOut } from "../leaving";
import { t } from "../locales";
import { tongueName } from "../tongues";
import Menu, { type Choice } from "./Menu";
import { dated, held, holdsOf } from "./paperFacts";
import Sharing from "./Sharing";
import SidePanels from "./SidePanels";
import { useSharedRow } from "./sharedRow";
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
// How long the words rest before the column counts them again.
const SETTLES = 300;
// Within this of the menu closing, a press on its button is the same press, not a new one.
const REOPENS = 400;

// Typing changes the words on every key; the column follows once they rest, and at once for another document.
const useSettled = <T,>(now: T, key: string): T => {
  const [held, setHeld] = useState({ key, now });
  useEffect(() => {
    const resting = setTimeout(() => setHeld({ key, now }), SETTLES);
    return () => clearTimeout(resting);
  }, [now, key]);
  return held.key === key ? held.now : now;
};

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
  /** The languages the document is told in, the one open now marked. */
  tongues?: readonly Filed[];
  onTongue?: (tongue: Filed) => void;
  doc?: string;
  onError?: (problem: unknown) => void;
  onKeep?: () => Promise<unknown> | undefined;
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
  const { paper, kept, pages, up, trail = [] } = props;
  const heads = useSettled(props.heads, paper);
  const body = useSettled(props.body, paper);
  const outline = heads.filter((one) => one.level > 0);
  const short = outline.length + pages.length <= SHORT_OUTLINE;
  const [facts, setFacts] = useState<DocFacts | null>(null);
  const [out, setOut] = useState<{ x: number; y: number } | null>(null);
  // Pressing the button closes the open menu as a press outside it; the click that follows must not open it again.
  const shutAt = useRef(0);

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

  const details = useFacts({ ...props, body, heads }, facts);
  const shared = useSharedRow(props.doc, props.onError, props.onKeep);
  const ways: Choice[] = [
    ...waysOut(false, props.making),
    { key: "copy", icon: "⧉", label: t("copyPlain"), apart: true, onPick: props.onCopy },
    { key: "save", icon: "⇩", label: t("saveAsMarkdown"), onPick: props.onTakeOut },
  ];
  const sheets = props.sheets ?? false;
  // The column is drawn again only when what it shows changes, so its presses ask for the window's latest.
  const latest = useRef(props);
  latest.current = props;
  const canSheets = props.onSheets !== undefined;
  const canTurn = props.onLandscape !== undefined;
  // Arrays a render makes anew are compared by what they say, or every key pressed would draw the column again.
  const trailSaid = trail.map((one) => `${one.id}:${one.name}`).join("|");
  const pagesSaid = pages.map((one) => `${one.id}:${one.file}:${one.title}`).join("|");
  const tonguesSaid = (props.tongues ?? []).map((one) => `${one.id}:${one.lang}`).join("|");
  // biome-ignore lint/correctness/useExhaustiveDependencies: the presses read the window's latest through `latest`, and arrays by what they say
  const panels = useMemo(
    (): Panel[] => [
      {
        kind: "title",
        title: props.title || t("untitledDoc"),
        trail: [
          { key: "all", label: t("everyPaper"), onPick: () => latest.current.onFolder?.(null) },
          ...trail.map((one) => ({
            key: one.id,
            label: one.name,
            onPick: () => latest.current.onFolder?.(one.id),
          })),
        ],
        tongues: (props.tongues ?? []).map((one) => ({
          key: one.id,
          label: tongueName(one.lang ?? ""),
          lang: one.lang ?? "",
          on: one.file === paper || one.id === up?.id,
          onPick: () => latest.current.onTongue?.(one),
        })),
        onApart: () => latest.current.onApart(),
        onShut: () => latest.current.onShut(),
      },
      {
        kind: "paper",
        leaf: props.leaf,
        onLeaf: (one) => latest.current.onLeaf(one),
        sheets: canSheets
          ? { on: sheets, onTurn: (on) => latest.current.onSheets?.(on) }
          : undefined,
        landscape:
          sheets && canTurn
            ? {
                on: props.landscape ?? false,
                onTurn: (turned) => latest.current.onLandscape?.(turned),
              }
            : undefined,
        exporting: props.making,
        onExport: (at) => {
          if (performance.now() - shutAt.current < REOPENS) return;
          const box = at.getBoundingClientRect();
          setOut((was) => (was ? null : { x: box.right - 230, y: box.bottom + 4 }));
        },
      },
      {
        kind: "index",
        heads,
        book: {
          up: up && { title: up.title, onPick: () => latest.current.onPage?.(up) },
          said: held(pages.length),
          pages: pages.map((one) => leafOf(one, paper, (doc) => latest.current.onPage?.(doc))),
        },
      },
      { kind: "details", facts: details, open: short },
      ...(shared.panel ? [shared.panel] : []),
    ],
    [
      props.title,
      trailSaid,
      props.leaf,
      canSheets,
      canTurn,
      sheets,
      props.landscape,
      props.making,
      heads,
      up?.id,
      up?.title,
      pagesSaid,
      tonguesSaid,
      paper,
      details,
      short,
      shared.panel,
    ],
  );

  return (
    <aside
      data-beside=""
      aria-label={t("aboutPaper")}
      className="absolute top-11 right-3 bottom-3 flex w-[320px] flex-col"
    >
      <SidePanels key={paper} panels={panels} className="min-h-0 flex-1" />
      {shared.asking && props.doc && (
        <Sharing
          doc={props.doc}
          title={props.title || t("untitledDoc")}
          termsOnly={shared.termsOnly}
          onShared={shared.onShared}
          onClose={shared.onClosed}
          onError={props.onError}
        />
      )}
      {out && (
        <Menu
          at={out}
          choices={ways}
          label={t("exportIt")}
          onClose={() => {
            shutAt.current = performance.now();
            setOut(null);
          }}
        />
      )}
    </aside>
  );
}
