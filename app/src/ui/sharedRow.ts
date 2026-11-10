import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import type { Panel } from "tisty-editor/read";
import { copied } from "../core";
import { fill, locale, t } from "../locales";
import {
  type Named,
  type ShareCard,
  type ShareStatus,
  shareAppeal,
  shareCard,
  shareEditLinkRenewed,
  sharePause,
  shareSignAs,
  shareStatus,
  shareStop,
  shareSync,
  shareWaiting,
  toldPlainly,
} from "../sharing";

const DAY_MS = 86_400_000;
const MINUTE_MS = 60_000;
const WAITING_LOOK_MS = 60_000;

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

const blockedSaid = ({ mayCorrect, appealed }: ShareStatus) => {
  if (mayCorrect) return t("sharedMayCorrect");
  return t(appealed ? "sharedAppealWaiting" : "sharedBlockedWhy");
};

const saidOf = (
  status: ShareStatus | null,
  told: string | null,
  stopping: boolean,
  renewing: boolean,
) =>
  [
    status?.blocked && blockedSaid(status),
    status?.paused && t("sharedPausedWhy"),
    status && !status.blocked && fixSaid(status),
    status && !status.blocked && fill("sharedStatus", dayOf(status.diesOn), String(status.reads)),
    told,
    stopping && t("shareStopAsk"),
    renewing && t("shareRenewAsk"),
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
  pending: boolean;
  askWaiting: () => void;
  accepted: string | undefined;
  setAccepted: (accepted: string | undefined) => void;
  setStatus: (status: ShareStatus | null) => void;
  setTold: (told: string | null) => void;
  setStopping: (stopping: boolean) => void;
  setRenewing: (renewing: boolean) => void;
  setCard: (card: ShareCard | null) => void;
  failing: (problem: unknown) => void;
  stillHeld: () => boolean;
  onKeep?: () => Promise<unknown> | undefined;
}

type Told = Exclude<
  Awaited<ReturnType<typeof shareSync>>,
  { state: "secrets" | "othersEdited" | "gone" }
>;

const termsNote = (synced: { termsUnchecked: boolean }) =>
  synced.termsUnchecked ? [t("shareTermsUnchecked")] : [];

const syncedWords = (synced: Told) => {
  if (synced.state === "unchanged")
    return [t("shareSyncUnchanged"), ...synced.told.map(toldPlainly), ...termsNote(synced)].join(
      " ",
    );
  if (synced.state === "sent")
    return [
      synced.version === null
        ? t("shareSyncSentAlong")
        : fill("shareSyncSent", String(synced.version)),
      ...synced.told.map(toldPlainly),
      ...termsNote(synced),
    ].join(" ");
  const other = synced.signer
    ? fill("shareSyncOther", String(synced.version), synced.signer)
    : fill("shareSyncOtherPrint", String(synced.version), synced.fingerprint);
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
  if (synced.state === "othersEdited") {
    h.setAccepted(synced.print);
    h.setTold(fill("toldOthersEdited", synced.editors.join(", ")));
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

// The row outlives the document it shows, so an answer that arrives late may no longer be about it.
const statusNow = (h: Hands) =>
  shareStatus(h.doc).then((now) => {
    if (h.stillHeld()) h.setStatus(now);
  });

// Whoever holds the link to manage hears of reports when pressing, since Tisty never asks on its own.
const statusAfterSync = (h: Hands) => {
  if (!h.card.taken) void statusNow(h).catch(() => {});
};

const syncNow = (h: Hands) => {
  h.setTold(null);
  void Promise.resolve(h.onKeep?.())
    .then(() => shareSync(h.doc, h.accepted))
    .then((synced) => syncAnswered(h, synced))
    .catch(h.failing)
    .finally(() => {
      h.askWaiting();
      statusAfterSync(h);
    });
};

const syncAct = (h: Hands): Act => ({
  key: "sync",
  label: syncLabel(h),
  main: true,
  off: Boolean((h.status?.blocked && !h.status.mayCorrect) || h.status?.paused || h.minutes > 0),
  onPick: () => syncNow(h),
});

// Folio already took the new permit even when the version that follows has to wait, so the row shows the new link either way.
const renewNow = (h: Hands) => {
  h.setRenewing(false);
  h.setTold(null);
  void Promise.resolve(h.onKeep?.())
    .then(() => shareEditLinkRenewed(h.doc))
    .then((synced) => {
      if (synced.state === "sent") h.setTold(t("shareRenewed"));
      else if (synced.state === "otherVersion" || synced.state === "unchanged")
        h.setTold(`${syncedWords(synced)} ${t("shareRenewWaits")}`);
      else syncAnswered(h, synced);
    })
    .catch((problem) => {
      h.failing(problem);
      if (codeOf(problem) === "folioWait" && untilOf(problem) > 0) h.setTold(t("shareRenewHeld"));
    })
    .finally(() => {
      void shareCard(h.doc)
        .then((card) => card && h.setCard(card))
        .catch(() => {});
      h.askWaiting();
    });
};

const renewingActs = (h: Hands): Act[] => [
  { key: "keep", label: t("shareCancel"), onPick: () => h.setRenewing(false) },
  { key: "renew", label: t("shareRenew"), danger: true, onPick: () => renewNow(h) },
];

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
    onPick: () => void statusNow(h).catch(h.failing),
  };
  if (h.status?.blocked && (h.status.mayCorrect || h.status.appealed)) return [see];
  if (h.status?.blocked)
    return [
      see,
      {
        key: "appeal",
        label: t("shareAppeal"),
        onPick: () =>
          void shareAppeal(h.doc)
            .then((accepted) => {
              h.setTold(t(accepted ? "shareAppealAccepted" : "shareAppealReview"));
              return statusNow(h);
            })
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
          .then(() => statusNow(h))
          .catch(h.failing),
    },
    ...(h.card.edit
      ? [{ key: "renewAsk", label: t("shareRenewLink"), onPick: () => h.setRenewing(true) }]
      : []),
    { key: "ask", label: t("shareStop"), danger: true, onPick: () => h.setStopping(true) },
  ];
};

const shown = ({ name, checked }: Named) => (checked ? fill("sharedChecked", name) : name);

const whoSaid = ({ owner, signer, editors, signsAs }: ShareCard) => [
  ...(owner && signer ? [fill("sharedOwnerAndSigner", shown(owner), shown(signer))] : []),
  ...(owner && !signer ? [fill("sharedOwner", shown(owner))] : []),
  ...(!owner && signer ? [fill("sharedVersionBy", shown(signer))] : []),
  ...(editors.length > 0 ? [fill("sharedEditedBy", editors.map(shown).join(", "))] : []),
  ...(signsAs?.now ? [fill("shareSignsAs", signsAs.now)] : []),
];

// One root signs as the owner, the last edit and an editor alike, so it is listed once, by its first role.
const printsOf = ({ owner, signer, editors }: ShareCard) => {
  const rows = new Map<string, string>();
  const add = (label: string, one: Named) => {
    if (one.print && !rows.has(one.print)) rows.set(one.print, label);
  };
  if (owner) add(fill("sharedPrintOwner", shown(owner)), owner);
  if (signer) add(fill("sharedPrintSigner", shown(signer)), signer);
  for (const one of editors) add(shown(one), one);
  if (!rows.size) return undefined;
  return {
    label: t("sharedPrints"),
    note: t("sharedPrintsWhy"),
    rows: [...rows].map(([print, label]) => [label, print] as const),
  };
};

const leavingActs = (h: Hands): Act[] => {
  const other = h.card.signsAs?.other;
  const signing: Act[] = other
    ? [
        {
          key: "signs",
          label: fill("shareSignAsOther", other),
          onPick: () => void shareSignAs(h.doc, other).then(h.setCard).catch(h.failing),
        },
      ]
    : [];
  return [
    ...signing,
    { key: "leave", label: t("shareStopEditing"), danger: true, onPick: () => h.setStopping(true) },
  ];
};

export const useSharedRow = (
  doc: string | undefined,
  onError?: (problem: unknown) => void,
  onKeep?: () => Promise<unknown> | undefined,
) => {
  const [card, setCard] = useState<ShareCard | null>(null);
  const [asking, setAsking] = useState(false);
  const [termsOnly, setTermsOnly] = useState(false);
  const cardNow = useRef(card);
  cardNow.current = card;
  const docNow = useRef(doc);
  docNow.current = doc;
  const [accepted, setAccepted] = useState<string | undefined>(undefined);
  const [told, setTold] = useState<string | null>(null);
  const [status, setStatus] = useState<ShareStatus | null>(null);
  const [stopping, setStopping] = useState(false);
  const [renewing, setRenewing] = useState(false);
  const [waitUntil, setWaitUntil] = useState(0);
  const [pending, setPending] = useState(false);
  const pendingNow = useRef(pending);
  pendingNow.current = pending;
  const [now, setNow] = useState(() => Date.now());

  // Folio's own wait can outlast this computer's, so the later of the two holds.
  const waitTill = useCallback((until: number) => {
    const at = Date.now();
    setNow(at);
    setWaitUntil((was) => Math.max(was > at ? was : 0, until));
  }, []);

  const askWaiting = useCallback(
    (look: boolean) => {
      if (!doc) return;
      void shareWaiting(doc, look)
        .then((waiting) => {
          if (docNow.current !== doc) return;
          waitTill(waiting.until ?? 0);
          if (look) setPending(waiting.pending);
        })
        .catch(() => {});
    },
    [doc, waitTill],
  );

  useEffect(() => {
    let gone = false;
    setCard(null);
    setTold(null);
    setStatus(null);
    setStopping(false);
    setRenewing(false);
    setAccepted(undefined);
    setWaitUntil(0);
    setPending(false);
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
    if (card && !card.held) askWaiting(true);
  }, [card, askWaiting]);

  useEffect(() => {
    if (!waitUntil) return;
    const ending = setTimeout(
      () => {
        setWaitUntil(0);
        setPending(false);
      },
      Math.max(0, waitUntil - Date.now()),
    );
    const looking = setInterval(() => {
      setNow(Date.now());
      askWaiting(!pendingNow.current);
    }, WAITING_LOOK_MS);
    return () => {
      clearTimeout(ending);
      clearInterval(looking);
    };
  }, [waitUntil, askWaiting]);

  const failing = useCallback(
    (problem: unknown) => {
      const code = codeOf(problem);
      if (code === "folioTermsToAccept") {
        setTermsOnly(true);
        setAsking(true);
        setTold(t("shareSyncAfterTerms"));
        return;
      }
      if (code === "folioTermsUpdating") {
        setTold(t("folioTermsUpdating"));
        return;
      }
      if (code === "folioGone") {
        setCard(null);
        setStatus(null);
        setTold(t("folioGone"));
        return;
      }
      if (code === "folioWait" && untilOf(problem) > 0) {
        waitTill(untilOf(problem));
        return;
      }
      onError?.(problem);
    },
    [onError, waitTill],
  );

  const hands = useMemo((): Hands | null => {
    if (!doc || !card) return null;
    const minutes = waitUntil ? Math.max(1, Math.ceil((waitUntil - now) / MINUTE_MS)) : 0;
    return {
      doc,
      card,
      status,
      minutes,
      pending,
      askWaiting: () => askWaiting(true),
      accepted,
      setAccepted,
      setStatus,
      setTold,
      setStopping,
      setRenewing,
      setCard,
      failing,
      stillHeld: () => docNow.current === doc && cardNow.current !== null,
      onKeep,
    };
  }, [doc, card, status, waitUntil, now, pending, askWaiting, accepted, onKeep, failing]);

  const panel = useMemo((): Panel | null => {
    if (!doc) return null;
    if (!hands)
      return {
        kind: "held",
        key: "shared",
        label: t("sharedLabel"),
        mark: t("sharedBeta"),
        gist: t("sharedNot"),
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
    if (hands.card.held)
      return {
        kind: "held",
        key: "shared",
        label: t("sharedLabel"),
        mark: t("sharedBeta"),
        gist: t("sharedHeldGist"),
        said: [t("sharedHeldWhy")],
        links: linksOf(hands.card, hands.status),
      };
    const rest = hands.card.taken ? leavingActs(hands) : managingActs(hands);
    return {
      kind: "held",
      key: "shared",
      label: t("sharedLabel"),
      mark: t("sharedBeta"),
      ...gistOf(hands.card, hands.status),
      said: [
        ...whoSaid(hands.card),
        ...saidOf(hands.status, told, stopping, renewing),
        ...(hands.minutes > 0 && hands.pending ? [t("shareSyncPending")] : []),
      ],
      prints: printsOf(hands.card),
      links: linksOf(hands.card, hands.status),
      acts: stopping
        ? stoppingActs(hands)
        : renewing
          ? renewingActs(hands)
          : [syncAct(hands), ...rest],
    };
  }, [doc, told, stopping, renewing, hands, onKeep]);

  return {
    panel,
    asking,
    termsOnly,
    onShared: (now: ShareCard) => setCard(now),
    onTermsAccepted: () => {
      setAsking(false);
      setTermsOnly(false);
      if (hands) syncNow(hands);
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
