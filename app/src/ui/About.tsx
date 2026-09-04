import { openUrl } from "@tauri-apps/plugin-opener";
import { useCallback, useEffect, useState } from "react";
import copypaste from "../assets/copypaste.png";
import linkunbound from "../assets/linkunbound.png";
import {
  about,
  type About as Build,
  notices,
  type Ready,
  type Underway,
  updateInstall,
  updateReady,
} from "../core";
import { fill, t } from "../locales";
import { composed } from "../markdown";
import { saidPlainly } from "../refusal";
import Composed from "./Composed";

const COFFEE = "https://buymeacoffee.com/rgdevment";
const SPONSOR = "https://github.com/sponsors/rgdevment";

const TOOLS = [
  {
    icon: copypaste,
    name: "CopyPaste",
    said: "toolCopyPaste",
    at: "https://github.com/rgdevment/CopyPaste",
  },
  {
    icon: linkunbound,
    name: "LinkUnbound",
    said: "toolLinkUnbound",
    at: "https://github.com/rgdevment/LinkUnbound",
  },
] as const;

export default function About({
  ready,
  step,
  onError,
  onGaveUp,
}: {
  ready: Ready | null;
  step?: Underway | null;
  onError: (problem: unknown) => void;
  onGaveUp?: () => void;
}) {
  const [build, setBuild] = useState<Build | null>(null);
  const [trouble, setTrouble] = useState<string | null>(null);
  const [asked, setAsked] = useState(false);
  const [said, setSaid] = useState<string | null>(null);
  const [looking, setLooking] = useState(false);
  const [found, setFound] = useState<Ready | "none" | null>(null);

  const look = useCallback(() => {
    setTrouble(null);
    about()
      .then(setBuild)
      .catch((problem) => setTrouble(saidPlainly(problem)));
  }, []);

  useEffect(look, [look]);

  const lookAgain = () => {
    setLooking(true);
    setFound(null);
    updateReady(true)
      .then((one) => setFound(one ?? "none"))
      .catch(onError)
      .finally(() => setLooking(false));
  };

  const newer = found === "none" ? null : (found ?? ready);

  return (
    <main className="flex flex-col overflow-hidden">
      <div data-tauri-drag-region className="h-9 shrink-0" />
      <div className="scroller mx-auto w-full max-w-[560px] px-6 pb-12">
        {build?.sandbox && (
          <p className="mb-4 rounded-[10px] bg-mark-priority px-4 py-3 text-[12.5px] text-ink">
            {fill("inSandbox", build.sandbox)}
          </p>
        )}

        <div className="flex items-center gap-3.5">
          <span
            aria-hidden="true"
            className="grid size-[52px] shrink-0 place-items-center rounded-[13px] bg-linear-150 from-accent to-[#6f4bd8] text-[26px] font-semibold text-white shadow-lift"
          >
            T
          </span>
          <span className="min-w-0">
            <h2 className="text-[22px] font-semibold tracking-[-0.015em]">Tisty</h2>
            <span className="mt-px flex items-center gap-2 text-[11.5px] text-faint tabular-nums">
              <span>{build?.version ?? "—"}</span>
              <span aria-hidden="true" className="size-[3px] rounded-full bg-line" />
              <span>{build?.license ?? t("aboutBuild")}</span>
            </span>
          </span>
        </div>

        <p className="mt-4 text-[13px] leading-relaxed text-soft">{t("aboutWhat")}</p>
        <p className="mt-1.5 text-[12px] leading-relaxed text-faint">{t("aboutPrivacy")}</p>

        {trouble && (
          <div className="mt-4">
            <p role="alert" className="text-[12.5px] leading-relaxed text-urgent">
              {t("aboutFailed")}
            </p>
            <p className="mt-1 text-[11.5px] leading-relaxed text-faint">{trouble}</p>
            <button type="button" onClick={look} className={`mt-2 ${mild}`}>
              {t("tryAgain")}
            </button>
          </div>
        )}

        {newer ? (
          <div className="mt-5 flex items-center gap-3 rounded-[10px] border border-hair bg-accent-soft px-3.5 py-3">
            <Pip />
            <span className="min-w-0 flex-1">
              <span className="block text-[13px] font-semibold">
                {fill("updateThere", newer.version)}
              </span>
              {step ? (
                step.stage === "installing" ? (
                  <span className="mt-0.5 block text-[12px] text-soft">
                    {t("updateInstalling")}
                  </span>
                ) : (
                  <>
                    <span className="mt-0.5 block text-[12px] text-soft">
                      {fill("updateGetting", `${step.far} %`)}
                    </span>
                    <span className="mt-1.5 block h-1 overflow-hidden rounded-full bg-desk">
                      <span
                        className="block h-full rounded-full bg-accent motion-safe:transition-[width]"
                        style={{ width: `${step.far}%` }}
                      />
                    </span>
                  </>
                )
              ) : (
                <span className="mt-0.5 block text-[12px] text-soft">
                  {newer.installs ? (
                    t("updateAsk")
                  ) : newer.route === "store" ? (
                    t("updateStore")
                  ) : (
                    <code>{fill("updateBrewCli", newer.package ?? "tisty")}</code>
                  )}
                </span>
              )}
            </span>
            {!step && newer.installs && (
              <button
                type="button"
                disabled={asked}
                onClick={() => {
                  setAsked(true);
                  updateInstall().catch((problem) => {
                    setAsked(false);
                    onGaveUp?.();
                    onError(problem);
                  });
                }}
                className="shrink-0 rounded-lg bg-accent px-3 py-1.5 text-[12.5px] font-medium text-white disabled:opacity-60"
              >
                {t("updateInstall")}
              </button>
            )}
          </div>
        ) : (
          <div className="mt-5 flex items-center gap-3">
            <Pip ok />
            <span className="flex-1 text-[12.5px] text-soft">
              {looking ? t("lookingNow") : t("lookNowNone")}
            </span>
            <button type="button" disabled={looking} onClick={lookAgain} className={mild}>
              {t("lookNow")}
            </button>
          </div>
        )}

        <Rule said={t("supportTitle")} />
        <p className="text-[13px] leading-relaxed text-soft">{t("supportWhy")}</p>
        <div className="mt-2.5 grid grid-cols-2 gap-2.5">
          <Gives
            said={t("supportSponsor")}
            where="github.com/sponsors"
            onPick={() => openUrl(SPONSOR).catch(onError)}
          >
            <path
              fill="#db61a2"
              d="M8 14.25 6.84 13.2C2.72 9.47 0 7.01 0 4.5 0 2.42 1.57 1 3.5 1c1.1 0 2.16.51 2.84 1.32h1.32C8.34 1.51 9.4 1 10.5 1 12.43 1 14 2.42 14 4.5c0 2.51-2.72 4.97-6.84 8.7L8 14.25Z"
            />
          </Gives>
          <Gives
            said={t("supportCoffee")}
            where="buymeacoffee.com"
            onPick={() => openUrl(COFFEE).catch(onError)}
          >
            <path
              fill="#c8892a"
              d="M2 5h9v5a3 3 0 0 1-3 3H5a3 3 0 0 1-3-3V5Zm10 0h1.5A2.5 2.5 0 0 1 16 7.5 2.5 2.5 0 0 1 13.5 10H12V5ZM2 14h9v1H2v-1Z"
            />
          </Gives>
        </div>

        <Rule said={t("otherTools")} />
        {TOOLS.map((tool) => (
          <button
            key={tool.name}
            type="button"
            onClick={() => openUrl(tool.at).catch(onError)}
            className="mb-2 flex w-full items-start gap-3 rounded-[10px] border border-hair px-3.5 py-3 text-left outline-none hover:bg-hover focus-visible:ring-2 focus-visible:ring-accent"
          >
            <img src={tool.icon} alt="" className="mt-px size-6 shrink-0 rounded-[6px]" />
            <span className="min-w-0 flex-1">
              <span className="block text-[13px] font-semibold">{tool.name}</span>
              <span className="mt-0.5 block text-[12px] leading-relaxed text-soft">
                {t(tool.said)}
              </span>
            </span>
            <span aria-hidden="true" className="mt-0.5 text-[13px] text-faint">
              ↗
            </span>
          </button>
        ))}

        {build && (
          <div className="mt-4 flex flex-wrap gap-2">
            <button
              type="button"
              onClick={() => openUrl(build.repository).catch(onError)}
              className={mild}
            >
              {t("aboutRepo")}
            </button>
            <button
              type="button"
              onClick={() => {
                if (said !== null) return setSaid(null);
                notices().then(setSaid).catch(onError);
              }}
              aria-expanded={said !== null}
              className={mild}
            >
              {t("aboutNotices")}
            </button>
          </div>
        )}
        {said !== null && (
          <Composed
            label={t("aboutNotices")}
            html={composed(said)}
            onError={onError}
            className="prose scroller mt-2.5 max-h-[380px] rounded-[8px] border border-hair px-3 py-2 text-[12px] leading-relaxed text-soft"
          />
        )}
      </div>
    </main>
  );
}

const mild =
  "rounded-[7px] border border-line px-2.5 py-1 text-[12.5px] hover:bg-hover disabled:border-hair disabled:bg-hair disabled:text-soft";

function Pip({ ok }: { ok?: boolean }) {
  return (
    <span
      aria-hidden="true"
      className={`size-2 shrink-0 rounded-full ${
        ok ? "bg-hue-green ring-3 ring-hue-green/15" : "bg-accent ring-3 ring-accent-soft"
      }`}
    />
  );
}

function Rule({ said }: { said: string }) {
  return (
    <div className="mt-6 mb-2 flex items-center gap-2.5 text-[11.5px] font-semibold tracking-[0.05em] text-faint uppercase">
      <span>{said}</span>
      <span className="h-px flex-1 bg-hair" />
    </div>
  );
}

function Gives({
  said,
  where,
  onPick,
  children,
}: {
  said: string;
  where: string;
  onPick: () => void;
  children: React.ReactNode;
}) {
  return (
    <button
      type="button"
      onClick={onPick}
      className="flex items-center gap-2.5 rounded-[9px] border border-hair bg-panel px-3 py-2.5 text-left outline-none hover:bg-hover focus-visible:ring-2 focus-visible:ring-accent"
    >
      <svg viewBox="0 0 16 16" aria-hidden="true" className="size-[17px] shrink-0">
        {children}
      </svg>
      <span className="min-w-0">
        <span className="block text-[12.5px] font-medium">{said}</span>
        <span className="block truncate text-[11.5px] text-faint">{where}</span>
      </span>
    </button>
  );
}
