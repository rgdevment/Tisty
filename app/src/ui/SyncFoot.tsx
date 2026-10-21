import { useEffect, useState } from "react";
import { useComing, useStuck } from "../coming";
import { type Carrying, cloudPause } from "../core";
import { decideAll, leftForLater, settledElsewhere, useLeftForLater } from "../deciding";
import { type Footing, footing, offered } from "../footing";
import { budgetResets, stamped } from "../format";
import { useKnocking } from "../knocking";
import { cloudNamed, fill, t } from "../locales";
import { saidPlainly } from "../refusal";
import { offlineNow } from "../syncSaid";
import { lookAgain, useSyncSeen } from "../syncSeen";
import { leaf } from "./Mode";

const A_WHILE = 60_000;
const SOON = 10_000;
// A round asked for by hand can end before the eye sees it start, so its end stays a moment.
const DONE_SHOWN = 2_000;

interface Props {
  onSync: () => Promise<unknown> | undefined;
  onOpen: () => void;
  onTrouble?: (text: string) => void;
  fallback?: React.ReactNode;
}

const DOT: Record<Footing["kind"], string> = {
  ok: "bg-hue-green",
  syncing: "bg-accent",
  wait: "bg-hue-amber",
  halted: "bg-hue-amber",
  spent: "bg-hue-amber",
  offline: "bg-faint",
  lost: "bg-urgent",
  paused: "bg-faint",
  look: "bg-hue-blue",
};

const until = (secs: number): string => stamped(new Date(secs * 1000).toISOString());

function told(foot: Footing, state: Carrying, cloud: string): { line: string; why?: string } {
  switch (foot.kind) {
    case "syncing":
      return {
        line:
          foot.whole > 0
            ? fill("footSyncingOf", String(foot.done), String(foot.whole))
            : t("syncing_"),
      };
    case "wait":
      return foot.troubled
        ? { line: fill("footTroubled", cloud), why: t("footTroubledWhy") }
        : { line: fill("footWait", cloud), why: fill("footWaitWhy", cloud) };
    case "halted":
      return { line: fill("footHalted", until(foot.until)), why: t("footHaltedWhy") };
    case "spent":
      return { line: t("footSpent"), why: fill("stateBudgetWhy", budgetResets()) };
    case "offline":
      return { line: t("footOffline"), why: t("footOfflineWhy") };
    case "lost":
      return { line: t("footSignIn"), why: t("stateReconnectWhy") };
    case "paused":
      return { line: t("footPaused"), why: t("footPausedWhy") };
    case "look":
      return { line: t("footLook"), why: t(foot.putOff ? "footLookDocs" : "footLookWhy") };
    default:
      return {
        line: fill("footUpToDate", state.last ? stamped(state.last) : t("syncNever")),
        why: t("syncNow"),
      };
  }
}

export default function SyncFoot({ onSync, onOpen, onTrouble, fallback }: Props) {
  const { state, status } = useSyncSeen();
  const [mine, setMine] = useState(false);
  const [finished, setFinished] = useState(false);
  const [offline, setOffline] = useState(offlineNow);
  const [, tick] = useState(0);
  const coming = useComing();
  const stuck = useStuck();
  const knocked = useKnocking().waiting.length > 0;
  const putOff = useLeftForLater();
  const idle = coming === null;
  const foot = footing({ state, status, coming, mine, offline, knocked, putOff, stuck });
  // Google and Microsoft can ask to wait, and let go again, in the middle of one long round.
  const often = !idle || foot?.kind === "wait";
  // A round that ends, or a refusal that comes or goes, may have written what the foot shows.
  useEffect(() => {
    if (idle) void lookAgain();
  }, [idle, stuck]);

  useEffect(() => {
    const ticking = setInterval(
      () => {
        tick((was) => was + 1);
        if (often) void lookAgain();
      },
      often ? SOON : A_WHILE,
    );
    return () => clearInterval(ticking);
  }, [often]);

  useEffect(() => {
    if (!finished) return;
    const gone = setTimeout(() => setFinished(false), DONE_SHOWN);
    return () => clearTimeout(gone);
  }, [finished]);

  // A look on focus catches what was written beside the rounds.
  useEffect(() => {
    const look = () => void lookAgain();
    const online = () => setOffline(offlineNow());
    window.addEventListener("focus", look);
    window.addEventListener("online", online);
    window.addEventListener("offline", online);
    return () => {
      window.removeEventListener("focus", look);
      window.removeEventListener("online", online);
      window.removeEventListener("offline", online);
    };
  }, []);

  if (!foot || !state) return fallback ?? null;

  const named =
    state.mode === "cloud"
      ? cloudNamed(status?.provider ?? state.cloudProvider)
      : leaf(state.chosen ?? "");
  const { line, why } =
    finished && foot.kind === "ok"
      ? { line: t("footDone"), why: t("syncNow") }
      : told(foot, state, named);

  const failed = (problem: unknown) => onTrouble?.(saidPlainly(problem));

  const syncNow = () => {
    setMine(true);
    void Promise.resolve(onSync())
      .then(() => setFinished(true))
      .catch(failed)
      .finally(() => setMine(false));
  };

  const press = () => {
    // Just done, another press would only start the same round again.
    if (finished) return;
    if (!offered(foot)) return onOpen();
    if (foot.kind === "ok") return syncNow();
    if (foot.kind === "paused")
      return void cloudPause(false)
        .then(() => {
          void lookAgain();
          syncNow();
        })
        .catch((problem) => {
          failed(problem);
          void lookAgain();
        });
    if (foot.kind === "look" && foot.putOff)
      return void settledElsewhere()
        .then(() => decideAll(leftForLater(), true))
        .then(({ said }) => {
          if (said) onTrouble?.(said);
          syncNow();
        })
        .catch(failed);
    onOpen();
  };

  return (
    <span title={why ?? `${named} · ${line}`} className="mx-1.5 flex min-w-0 flex-1">
      <button
        type="button"
        aria-label={[named, line, foot.kind === "ok" ? undefined : why].filter(Boolean).join(" · ")}
        onClick={press}
        className="flex min-w-0 flex-1 items-center gap-2 rounded-md px-2 py-1 text-left text-[11.5px] text-faint hover:bg-hover hover:text-ink"
      >
        <span
          aria-hidden="true"
          className={`size-1.5 shrink-0 rounded-full ${DOT[foot.kind]} ${
            foot.kind === "syncing" ? "motion-safe:animate-pulse" : ""
          }`}
        />
        <span className="min-w-0 truncate">
          {named} · {line}
        </span>
      </button>
    </span>
  );
}
