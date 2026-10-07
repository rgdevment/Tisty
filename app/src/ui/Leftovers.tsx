import { useState } from "react";
import {
  type About,
  type Astray,
  type Gone,
  type Reviewed,
  revealed,
  type Stray,
  type Twins,
  twinned,
} from "../core";
import { weigh } from "../format";
import { fill, t } from "../locales";
import { saidPlainly } from "../refusal";
import { type Brittle, scanned } from "../scanning";
import Card, { type Which, type Word } from "./Card";
import { hushed, hushedName } from "./Keys";
import { dated, Group, mild } from "./Rows";

type Run = <T>(card: Which, work: Promise<T>, then: (answer: T) => void) => void;

interface Props {
  audit: Reviewed | null;
  build: About | null;
  busy: Which | null;
  said?: Word;
  trouble?: Word;
  run: Run;
  fail: (word?: Word) => void;
  letGo: (reference: string, shared?: boolean) => void;
  letGoOfAll: (astray: Astray[]) => void;
  takeIn: (file: string) => void;
  takeInAll: (strays: Stray[]) => void;
  letGoOfPaper: (one: Stray) => void;
  forgetMissing: (one: Gone) => void;
}

export default function Leftovers({
  audit,
  build,
  busy,
  said,
  trouble,
  run,
  fail,
  letGo,
  letGoOfAll,
  takeIn,
  takeInAll,
  letGoOfPaper,
  forgetMissing,
}: Props) {
  const held = busy !== null;
  const [alike, setAlike] = useState<Twins[] | null>(null);
  const [brittle, setBrittle] = useState<Brittle[] | null>(null);

  return (
    <>
      <Group label={t("looseAre")} />

      <Card title={t("looseAre")} which="review" busy={busy} said={said} trouble={trouble}>
        <p className="text-[12.5px] leading-relaxed text-soft">{t("looseWhat")}</p>
        {audit?.machines.some(hushed) && (
          <p className="mt-1.5 text-[12.5px] leading-relaxed text-soft">{t("looseWait")}</p>
        )}
        {audit && audit.loose === 0 && (
          <p className="mt-2 text-[12.5px] text-faint">{t("looseNone")}</p>
        )}
        {audit && audit.loose > 0 && (
          <>
            <p className="mt-2 text-[12.5px] tabular-nums text-soft">
              {`${fill("looseTotal", String(audit.loose))} · ${weigh(audit.looseBytes)}`}
            </p>
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
                      onClick={() => letGo(one.at, one.shared)}
                      className="text-[11.5px] text-urgent hover:underline disabled:text-soft"
                    >
                      {t("looseDrop")}
                    </button>
                  </span>
                </li>
              ))}
            </ul>
            <div className="mt-2.5 flex items-center gap-2.5">
              <button
                type="button"
                disabled={held || audit.loose === 0}
                onClick={() => letGoOfAll(audit.astray)}
                className="rounded-[10px] border border-urgent/40 px-2.5 py-1 text-[12.5px] text-urgent hover:bg-hover disabled:border-hair disabled:text-faint"
              >
                {t("upkeepSafeAll")}
              </button>
              <button
                type="button"
                disabled={!build}
                onClick={() =>
                  build &&
                  revealed(build.store).catch((e) => fail({ card: "review", text: saidPlainly(e) }))
                }
                className={mild}
              >
                {t("aboutReveal")}
              </button>
            </div>
          </>
        )}
      </Card>

      <Group label={t("upkeepLook")} />

      <Card title={t("upkeepLook")} which="review" busy={busy} said={said} trouble={trouble}>
        <p className="text-[12.5px] leading-relaxed text-soft">{t("upkeepLookWhat")}</p>
        {audit && audit.stranded.length === 0 && (
          <p className="mt-2 text-[12.5px] text-faint">{t("upkeepNothing")}</p>
        )}
        {audit && audit.stranded.length > 0 && (
          <>
            <ul className="scroller mt-2 flex max-h-[22rem] flex-col gap-1 overflow-y-auto text-[12.5px]">
              {audit.stranded.map((one) => (
                <li key={one.file} className="flex items-baseline justify-between gap-4">
                  <span className="min-w-0">
                    <span className="block truncate">{one.title || t("untitledDoc")}</span>
                    <span className="block font-mono text-[10.5px] text-faint">
                      {`${one.file} · ${weigh(one.bytes)} · ${dated(one.when)}`}
                    </span>
                  </span>
                  <span className="flex shrink-0 items-baseline gap-2.5">
                    <button
                      type="button"
                      disabled={held}
                      onClick={() => takeIn(one.file)}
                      className="text-[11.5px] text-accent hover:underline disabled:text-soft"
                    >
                      {t("upkeepTakeIn")}
                    </button>
                    <button
                      type="button"
                      disabled={held}
                      onClick={() => letGoOfPaper(one)}
                      className="text-[11.5px] text-urgent hover:underline disabled:text-soft"
                    >
                      {t("upkeepDropIt")}
                    </button>
                  </span>
                </li>
              ))}
            </ul>
            <div className="mt-2.5">
              <button
                type="button"
                disabled={held}
                onClick={() => takeInAll(audit.stranded)}
                className="rounded-[10px] border border-line px-2.5 py-1 text-[12.5px] hover:bg-hover disabled:border-hair disabled:text-faint"
              >
                {t("upkeepTakeInAll")}
              </button>
            </div>
          </>
        )}
      </Card>

      <Group label={fill("upkeepWaiting", hushedName(audit?.machines ?? []) ?? t("theMachines"))} />

      <Card
        title={fill("upkeepWaiting", hushedName(audit?.machines ?? []) ?? t("theMachines"))}
        which="review"
        busy={busy}
        said={said}
        trouble={trouble}
      >
        <p className="text-[12.5px] leading-relaxed text-soft">{t("upkeepWaitingWhat")}</p>
        {audit && audit.missing.length === 0 && (
          <p className="mt-2 text-[12.5px] text-faint">{t("upkeepNothing")}</p>
        )}
        {audit && audit.missing.length > 0 && (
          <ul className="scroller mt-2 flex max-h-[22rem] flex-col gap-1 overflow-y-auto text-[12.5px]">
            {audit.missing.map((one) => (
              <li key={one.file} className="flex items-baseline justify-between gap-4">
                <span className="min-w-0">
                  <span className="block truncate">{one.title || t("untitledDoc")}</span>
                  <span className="block font-mono text-[10.5px] text-faint">{one.file}</span>
                </span>
                <span className="flex shrink-0 items-baseline gap-2.5">
                  {hushedName(audit?.machines ?? []) ? (
                    <span className="text-[11.5px] text-faint">
                      {fill("upkeepForgetWaits", hushedName(audit?.machines ?? []) ?? "")}
                    </span>
                  ) : (
                    <button
                      type="button"
                      disabled={held}
                      onClick={() => forgetMissing(one)}
                      className="text-[11.5px] text-urgent hover:underline disabled:text-soft"
                    >
                      {t("upkeepForget")}
                    </button>
                  )}
                </span>
              </li>
            ))}
          </ul>
        )}
      </Card>

      <Group label={t("twinsAre")} />

      <Card title={t("twinsAre")} which="review" busy={busy} said={said} trouble={trouble}>
        <p className="text-[12.5px] leading-relaxed text-soft">{t("twinsWhat")}</p>
        {alike?.length === 0 && <p className="mt-2 text-[12.5px] text-faint">{t("twinsNone")}</p>}
        {alike && alike.length > 0 && (
          <ul className="scroller mt-2 flex max-h-[22rem] flex-col gap-2 overflow-y-auto text-[12.5px]">
            {alike.map((one) => (
              <li key={one.at.join("|")}>
                <span className="tabular-nums text-faint">{weigh(one.bytes)}</span>
                {one.at.map((named) => (
                  <span key={named} className="block font-mono text-[11.5px] break-all text-soft">
                    {named.replace("attachments/", "")}
                  </span>
                ))}
              </li>
            ))}
          </ul>
        )}
        <div className="mt-2.5 flex items-center gap-2.5">
          <button
            type="button"
            disabled={held}
            onClick={() => run("review", twinned(), setAlike)}
            className={mild}
          >
            {t(alike ? "twinsAgain" : "twinsRun")}
          </button>
        </div>
      </Card>

      <Group label={t("brittleAre")} />

      <Card title={t("brittleAre")} which="brittle" busy={busy} said={said} trouble={trouble}>
        <p className="text-[12.5px] leading-relaxed text-soft">{t("brittleWhat")}</p>
        {brittle?.length === 0 && (
          <p className="mt-2 text-[12.5px] text-faint">{t("brittleNone")}</p>
        )}
        {brittle && brittle.length > 0 && (
          <ul className="scroller mt-2 flex max-h-[22rem] flex-col gap-1.5 overflow-y-auto text-[12.5px]">
            {brittle.map((one) => (
              <li key={one.file}>
                <span className="text-soft">{one.title || one.file}</span>
                <span className="block text-[11.5px] text-faint">
                  {one.brings.map((what) => t(what as Parameters<typeof t>[0])).join(" · ")}
                </span>
              </li>
            ))}
          </ul>
        )}
        <div className="mt-2.5 flex items-center gap-2.5">
          <button
            type="button"
            disabled={held}
            onClick={() => run("brittle", scanned(), setBrittle)}
            className={mild}
          >
            {t(brittle ? "brittleAgain" : "brittleRun")}
          </button>
        </div>
      </Card>
    </>
  );
}
