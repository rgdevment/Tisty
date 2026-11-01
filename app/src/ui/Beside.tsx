import { useEffect, useMemo, useRef, useState } from "react";
import {
  type Facts,
  fields,
  type Head,
  type Leaf,
  type Link,
  overgrown,
  type Panel,
} from "tisty-editor/read";
import type { Filed, Folded } from "../core";
import { type DocFacts, docFacts, type Paper } from "../core";
import { dayOf, weigh } from "../format";
import { waysOut } from "../leaving";
import { fill, t } from "../locales";
import { type ShareCard, shareCard, shareChange, shareStill, shareStop } from "../shares";
import Menu, { type Choice } from "./Menu";
import { dated, held, holdsOf } from "./paperFacts";
import ShareSettings from "./ShareSettings";
import Sharing from "./Sharing";
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
  onError: (problem: unknown) => void;
  /** Saves what is still waiting in the editor, so what «Share» looks through is what it sends. */
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

// How a share stands, in a few words: who can open it, when it ends and how often it was read.
const gistOf = (card: ShareCard | null): string => {
  if (!card) return t("sharedNot");
  const info = card.info;
  if (info?.blocked) return t("sharedBlocked");
  if (info?.paused) return t("sharedPaused");
  const reads =
    info && (info.reads === 1 ? t("sharedReadsOne") : fill("sharedReads", String(info.reads)));
  return [
    t(card.gated ? "sharedGated" : "sharedUnlisted"),
    info ? fill("sharedEnds", dayOfSeconds(info.ends)) : "",
    reads ?? "",
  ]
    .filter(Boolean)
    .join(" · ");
};

// The row warns this long before an idle share runs out, with no notice of its own (decision 50).
const ENDS_SOON = 7 * 86_400;

const dayOfSeconds = (at: number) => dayOf(new Date(at * 1000).toISOString());

// What the row says under its gist: why it is blocked or paused, when it runs out soon, or where its links live.
const saidOf = (card: ShareCard): string => {
  const info = card.info;
  if (!info) return t("sharedOffline");
  if (info.blocked) return t("sharedBlockedWhy");
  if (info.paused) return fill("sharedPausedWhy", dayOfSeconds(info.ends));
  if (info.ends - Date.now() / 1000 < ENDS_SOON)
    return fill("sharedEndsSoon", dayOfSeconds(info.ends));
  return `${t(card.still ? "sharedStill" : "sharedFollows")} ${t("shareAdminKept")}`;
};

const toneOf = (card: ShareCard): "good" | "warn" | "bad" | undefined => {
  const info = card.info;
  if (!info) return undefined;
  if (info.blocked) return "bad";
  return info.paused || info.ends - Date.now() / 1000 < ENDS_SOON ? "warn" : "good";
};

// The «Shared» row: an offer to share, or the links to hand out and ways to change or stop it.
const sharedPanel = (
  card: ShareCard | null,
  facts: DocFacts | null,
  asked: {
    onShare: () => void;
    onSettings: () => void;
    onPause: (paused: boolean) => void;
    onStill: (still: boolean) => void;
    onStop: () => void;
  },
): Panel => {
  // Someone else's document is not offered at all (decision 68); until its facts arrive nothing is offered either.
  if (!card && facts?.shareable === "notMine")
    return { kind: "shared", gist: t("sharedNotMine"), said: t("sharedNotMineWhy") };
  if (!card && facts?.shareable === "noKey")
    return { kind: "shared", gist: t("sharedNoKey"), said: t("sharedNoKeyWhy") };
  if (!card)
    return {
      kind: "shared",
      gist: t("sharedNot"),
      said: t("sharedWhy"),
      acts: facts
        ? [{ key: "share", label: t("shareIt"), main: true, onPick: asked.onShare }]
        : undefined,
    };
  // Someone else's share edited here: its link to read, and a way to stop editing it; theirs to manage.
  if (card.taken)
    return {
      kind: "shared",
      gist: t("sharedTaken"),
      said: t("sharedTakenWhy"),
      links: [{ key: "read", label: t("shareLinkRead"), url: card.read }],
      acts: [{ key: "stop", label: t("shareStopEditing"), onPick: asked.onStop }],
    };
  const links: Link[] = [{ key: "read", label: t("shareLinkRead"), url: card.read }];
  if (card.edit)
    links.push({
      key: "edit",
      label: t("shareLinkEdit"),
      url: card.edit,
      note: t("shareLinkEditNote"),
    });
  // The server refuses everything but a look on a blocked document, so nothing is offered on one.
  const blocked = card.info?.blocked ?? false;
  const paused = card.info?.paused ?? false;
  // Unreached, the server cannot be asked to change anything, so only stopping stays.
  const changes = card.info
    ? [
        { key: "settings", label: t("shareSettings"), onPick: asked.onSettings },
        {
          key: "still",
          label: t(card.still ? "shareFollowIt" : "shareStillIt"),
          onPick: () => asked.onStill(!card.still),
        },
        {
          key: "pause",
          label: t(paused ? "shareResume" : "sharePause"),
          onPick: () => asked.onPause(!paused),
        },
      ]
    : [];
  return {
    kind: "shared",
    gist: gistOf(card),
    tone: toneOf(card),
    said: saidOf(card),
    links: blocked ? [] : links,
    acts: blocked ? [] : [...changes, { key: "stop", label: t("shareStop"), onPick: asked.onStop }],
  };
};

// The card is asked for once per document, and again only when this column changes it.
const useShared = (paper: string) => {
  const [card, setCard] = useState<ShareCard | null>(null);
  useEffect(() => {
    let gone = false;
    setCard(null);
    shareCard(paper)
      .then((now) => {
        if (!gone) setCard(now);
      })
      .catch(() => {});
    return () => {
      gone = true;
    };
  }, [paper]);
  return [card, setCard] as const;
};

export default function Beside(props: Props) {
  const { paper, kept, pages, up, trail = [] } = props;
  const heads = useSettled(props.heads, paper);
  const body = useSettled(props.body, paper);
  const outline = heads.filter((one) => one.level > 0);
  const short = outline.length + pages.length <= SHORT_OUTLINE;
  const [facts, setFacts] = useState<DocFacts | null>(null);
  const [out, setOut] = useState<{ x: number; y: number } | null>(null);
  const [card, setCard] = useShared(paper);
  const [sharing, setSharing] = useState(false);
  const [settings, setSettings] = useState(false);
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
      sharedPanel(card, facts, {
        onShare: () => {
          void Promise.resolve(latest.current.onKeep?.())
            .catch(() => {})
            .finally(() => setSharing(true));
        },
        onSettings: () => setSettings(true),
        onStill: (still) => shareStill(paper, still).then(setCard).catch(latest.current.onError),
        onPause: (pausing) =>
          shareChange(paper, { do: pausing ? "pause" : "resume" })
            .then(setCard)
            .catch(latest.current.onError),
        onStop: () =>
          shareStop(paper)
            .then(() => setCard(null))
            .catch(latest.current.onError),
      }),
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
      paper,
      details,
      short,
      card,
      facts,
    ],
  );

  return (
    <aside
      data-beside=""
      aria-label={t("aboutPaper")}
      className="absolute top-11 right-3 bottom-3 flex w-[320px] flex-col"
    >
      <SidePanels key={paper} panels={panels} className="min-h-0 flex-1" />
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
      {sharing && (
        <Sharing
          file={paper}
          title={props.title || t("untitledDoc")}
          onShared={(made) => {
            setCard(made);
            setSharing(false);
          }}
          onClose={() => setSharing(false)}
          onError={props.onError}
        />
      )}
      {settings && card && (
        <ShareSettings
          file={paper}
          card={card}
          onChanged={(now) => {
            setCard(now);
            setSettings(false);
          }}
          onClose={() => setSettings(false)}
          onError={props.onError}
        />
      )}
    </aside>
  );
}
