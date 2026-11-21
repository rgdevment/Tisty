import { openUrl } from "@tauri-apps/plugin-opener";
import { type ReactNode, useCallback, useEffect, useState } from "react";
import { cloudStanding, type Exit, storageOf } from "../cloudState";
import { typed } from "../clouds";
import { carding, unstuck, useAwaitsChoice } from "../coming";
import {
  type Carrying,
  type CloudStatus,
  type Connected,
  chooseSync,
  cloudAbandon,
  cloudChoose,
  cloudConnect,
  cloudConnectCancel,
  cloudForget,
  cloudPause,
  cloudRevoke,
  cloudStatus,
  cloudStranded,
} from "../core";
import { stamped } from "../format";
import { fill, t } from "../locales";
import { named } from "../previews";
import { plainly, saidPlainly } from "../refusal";
import { sure } from "../sure";
import { syncingSaid } from "../syncSaid";
import type { Run, Word } from "./Card";
import { useCareful } from "./Careful";
import CloudMark from "./CloudMark";
import DavForm from "./DavForm";
import { DANGERS, type Danger } from "./dangers";
import { Feedback, Heeded, useRows } from "./RowNotes";
import { Fold, mild, strong } from "./Rows";
import { useSince } from "./useSince";
import { held as isHeld, type Working } from "./working";
import { ZoneButtons } from "./Zone";

interface Props {
  state: Carrying;
  busy: Working;
  run: Run;
  fail: (word?: Word) => void;
  carry: (way?: "again") => void;
  onMove: () => void;
  lead?: ReactNode;
  children?: ReactNode;
}

const SIGNED = new Set(["cloudLost", "cloudElsewhere"]);

const LOOK_AGAIN = 10_000;

export default function Clouded({ state, busy, run, fail, carry, onMove, lead, children }: Props) {
  const [status, setStatus] = useState<CloudStatus | null>(null);
  const [which, setWhich] = useState(false);
  const [signedAt, setSignedAt] = useState<number>();
  const [askingPassword, setAskingPassword] = useState(false);
  const [unread, setUnread] = useState<string>();
  const [signing, setSigning] = useState(false);
  const { said, trouble } = useRows();
  const held = isHeld(busy, "sync");
  const syncing = busy.has("sync");
  const waiting = useAwaitsChoice();
  const running = useSince(syncing, busy.get("sync"));
  const careful = useCareful();

  const look = useCallback(() => {
    setUnread(undefined);
    cloudStatus()
      .then(setStatus)
      .catch((e) => setUnread(saidPlainly(e)));
  }, []);

  useEffect(look, [look, state.last, held]);

  const unsettled = Boolean(
    status?.running || status?.sealingElsewhere || (status && !status.joined),
  );
  useEffect(() => {
    if (!unsettled) return;
    const ticking = setInterval(look, LOOK_AGAIN);
    return () => clearInterval(ticking);
  }, [unsettled, look]);

  const last = status?.last;
  // A sign-in that went through answers the round that found the door shut, before the next round ends.
  const answered =
    last !== undefined && last !== null && last.at === signedAt && SIGNED.has(last.code);
  const standing = status && !answered ? cloudStanding(status) : null;
  const awaits = waiting || standing?.exit === "choose";
  const carded = standing?.exit && last ? last.code : null;
  const wrong = trouble.sync?.text;
  const repeats =
    (carded !== null && wrong === plainly({ code: carded, name: last?.name ?? undefined })) ||
    (standing?.urgent === true && wrong === standing.told);

  useEffect(() => {
    carding(carded);
    return () => carding(null);
  }, [carded]);
  const paused = status?.pausedHere ?? false;
  const left = status?.last?.left ?? 0;

  const switched = (on: boolean) => run("sync", cloudPause(!on), look);

  const provider = status?.provider ?? state.cloudProvider ?? "google";

  // A copy this machine already carried stays with its account; one never carried may change it.
  const reconnected = async (account: Connected) => {
    const was = status?.account;
    if (was && was !== account.account && status?.joined) {
      await cloudForget(account.account, account.provider).catch(() => undefined);
      throw { code: "cloudElsewhere", name: account.email };
    }
    await cloudChoose(account.account, account.email, account.provider);
    if (was && was !== account.account)
      await cloudForget(was, status?.provider).catch(() => undefined);
    setSignedAt(status?.last?.at);
    unstuck();
  };

  const reconnect = () => {
    if (typed(provider)) return setAskingPassword(true);
    setSigning(true);
    run(
      "sync",
      cloudConnect(provider)
        .then(reconnected)
        .then(
          () => true,
          (e) => {
            if (e?.code === "cloudDropped") return false;
            throw e;
          },
        )
        .finally(() => setSigning(false)),
      (went) => went && carry(),
    );
  };

  const passwordGiven = (account: Connected) => {
    setAskingPassword(false);
    run("sync", reconnected(account), () => carry());
  };

  const stop = () => {
    const was = status?.account;
    run(
      "sync",
      chooseSync().then(() => (was ? cloudForget(was, status?.provider) : undefined)),
      () => {},
    );
  };

  const exit = async (how: Exit) => {
    if (how === "again") {
      if (await sure(t("stateEmptiedSure"))) carry("again");
      return;
    }
    if (how === "reconnect") return reconnect();
    if (how === "turnOn") return switched(true);
    if (how === "choose") return carry();
    const storage = storageOf(status?.provider ?? state.cloudProvider);
    if (!storage) return;
    openUrl(storage).catch((e) => fail({ card: "sync", text: saidPlainly(e) }));
  };

  const forget = async () => {
    if (held || !(await careful.ask(DANGERS.forget()))) return;
    run("cloudForget", () => cloudForget(), look);
  };

  const lost = Boolean(status?.lost);
  const revokes = !typed(provider);

  return (
    <>
      {careful.shown}
      <section className="rounded-[10px] border border-hair px-4 py-3.5">
        {lead}
        <div className="flex items-center gap-2.5 rounded-[10px] border border-hair px-3 py-2">
          <CloudMark provider={provider} />
          <span className="min-w-0 flex-1">
            <span className="block text-[13px] font-medium">{t("cloudChosen")}</span>
            <span className="block truncate text-[11.5px] text-faint">
              {status?.email ?? state.cloud}
            </span>
          </span>
          {!status && !unread && (
            <span
              role="status"
              className="flex-none text-[10.5px] text-faint motion-safe:animate-pulse"
            >
              {t("checkingNow")}
            </span>
          )}
          {!status && unread && (
            <button
              type="button"
              onClick={look}
              className="flex-none text-[10.5px] text-accent hover:underline"
            >
              {t("tryAgain")}
            </button>
          )}
          {status && !standing && (
            <span className="flex-none rounded-md bg-hue-green/10 px-1.5 py-0.5 text-[10.5px] text-hue-green">
              {t("cloudConnectedTag")}
            </span>
          )}
        </div>

        {!status && unread && (
          <p role="alert" className="mt-2 text-[11.5px] text-urgent">
            {unread}
          </p>
        )}
        <p className="mt-2 text-[11.5px] leading-relaxed text-faint">{t("cloudWhere")}</p>

        {standing && (
          <div
            role="status"
            className="mt-2.5 flex items-start gap-2.5 rounded-[10px] border border-hue-amber/40 px-3 py-2"
          >
            <span className="min-w-0 flex-1 text-[11.5px] leading-relaxed text-soft">
              <span
                className={`block text-[12.5px] font-semibold text-ink ${
                  standing.pending ? "motion-safe:animate-pulse" : ""
                }`}
              >
                {standing.name ? fill(standing.said, standing.name) : t(standing.said)}
              </span>
              {standing.why ? (
                t(standing.why)
              ) : standing.urgent ? (
                <span className="text-urgent">{standing.told}</span>
              ) : (
                standing.told
              )}
            </span>
            {standing.exit && standing.exitSaid && (
              <button
                type="button"
                disabled={held}
                onClick={() => standing.exit && void exit(standing.exit)}
                className={strong}
              >
                {t(standing.exitSaid)}
              </button>
            )}
          </div>
        )}

        {askingPassword && status && standing?.exit === "reconnect" && (
          <div className="mt-2.5 rounded-[10px] border border-hair px-3 py-2">
            <DavForm
              account={status.account}
              busy={held}
              onConnected={passwordGiven}
              onTrouble={(text) => text && fail({ card: "sync", text })}
            />
          </div>
        )}

        <div className="mt-2.5 flex flex-wrap items-center gap-2.5">
          <button
            type="button"
            disabled={held || paused}
            onClick={() => carry()}
            className={strong}
          >
            {syncing && !signing ? syncingSaid(running) : t("syncNow")}
          </button>
          {standing?.exit !== "turnOn" && (
            <button type="button" disabled={held} onClick={() => switched(paused)} className={mild}>
              {paused ? t("cloudTurnOn") : t("cloudTurnOff")}
            </button>
          )}
          <span className="ml-auto text-[11.5px] text-faint">
            {awaits
              ? t("syncChoosing")
              : fill("syncLast", state.last ? stamped(state.last) : t("syncNever"))}
          </span>
        </div>

        {signing && (
          <p
            role="status"
            aria-live="polite"
            className="mt-2 text-[11.5px] text-faint motion-safe:animate-pulse"
          >
            {t("cloudWaiting")}{" "}
            <button
              type="button"
              onClick={() => void cloudConnectCancel().catch(() => undefined)}
              className="text-accent hover:underline"
            >
              {t("cancel")}
            </button>
          </p>
        )}

        {left > 0 && (
          <p className="mt-2 text-[12.5px] text-soft">
            {t("cloudForLater")}: {fill("cloudForLaterCount", String(left))}
            {(status?.last?.names.length ?? 0) > 0 && (
              <>
                {" · "}
                <button
                  type="button"
                  aria-expanded={which}
                  onClick={() => setWhich(!which)}
                  className="text-accent hover:underline"
                >
                  {t("cloudSeeWhich")}
                </button>
              </>
            )}
          </p>
        )}
        {which && left > 0 && (
          <ul className="mt-1 list-disc pl-5 text-[11.5px] text-faint">
            {status?.last?.names.map((one) => (
              <li key={one}>{named(one)}</li>
            ))}
          </ul>
        )}

        <div className="mt-3 border-t border-hair pt-3">
          <div className="grid grid-cols-2 gap-2">
            <p className="col-span-2 text-[11.5px] text-faint">{t("syncSetUp")}</p>
            {status?.joined && (
              <button type="button" disabled={held} onClick={onMove} className={mild}>
                {t("moveOpen")}
              </button>
            )}
            {status && !status.joined && (
              <button type="button" disabled={held} onClick={stop} className={mild}>
                {t("cloudStop")}
              </button>
            )}
          </div>
          <Fold>
            <button
              type="button"
              disabled={held || paused}
              title={t("syncAgainWhy")}
              onClick={() => void exit("again")}
              className={mild}
            >
              {t("syncAgain")}
            </button>
          </Fold>
        </div>
        {!standing && <Heeded row="syncing" />}
        <Feedback
          which="sync"
          quiet
          hideWorking
          hideTrouble={repeats}
          hideSaid={awaits && said.sync?.busy === true}
        />
      </section>
      {children}
      <CloudExits
        lost={lost}
        revokes={revokes}
        held={held}
        run={run}
        fail={fail}
        look={look}
        onForget={forget}
        ask={careful.ask}
      />
    </>
  );
}

function CloudExits({
  lost,
  revokes,
  held,
  run,
  fail,
  look,
  onForget,
  ask,
}: {
  lost: boolean;
  revokes: boolean;
  held: boolean;
  run: Run;
  fail: (word?: Word) => void;
  look: () => void;
  onForget: () => void;
  ask: (danger: Danger) => Promise<boolean>;
}) {
  const [counting, setCounting] = useState(false);

  // Google will never let this copy in again, so it is left as it is, saying first what stays only in Drive.
  const abandon = async () => {
    if (held || counting) return;
    setCounting(true);
    const left = await cloudStranded()
      .catch((e) => {
        fail({ card: "cloudAbandon", text: saidPlainly(e) });
        return null;
      })
      .finally(() => setCounting(false));
    if (!left) return;
    const asked =
      left.files === 0
        ? t("cloudAbandonSureNone")
        : left.files === 1
          ? t("cloudAbandonSureOne")
          : fill("cloudAbandonSure", String(left.files));
    if (!(await ask(DANGERS.abandon(asked)))) return;
    run("cloudAbandon", () => cloudAbandon(), look);
  };

  const revoke = async () => {
    if (held || !(await ask(DANGERS.revoke()))) return;
    run("cloudRevoke", () => cloudRevoke(), look);
  };

  return (
    <ZoneButtons
      exits={[
        {
          row: "cloudForget",
          which: "cloudForget",
          why: t("cloudForgetWhy"),
          label: t("cloudForgetHere"),
          onPress: onForget,
          disabled: held,
        },
        lost && {
          row: "cloudAbandon",
          which: "cloudAbandon",
          why: t("cloudAbandonWhy"),
          label: counting ? t("checkingNow") : t("cloudAbandon"),
          onPress: abandon,
          disabled: held || counting,
        },
        revokes && {
          row: "cloudRevoke",
          which: "cloudRevoke",
          why: t("cloudRevokeWhy"),
          label: t("cloudRevokeAll"),
          onPress: revoke,
          disabled: held,
        },
      ]}
    />
  );
}
