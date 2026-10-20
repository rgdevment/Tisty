import { useState } from "react";
import {
  type About,
  type Astray,
  type Carrying,
  checked,
  type Freeing,
  type Joined,
  type Pair,
  type Reviewed,
  retireAttachment,
  retireAttachments,
  revealed,
  twinned,
  twinsJoin,
  type Unused,
} from "../core";
import { weigh } from "../format";
import { fill, t } from "../locales";
import { saidPlainly } from "../refusal";
import { sure } from "../sure";
import { type Run, type Which, type Word, working } from "./Card";
import FreeSpace from "./FreeSpace";
import { hushed } from "./Keys";
import { Band, dated, Line, mild, risky } from "./Rows";
import Twins from "./Twins";

interface Props {
  audit: Reviewed | null;
  setAudit: (audit: Reviewed) => void;
  build: About | null;
  state: Carrying;
  freeing: Freeing | null;
  setFreeing: (freeing: Freeing | null) => void;
  days: number;
  setDays: (days: number) => void;
  busy: Which | null;
  secs: number;
  said?: Word;
  trouble?: Word;
  run: Run;
  tell: (word?: Word) => void;
  fail: (word?: Word) => void;
  onDoc?: (paper: string) => void;
}

export default function FreeUp({
  audit,
  setAudit,
  build,
  state,
  freeing,
  setFreeing,
  days,
  setDays,
  busy,
  secs,
  said,
  trouble,
  run,
  tell,
  fail,
  onDoc,
}: Props) {
  const held = busy !== null;
  const [alike, setAlike] = useState<Pair[] | null>(null);

  const join = (keep: string, go: string[]): Promise<Joined> => {
    const asked = twinsJoin(keep, go);
    run("twins", asked.then(checked), setAudit);
    return asked;
  };
  const [unused, setUnused] = useState<Unused | null>(null);
  const loose = audit?.loose ?? 0;
  const total = (audit?.looseBytes ?? 0) + (state.freesSpace ? (unused?.bytes ?? 0) : 0);

  const letGo = (one: Astray) => {
    if (held) return;
    const named = one.at.split("/").pop() ?? one.at;
    sure(fill(one.shared ? "looseDropSharedSure" : "looseDropSure", named))
      .then(
        (sure) =>
          sure &&
          run("loose", retireAttachment(one.at).then(checked), (now) => {
            setAudit(now);
            tell({ card: "loose", text: t("looseDropped") });
          }),
      )
      .catch((e) => fail({ card: "loose", text: saidPlainly(e) }));
  };

  const letGoOfAll = (astray: Astray[]) => {
    if (held || astray.length === 0) return;
    sure(fill("upkeepSafeAllSure", String(astray.length)))
      .then((sure) => {
        if (!sure) return;
        run("loose", retireAttachments(astray.map((one) => one.at)).then(checked), (now) => {
          setAudit(now);
          tell({ card: "loose", text: t("looseDropped") });
        });
      })
      .catch((e) => fail({ card: "loose", text: saidPlainly(e) }));
  };

  return (
    <>
      <Band label={t("freeTitle")} />
      {audit && (
        <p className="mb-1.5 text-[12.5px] tabular-nums text-soft">
          {total > 0 ? fill("freeTotal", weigh(total)) : t("freeNothing")}
        </p>
      )}
      <div className="border-t border-hair">
        <Line
          title={t("looseAre")}
          why={
            <>
              <span className="block">{t("looseWhat")}</span>
              {audit && loose === 0 && <span className="mt-1 block">{t("looseNone")}</span>}
            </>
          }
          which="loose"
          said={said}
          trouble={trouble}
          more={
            audit &&
            loose > 0 && (
              <>
                {audit.machines.some(hushed) && (
                  <p className="mt-1.5 text-[12.5px] leading-relaxed text-soft">{t("looseWait")}</p>
                )}
                <ul className="scroller mt-2 flex max-h-[22rem] flex-col gap-1 overflow-y-auto text-[12.5px]">
                  {audit.astray.map((one) => (
                    <li key={one.at} className="flex items-baseline justify-between gap-4">
                      <span className="font-mono text-[11.5px] break-all text-soft">
                        {one.at.split("/").pop()}
                      </span>
                      <span className="flex shrink-0 items-baseline gap-2.5 tabular-nums">
                        <span className="text-faint">
                          {`${weigh(one.bytes)} · ${dated(one.when)}${
                            one.shared ? ` · ${t("looseShared")}` : ""
                          }`}
                        </span>
                        <button
                          type="button"
                          disabled={held}
                          onClick={() => letGo(one)}
                          className={risky}
                        >
                          {t("looseDrop")}
                        </button>
                      </span>
                    </li>
                  ))}
                </ul>
                <div className="mt-2.5 flex flex-wrap items-center gap-2.5">
                  <button
                    type="button"
                    disabled={held}
                    onClick={() => letGoOfAll(audit.astray)}
                    className={risky}
                  >
                    {fill("upkeepSafeAll", String(loose), weigh(audit.looseBytes))}
                  </button>
                  <button
                    type="button"
                    disabled={!build}
                    onClick={() =>
                      build &&
                      revealed(build.store).catch((e) =>
                        fail({ card: "loose", text: saidPlainly(e) }),
                      )
                    }
                    className={mild}
                  >
                    {t("aboutReveal")}
                  </button>
                </div>
              </>
            )
          }
        />
        <Line
          title={t("twinsAre")}
          why={t("twinsWhat")}
          which="twins"
          said={said}
          trouble={trouble}
          more={
            alike && (
              <>
                {alike.length === 0 && (
                  <p className="mt-2 text-[12.5px] text-faint">{t("twinsNone")}</p>
                )}
                {alike.length > 0 && (
                  <ul className="scroller mt-2 flex max-h-[32rem] flex-col gap-4 overflow-y-auto">
                    {alike.map((one) => (
                      <Twins
                        key={one.copies.map((copy) => copy.at).join("|")}
                        pair={one}
                        busy={held}
                        onJoin={join}
                        onDoc={onDoc}
                      />
                    ))}
                  </ul>
                )}
              </>
            )
          }
        >
          <button
            type="button"
            disabled={held}
            onClick={() => run("twins", twinned(), setAlike)}
            className={mild}
          >
            {busy === "twins"
              ? working("twinsLooking", secs)
              : t(alike ? "twinsAgain" : "twinsRun")}
          </button>
        </Line>
        {state.freesSpace && (
          <FreeSpace
            busy={busy}
            freeing={freeing}
            setFreeing={setFreeing}
            days={days}
            setDays={setDays}
            said={said}
            trouble={trouble}
            fail={fail}
            onFound={setUnused}
          />
        )}
      </div>
    </>
  );
}
