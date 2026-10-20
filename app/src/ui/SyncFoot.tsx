import { useCallback, useEffect, useState } from "react";
import { useComing, useStuck } from "../coming";
import { type Carrying, type CloudStatus, cloudPause, cloudStatus, syncState } from "../core";
import { decideAll, leftForLater, useLeftForLater } from "../deciding";
import { type Footing, footing, offered } from "../footing";
import { budgetResets, stamped } from "../format";
import { useKnocking } from "../knocking";
import { cloudNamed, fill, t } from "../locales";
import { saidPlainly } from "../refusal";
import { offlineNow } from "../syncSaid";

const A_WHILE = 60_000;

interface Props {
  carried?: number;
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

const folderName = (at: string): string => at.split(/[\\/]/).filter(Boolean).pop() ?? at;

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

export default function SyncFoot({ carried, onSync, onOpen, onTrouble, fallback }: Props) {
  const [state, setState] = useState<Carrying | null>(null);
  const [status, setStatus] = useState<CloudStatus | null>(null);
  const [mine, setMine] = useState(false);
  const [offline, setOffline] = useState(offlineNow);
  const coming = useComing();
  const stuck = useStuck();
  const knocked = useKnocking().waiting.length > 0;
  const putOff = useLeftForLater();
  const going = coming !== null;

  const look = useCallback(() => {
    syncState()
      .then((now) => {
        setState(now ?? null);
        return now?.mode === "cloud" ? cloudStatus() : null;
      })
      .then((now) => setStatus(now ?? null))
      .catch(() => undefined);
  }, []);

  useEffect(() => {
    if (!going) look();
  }, [look, going, stuck, carried]);

  useEffect(() => {
    const online = () => setOffline(offlineNow());
    const ticking = setInterval(look, A_WHILE);
    window.addEventListener("focus", look);
    window.addEventListener("online", online);
    window.addEventListener("offline", online);
    return () => {
      clearInterval(ticking);
      window.removeEventListener("focus", look);
      window.removeEventListener("online", online);
      window.removeEventListener("offline", online);
    };
  }, [look]);

  const foot = footing({ state, status, coming, mine, offline, knocked, putOff, stuck });
  if (!foot || !state) return fallback ?? null;

  const named =
    state.mode === "cloud"
      ? cloudNamed(status?.provider ?? state.cloudProvider)
      : folderName(state.chosen ?? "");
  const { line, why } = told(foot, state, named);

  const failed = (problem: unknown) => onTrouble?.(saidPlainly(problem));

  const syncNow = () => {
    setMine(true);
    void Promise.resolve(onSync())
      .catch(failed)
      .finally(() => {
        setMine(false);
        look();
      });
  };

  const press = () => {
    if (foot.kind === "ok") return syncNow();
    if (foot.kind === "paused")
      return void cloudPause(false)
        .then(syncNow)
        .catch((problem) => {
          failed(problem);
          look();
        });
    if (foot.kind === "look" && foot.putOff)
      return void decideAll(leftForLater(), true)
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
        disabled={!offered(foot)}
        aria-label={[named, line, foot.kind === "ok" ? undefined : why].filter(Boolean).join(" · ")}
        onClick={press}
        className="flex min-w-0 flex-1 items-center gap-2 rounded-md px-2 py-1 text-left text-[11.5px] text-faint hover:bg-hover hover:text-ink disabled:hover:bg-transparent disabled:hover:text-faint"
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
