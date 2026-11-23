import type { ReactNode } from "react";
import type { DropAnswer } from "../agentsDropping";
import type { Both, Loose, Reach, Rounded } from "../agentsSync";
import { fill, t } from "../locales";
import ComparedLines from "./ComparedLines";
import Modal from "./Modal";

export const button = "rounded-[10px] px-3 py-1.5 text-faint hover:text-ink";
export const primary =
  "rounded-[10px] bg-accent px-3.5 py-1.5 font-semibold text-bg disabled:opacity-60";

export type Waiting =
  | { kind: "both"; both: Both }
  | { kind: "loose"; loose: Loose }
  | { kind: "unled"; assistant: string };

// A file the person let stay is asked about only when they open it themselves.
export const waitingOf = (
  said: Pick<Rounded, "both" | "loose" | "unled">,
  stayed = false,
): Waiting[] => [
  ...said.both.map((both): Waiting => ({ kind: "both", both })),
  ...(said.loose ?? [])
    .filter((loose) => stayed || !loose.stayed)
    .map((loose): Waiting => ({ kind: "loose", loose })),
  ...(said.unled ?? []).map((assistant): Waiting => ({ kind: "unled", assistant })),
];

interface Shared {
  readonly name: string;
  readonly busy: boolean;
  readonly trouble?: string;
}

function Footer({ trouble, children }: Readonly<{ trouble?: string; children: ReactNode }>) {
  return (
    <>
      {trouble && <p className="mt-2.5 text-[11.5px] text-urgent">{trouble}</p>}
      <div className="mt-5 flex flex-wrap justify-end gap-2 text-[12.5px]">{children}</div>
    </>
  );
}

export function BothReview({
  both,
  name,
  busy,
  trouble,
  bringable,
  onLater,
  onKeep,
  onBring,
}: Shared &
  Readonly<{
    both: Both;
    bringable: boolean;
    onLater: () => void;
    onKeep: (tisty: boolean) => void;
    onBring: () => void;
  }>) {
  return (
    <Modal title={t("agentsBothTitle")} wide>
      <p className="text-[13px] leading-relaxed text-soft">{fill("agentsBothLead", name)}</p>
      <p className="mt-1.5 font-mono text-[11.5px] break-all text-faint">{both.at}</p>
      <ComparedLines lines={both.lines} gone={t("agentsInTisty")} came={t("agentsInFile")} />
      {bringable && <p className="mt-2.5 text-[12.5px] text-soft">{t("agentsBothBring")}</p>}
      {both.composed && !bringable && (
        <p className="mt-2.5 text-[12.5px] text-soft">{t("agentsBothComposed")}</p>
      )}
      <Footer trouble={trouble}>
        <button type="button" disabled={busy} onClick={onLater} className={button}>
          {t("agentsLater")}
        </button>
        {!both.composed && (
          <button type="button" disabled={busy} onClick={() => onKeep(false)} className={button}>
            {t("agentsKeepFile")}
          </button>
        )}
        {bringable && (
          <button type="button" disabled={busy} onClick={onBring} className={button}>
            {t("agentsBringIn")}
          </button>
        )}
        <button type="button" disabled={busy} onClick={() => onKeep(true)} className={primary}>
          {t("agentsKeepTisty")}
        </button>
      </Footer>
    </Modal>
  );
}

export function LooseReview({
  loose,
  name,
  busy,
  trouble,
  onAnswer,
}: Shared & Readonly<{ loose: Loose; onAnswer: (away: boolean) => void }>) {
  return (
    <Modal title={t("agentsLooseTitle")} wide>
      <p className="text-[13px] leading-relaxed text-soft">{fill("agentsLooseLead", name)}</p>
      <p className="mt-1.5 font-mono text-[11.5px] break-all text-faint">{loose.at}</p>
      {loose.changed && (
        <p className="mt-2.5 text-[12.5px] text-hue-amber">{t("agentsLooseChanged")}</p>
      )}
      <p className="mt-2.5 text-[12.5px] text-soft">{t("agentsLooseBackup")}</p>
      <Footer trouble={trouble}>
        <button type="button" disabled={busy} onClick={() => onAnswer(false)} className={button}>
          {t("agentsLooseStay")}
        </button>
        <button type="button" disabled={busy} onClick={() => onAnswer(true)} className={primary}>
          {t("agentsLooseAway")}
        </button>
      </Footer>
    </Modal>
  );
}

export function UnledReview({
  name,
  busy,
  trouble,
  onLater,
  onLead,
}: Shared & Readonly<{ onLater: () => void; onLead: () => void }>) {
  return (
    <Modal title={t("agentsUnledTitle")} wide>
      <p className="text-[13px] leading-relaxed text-soft">{fill("agentsUnledLead", name)}</p>
      <Footer trouble={trouble}>
        <button type="button" disabled={busy} onClick={onLater} className={button}>
          {t("agentsLeavePaused")}
        </button>
        <button type="button" disabled={busy} onClick={onLead} className={primary}>
          {t("agentsLeadAgain")}
        </button>
      </Footer>
    </Modal>
  );
}

export const namesJoined = (names: readonly string[]): string =>
  names.length < 2
    ? names.join("")
    : `${names.slice(0, -1).join(", ")} ${t("agentsAnd")} ${names[names.length - 1]}`;

export function DropReview({
  said,
  reach,
  onAnswer,
}: Readonly<{
  said: string;
  reach: Reach;
  onAnswer: (answer: DropAnswer) => void;
}>) {
  const away = reach.agents.length > 0;
  const paused = reach.paused.length > 1 ? "agentsDropPausedMany" : "agentsDropPaused";
  return (
    <Modal title={t("sureTitle")} wide onClose={() => onAnswer(null)}>
      <p className="text-[13px] leading-relaxed whitespace-pre-line text-soft">{said}</p>
      {away && (
        <p className="mt-2 text-[13px] font-semibold text-ink">
          {fill("agentsDropAsk", namesJoined(reach.agents))}
        </p>
      )}
      {reach.changed.length > 0 && (
        <p className="mt-2.5 text-[12.5px] text-hue-amber">
          {fill("agentsDropChanged", namesJoined(reach.changed))}
        </p>
      )}
      {reach.paused.length > 0 && (
        <p className="mt-2.5 text-[12.5px] text-soft">{fill(paused, namesJoined(reach.paused))}</p>
      )}
      {reach.rewritten.length > 0 && (
        <p className="mt-2.5 text-[12.5px] text-soft">
          {fill("agentsDropRewritten", namesJoined(reach.rewritten))}
        </p>
      )}
      {away && <p className="mt-2.5 text-[12.5px] text-soft">{t("agentsDropBackup")}</p>}
      <Footer>
        <button type="button" onClick={() => onAnswer(null)} className={button}>
          {t("cancel")}
        </button>
        <button type="button" onClick={() => onAnswer("tisty")} className={away ? button : primary}>
          {t("agentsDropOnlyTisty")}
        </button>
        {away && (
          <button type="button" onClick={() => onAnswer("away")} className={primary}>
            {t("agentsDropAway")}
          </button>
        )}
      </Footer>
    </Modal>
  );
}
