import { openUrl } from "@tauri-apps/plugin-opener";
import { useCallback, useEffect, useState } from "react";
import { cloudStanding, type Exit, STORAGE } from "../cloudState";
import { carding, unstuck, useAwaitsChoice } from "../coming";
import {
  type Carrying,
  type CloudStatus,
  chooseSync,
  cloudAbandon,
  cloudChoose,
  cloudConnect,
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
import type { Run, Which, Word } from "./Card";
import Here from "./Here";
import { mild, risky, strong } from "./Rows";
import { useSince } from "./useSince";

interface Props {
  state: Carrying;
  busy: Which | null;
  busySince?: number;
  said?: Word;
  trouble?: Word;
  run: Run;
  fail: (word?: Word) => void;
  carry: (way?: "again") => void;
  onMove: () => void;
}

const SIGNED = new Set(["cloudLost", "cloudElsewhere"]);

export default function Clouded({
  state,
  busy,
  busySince,
  said,
  trouble,
  run,
  fail,
  carry,
  onMove,
}: Props) {
  const [status, setStatus] = useState<CloudStatus | null>(null);
  const [which, setWhich] = useState(false);
  const [signedAt, setSignedAt] = useState<number>();
  const held = busy !== null;
  const waiting = useAwaitsChoice();
  const running = useSince(busy === "sync", busySince);

  const look = useCallback(() => {
    cloudStatus()
      .then(setStatus)
      .catch(() => undefined);
  }, []);

  useEffect(look, [look, state.last, busy]);

  const last = status?.last;
  // A sign-in that went through answers the round that found the door shut, before the next round ends.
  const answered =
    last !== undefined && last !== null && last.at === signedAt && SIGNED.has(last.code);
  const standing = status && !answered ? cloudStanding(status) : null;
  const awaits = waiting || standing?.exit === "choose";
  const carded = standing?.exit && last ? last.code : null;
  const repeats =
    (carded !== null &&
      trouble?.text === plainly({ code: carded, name: last?.name ?? undefined })) ||
    (standing?.urgent === true && trouble?.text === standing.told);

  useEffect(() => {
    carding(carded);
    return () => carding(null);
  }, [carded]);
  const paused = status?.pausedHere ?? false;
  const left = status?.last?.left ?? 0;

  const switched = (on: boolean) => run("sync", cloudPause(!on), look);

  // A copy this machine already carried stays with its account; one never carried may change it.
  const reconnected = async () => {
    const account = await cloudConnect();
    const was = status?.account;
    if (was && was !== account.account && status?.joined) {
      await cloudForget(account.account).catch(() => undefined);
      throw { code: "cloudElsewhere", name: account.email };
    }
    await cloudChoose(account.account, account.email);
    if (was && was !== account.account) await cloudForget(was).catch(() => undefined);
    setSignedAt(status?.last?.at);
    unstuck();
  };

  const reconnect = () => run("sync", reconnected(), () => carry());

  const stop = () => {
    const was = status?.account;
    run(
      "sync",
      chooseSync(undefined).then(() => (was ? cloudForget(was) : undefined)),
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
    openUrl(STORAGE).catch((e) => fail({ card: "sync", text: saidPlainly(e) }));
  };

  const forget = async () => {
    if (held || !(await sure(t("cloudForgetHereSure")))) return;
    run("sync", cloudForget(), look);
  };

  // Google will never let this copy in again, so it is left as it is, saying first what stays only in Drive.
  const abandon = async () => {
    if (held) return;
    const left = await cloudStranded().catch((e) => {
      fail({ card: "sync", text: saidPlainly(e) });
      return null;
    });
    if (!left) return;
    const asked =
      left.files === 0
        ? t("cloudAbandonSureNone")
        : left.files === 1
          ? t("cloudAbandonSureOne")
          : fill("cloudAbandonSure", String(left.files));
    if (!(await sure(asked))) return;
    run("sync", cloudAbandon(), look);
  };

  const revoke = async () => {
    if (held || !(await sure(t("cloudRevokeAllSure")))) return;
    run("sync", cloudRevoke(), look);
  };

  return (
    <>
      <div className="flex items-center gap-2.5 rounded-[10px] border border-hair px-3 py-2">
        <span className="grid size-6 flex-none place-items-center rounded-md bg-[#1a73e8] text-[11.5px] font-semibold text-white">
          G
        </span>
        <span className="min-w-0 flex-1">
          <span className="block text-[13px] font-medium">{t("cloudGoogle")}</span>
          <span className="block truncate text-[11.5px] text-faint">
            {status?.email ?? state.cloud}
          </span>
        </span>
        {!standing && (
          <span className="flex-none rounded-md bg-hue-green/10 px-1.5 py-0.5 text-[10.5px] text-hue-green">
            {t("cloudConnectedTag")}
          </span>
        )}
      </div>

      <p className="mt-2 text-[11.5px] leading-relaxed text-faint">{t("cloudWhere")}</p>

      {standing && (
        <div
          role="status"
          className="mt-2.5 flex items-start gap-2.5 rounded-[10px] border border-hue-amber/40 px-3 py-2"
        >
          <span className="min-w-0 flex-1 text-[11.5px] leading-relaxed text-soft">
            <span className="block text-[12.5px] font-semibold text-ink">
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

      <div className="mt-2.5 flex flex-wrap items-center gap-2.5">
        <button type="button" disabled={held || paused} onClick={() => carry()} className={strong}>
          {busy === "sync" ? syncingSaid(running) : t("syncNow")}
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

      <div className="mt-3 grid grid-cols-2 gap-2 border-t border-hair pt-3">
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
        <button type="button" disabled={held} onClick={forget} className={mild}>
          {t("cloudForgetHere")}
        </button>
        {status?.lost && (
          <button type="button" disabled={held} onClick={abandon} className={risky}>
            {t("cloudAbandon")}
          </button>
        )}
        <button type="button" disabled={held} onClick={revoke} className={risky}>
          {t("cloudRevokeAll")}
        </button>
      </div>

      <Here held={held} />
      {trouble?.card === "sync" && !repeats && (
        <p className="mt-2 text-[11.5px] text-urgent">{trouble.text}</p>
      )}
      {said?.card === "sync" && !(awaits && said.busy) && (
        <p className="mt-2 text-[11.5px] text-faint">{said.text}</p>
      )}
    </>
  );
}
