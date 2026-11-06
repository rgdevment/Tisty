import { useCallback, useEffect, useMemo, useRef, useState } from "react";
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
  toldPlainly,
} from "../sharing";

const DAY_MS = 86_400_000;
const MINUTE_MS = 60_000;

type HeldPanel = Extract<Panel, { kind: "held" }>;
type Act = NonNullable<HeldPanel["acts"]>[number];

const codeOf = (problem: unknown) => (problem as { code?: string } | null)?.code;
const untilOf = (problem: unknown) => Number((problem as { name?: string } | null)?.name);

// Folio counts days from 1970 in UTC, and elsewhere its midnight falls on the day before.
const dayOf = (day: number) =>
  new Date(day * DAY_MS).toLocaleDateString(locale(), { timeZone: "UTC" });

const fixSaid = ({ fixBy }: ShareStatus) => {
  if (fixBy === null) return null;
  return Date.now() >= (fixBy + 1) * DAY_MS
    ? t("sharedFixLate")
    : fill("sharedFixBy", dayOf(fixBy));
};

const gistOf = (card: ShareCard, status: ShareStatus | null) => {
  if (status?.blocked) return { gist: t("sharedBlockedGist"), tone: "bad" as const };
  if (status?.paused) return { gist: t("sharedPausedGist"), tone: "warn" as const };
  if (status && fixSaid(status)) return { gist: t("sharedReportedGist"), tone: "warn" as const };
  return { gist: t(card.edit ? "sharedCanEdit" : "sharedReadOnly"), tone: undefined };
};

const saidOf = (status: ShareStatus | null, told: string | null, stopping: boolean) =>
  [
    status?.blocked && t(status.mayCorrect ? "sharedMayCorrect" : "sharedBlockedWhy"),
    status?.paused && t("sharedPausedWhy"),
    status && !status.blocked && fixSaid(status),
    status && !status.blocked && fill("sharedStatus", dayOf(status.diesOn), String(status.reads)),
    told,
    stopping && t("shareStopAsk"),
  ].filter((one): one is string => Boolean(one));

const linksOf = (card: ShareCard, status: ShareStatus | null): HeldPanel["links"] => {
  if (status?.blocked || card.taken) return [];
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
  accepted: string | undefined;
  setAccepted: (accepted: string | undefined) => void;
  setStatus: (status: ShareStatus | null) => void;
  setTold: (told: string | null) => void;
  setStopping: (stopping: boolean) => void;
  setCard: (card: ShareCard | null) => void;
  failing: (problem: unknown) => void;
  stillHeld: () => boolean;
  onKeep?: () => Promise<unknown> | undefined;
}

type Told = Exclude<Awaited<ReturnType<typeof shareSync>>, { state: "secrets" | "gone" }>;

const syncedWords = (synced: Told) => {
  if (synced.state === "unchanged") return t("shareSyncUnchanged");
  if (synced.state === "sent")
    return [fill("shareSyncSent", String(synced.version)), ...synced.told.map(toldPlainly)].join(
      " ",
    );
  const other = fill("shareSyncOther", String(synced.version), synced.signer);
  return synced.doubtful ? `${other} ${t("shareSyncOtherDoubt")}` : other;
};

const syncLabel = ({ minutes, accepted }: Hands) => {
  if (minutes) return fill("shareSyncWait", String(minutes));
  return t(accepted ? "shareGoAnyway" : "shareSync");
};

const syncAnswered = (h: Hands, synced: Awaited<ReturnType<typeof shareSync>>) => {
  if (synced.state === "secrets") {
    h.setAccepted(synced.print);
    h.setTold(fill("shareSecretsSync", synced.place));
    return;
  }
  h.setAccepted(undefined);
  if (synced.state === "gone") {
    h.setCard(null);
    h.setStatus(null);
    h.setTold(t("folioGone"));
    return;
  }
  h.setTold(syncedWords(synced));
};

// Whoever holds the link to manage hears of reports when pressing, since Tisty never asks on its own.
const statusAfterSync = (h: Hands) => {
  if (!h.card.taken)
    void shareStatus(h.doc)
      .then((now) => {
        if (h.stillHeld()) h.setStatus(now);
      })
      .catch(() => {});
};

const syncNow = (h: Hands) => {
  h.setTold(null);
  void Promise.resolve(h.onKeep?.())
    .then(() => shareSync(h.doc, h.accepted))
    .then((synced) => syncAnswered(h, synced))
    .catch(h.failing)
    .finally(() => statusAfterSync(h));
};

const syncAct = (h: Hands): Act => ({
  key: "sync",
  label: syncLabel(h),
  main: true,
  off: Boolean((h.status?.blocked && !h.status.mayCorrect) || h.status?.paused || h.minutes > 0),
  onPick: () => syncNow(h),
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
  if (h.status?.blocked && h.status.mayCorrect) return [see];
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
  const [termsOnly, setTermsOnly] = useState(false);
  const syncing = useRef<(() => void) | null>(null);
  const cardNow = useRef(card);
  cardNow.current = card;
  const [accepted, setAccepted] = useState<string | undefined>(undefined);
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
    setAccepted(undefined);
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
        setTermsOnly(true);
        setAsking(true);
        setTold(t("shareSyncAfterTerms"));
        return;
      }
      if (code === "folioGone") {
        setCard(null);
        setStatus(null);
        setTold(t("folioGone"));
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
        said: told ? [told, t("sharedWhy")] : [t("sharedWhy")],
        acts: [
          {
            key: "share",
            label: t("shareIt"),
            main: true,
            onPick: () =>
              void Promise.resolve(onKeep?.()).finally(() => {
                setTermsOnly(false);
                setAsking(true);
              }),
          },
        ],
      };
    if (card.held)
      return {
        kind: "held",
        key: "shared",
        label: t("sharedLabel"),
        gist: t("sharedHeldGist"),
        open: true,
        said: [t("sharedHeldWhy")],
        links: linksOf(card, status),
      };
    const minutes = waitUntil ? Math.max(1, Math.ceil((waitUntil - Date.now()) / MINUTE_MS)) : 0;
    const hands: Hands = {
      doc,
      card,
      status,
      minutes,
      accepted,
      setAccepted,
      setStatus,
      setTold,
      setStopping,
      setCard,
      failing,
      stillHeld: () => cardNow.current !== null,
      onKeep,
    };
    syncing.current = () => syncNow(hands);
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
  }, [doc, card, status, told, stopping, waitUntil, accepted, onKeep, failing]);

  return {
    panel,
    asking,
    termsOnly,
    onShared: (now: ShareCard) => setCard(now),
    onTermsAccepted: () => {
      setAsking(false);
      setTermsOnly(false);
      syncing.current?.();
    },
    onClosed: () => {
      setAsking(false);
      if (doc)
        void shareCard(doc)
          .then(setCard)
          .catch(() => {});
    },
  };
};
