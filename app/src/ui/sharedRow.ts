import { useCallback, useEffect, useMemo, useState } from "react";
import type { Panel } from "tisty-editor/read";
import { copied } from "../core";
import { fill, locale, t } from "../locales";
import {
  type ShareCard,
  type ShareStatus,
  shareAppeal,
  shareCard,
  sharePause,
  shareStatus,
  shareStop,
  shareSync,
} from "../sharing";

const DAY_MS = 86_400_000;
const MINUTE_MS = 60_000;

type HeldPanel = Extract<Panel, { kind: "held" }>;
type Act = NonNullable<HeldPanel["acts"]>[number];

const codeOf = (problem: unknown) => (problem as { code?: string } | null)?.code;
const untilOf = (problem: unknown) => Number((problem as { name?: string } | null)?.name);

const gistOf = (card: ShareCard, status: ShareStatus | null) => {
  if (status?.blocked) return { gist: t("sharedBlockedGist"), tone: "bad" as const };
  if (status?.paused) return { gist: t("sharedPausedGist"), tone: "warn" as const };
  return { gist: t(card.edit ? "sharedCanEdit" : "sharedReadOnly"), tone: undefined };
};

const saidOf = (status: ShareStatus | null, told: string | null, stopping: boolean) =>
  [
    status?.blocked && t("sharedBlockedWhy"),
    status?.paused && t("sharedPausedWhy"),
    status &&
      !status.blocked &&
      fill(
        "sharedStatus",
        new Date(status.diesOn * DAY_MS).toLocaleDateString(locale()),
        String(status.reads),
      ),
    told,
    stopping && t("shareStopAsk"),
  ].filter((one): one is string => Boolean(one));

const linksOf = (card: ShareCard, status: ShareStatus | null): HeldPanel["links"] => {
  if (status?.blocked) return [];
  const held = (key: string, label: string, url: string, note?: string) => ({
    key,
    label,
    url,
    note,
    copy: t("shareCopy"),
    onCopy: () => void copied(url).catch(() => {}),
  });
  const read = held("read", t("shareLinkRead"), card.read);
  if (!card.edit) return [read];
  return [read, held("edit", t("shareLinkEdit"), card.edit, t("shareLinkEditNote"))];
};

interface Hands {
  doc: string;
  card: ShareCard;
  status: ShareStatus | null;
  minutes: number;
  setStatus: (status: ShareStatus | null) => void;
  setTold: (told: string | null) => void;
  setStopping: (stopping: boolean) => void;
  setCard: (card: ShareCard | null) => void;
  failing: (problem: unknown) => void;
  onKeep?: () => Promise<unknown> | undefined;
}

const syncedWords = (synced: Awaited<ReturnType<typeof shareSync>>) => {
  if (synced.state === "unchanged") return t("shareSyncUnchanged");
  if (synced.state === "sent") return fill("shareSyncSent", String(synced.version));
  return fill("shareSyncOther", String(synced.version), synced.signer);
};

const syncAct = (h: Hands): Act => ({
  key: "sync",
  label: h.minutes ? fill("shareSyncWait", String(h.minutes)) : t("shareSync"),
  main: true,
  off: Boolean(h.status?.blocked || h.status?.paused || h.minutes > 0),
  onPick: () => {
    h.setTold(null);
    void Promise.resolve(h.onKeep?.())
      .then(() => shareSync(h.doc))
      .then((synced) => h.setTold(syncedWords(synced)))
      .catch(h.failing);
  },
});

const stoppingActs = (h: Hands): Act[] => [
  { key: "keep", label: t("shareCancel"), onPick: () => h.setStopping(false) },
  {
    key: "stop",
    label: t(h.card.taken ? "shareStopEditing" : "shareStop"),
    danger: true,
    onPick: () =>
      void shareStop(h.doc)
        .then(() => {
          h.setStopping(false);
          h.setCard(null);
          h.setStatus(null);
          h.setTold(null);
        })
        .catch(h.failing),
  },
];

const managingActs = (h: Hands): Act[] => {
  const see: Act = {
    key: "status",
    label: t("sharedSeeStatus"),
    onPick: () => void shareStatus(h.doc).then(h.setStatus).catch(h.failing),
  };
  if (h.status?.blocked)
    return [
      see,
      {
        key: "appeal",
        label: t("shareAppeal"),
        onPick: () =>
          void shareAppeal(h.doc)
            .then((accepted) =>
              h.setTold(t(accepted ? "shareAppealAccepted" : "shareAppealReview")),
            )
            .catch(h.failing),
      },
    ];
  const paused = h.status?.paused ?? false;
  return [
    see,
    {
      key: "pause",
      label: t(paused ? "shareResume" : "sharePause"),
      onPick: () =>
        void sharePause(h.doc, !paused)
          .then(() => shareStatus(h.doc))
          .then(h.setStatus)
          .catch(h.failing),
    },
    { key: "ask", label: t("shareStop"), danger: true, onPick: () => h.setStopping(true) },
  ];
};

const leavingActs = (h: Hands): Act[] => [
  { key: "leave", label: t("shareStopEditing"), danger: true, onPick: () => h.setStopping(true) },
];

export const useSharedRow = (
  doc: string | undefined,
  onError?: (problem: unknown) => void,
  onKeep?: () => Promise<unknown> | undefined,
) => {
  const [card, setCard] = useState<ShareCard | null>(null);
  const [asking, setAsking] = useState(false);
  const [told, setTold] = useState<string | null>(null);
  const [status, setStatus] = useState<ShareStatus | null>(null);
  const [stopping, setStopping] = useState(false);
  const [waitUntil, setWaitUntil] = useState(0);

  useEffect(() => {
    let gone = false;
    setCard(null);
    setTold(null);
    setStatus(null);
    setStopping(false);
    if (!doc) return;
    shareCard(doc)
      .then((now) => {
        if (!gone) setCard(now);
      })
      .catch(() => {});
    return () => {
      gone = true;
    };
  }, [doc]);

  useEffect(() => {
    if (!waitUntil) return;
    const ending = setTimeout(() => setWaitUntil(0), Math.max(0, waitUntil - Date.now()));
    return () => clearTimeout(ending);
  }, [waitUntil]);

  const failing = useCallback(
    (problem: unknown) => {
      const code = codeOf(problem);
      if (code === "folioTermsToAccept") {
        setAsking(true);
        return;
      }
      if (code === "folioWait" && untilOf(problem) > 0) {
        setWaitUntil(untilOf(problem));
        return;
      }
      onError?.(problem);
    },
    [onError],
  );

  const panel = useMemo((): Panel | null => {
    if (!doc) return null;
    if (!card)
      return {
        kind: "held",
        key: "shared",
        label: t("sharedLabel"),
        gist: t("sharedNot"),
        open: true,
        said: [t("sharedWhy")],
        acts: [
          {
            key: "share",
            label: t("shareIt"),
            main: true,
            onPick: () => void Promise.resolve(onKeep?.()).finally(() => setAsking(true)),
          },
        ],
      };
    const minutes = waitUntil ? Math.max(1, Math.ceil((waitUntil - Date.now()) / MINUTE_MS)) : 0;
    const hands: Hands = {
      doc,
      card,
      status,
      minutes,
      setStatus,
      setTold,
      setStopping,
      setCard,
      failing,
      onKeep,
    };
    const rest = card.taken ? leavingActs(hands) : managingActs(hands);
    return {
      kind: "held",
      key: "shared",
      label: t("sharedLabel"),
      ...gistOf(card, status),
      open: true,
      said: saidOf(status, told, stopping),
      links: linksOf(card, status),
      acts: stopping ? stoppingActs(hands) : [syncAct(hands), ...rest],
    };
  }, [doc, card, status, told, stopping, waitUntil, onKeep, failing]);

  return {
    panel,
    asking,
    onShared: (now: ShareCard) => setCard(now),
    onClosed: () => setAsking(false),
  };
};
