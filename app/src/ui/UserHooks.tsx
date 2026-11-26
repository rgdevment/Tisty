import { type ReactNode, useCallback, useEffect, useState } from "react";
import type { Filed } from "../core";
import { fill, t, type Word } from "../locales";
import { saidPlainly } from "../refusal";
import {
  type AgentHook,
  type HookEvent,
  type HookLook,
  type HookState,
  hookLook,
  hookRegister,
  hookTry,
  hookUnregister,
  type Tried,
} from "../userHooks";
import type { RowOf } from "./AgentGroupRows";
import Modal from "./Modal";

const STATE_WORD: Record<HookState, Word> = {
  registered: "agentHookRegistered",
  waiting: "agentHookWaiting",
  changed: "agentHookChanged",
  notAgreed: "agentHookNotAgreed",
  placedByAgent: "agentHookNotAgreed",
  elsewhere: "agentHookElsewhere",
  away: "agentHookAway",
};

export const EVENT_WORD: Record<HookEvent, Word> = {
  start: "agentHookEventStart",
  prompt: "agentHookEventPrompt",
  "before-tool": "agentHookEventBeforeTool",
  "after-tool": "agentHookEventAfterTool",
  stop: "agentHookEventStop",
};

const button =
  "shrink-0 rounded-md border border-line px-2 py-0.5 text-[11.5px] text-soft hover:border-accent hover:text-accent disabled:text-faint";
const primary =
  "shrink-0 rounded-md bg-accent px-2.5 py-1 text-[12.5px] text-bg disabled:bg-hair disabled:text-soft";

const registrable = (look: HookLook) =>
  look.state === "waiting" || look.state === "changed" || look.state === "registered";

const agentSaid = (one: AgentHook): Word => {
  if (!one.takes) return "agentHookAgentNoMoment";
  if (one.unread) return "agentHookAgentUnread";
  if (one.apart) return "agentHookAgentApart";
  if (one.stuck === "puzzling") return "agentHookAgentPuzzling";
  if (one.present && !one.here) return "agentHookAgentAstray";
  if (one.present) return "agentHookAgentOk";
  return one.installed ? "agentHookAgentMissing" : "agentHookAgentNotInstalled";
};

const triedSaid = (tried: Tried): string => {
  if (tried.missing) return fill("agentHookTriedMissing", tried.missing);
  if (tried.timedOut) return t("agentHookTriedLate");
  if (tried.code === 2) return t("agentHookTriedBlock");
  return fill("agentHookTriedPass", String(tried.code ?? "?"));
};

function Script({ text }: Readonly<{ text: string }>) {
  return (
    <pre className="scroller mt-2 max-h-64 overflow-auto rounded-md border border-hair bg-hover px-3 py-2 font-mono text-[12.5px] whitespace-pre-wrap">
      {text}
    </pre>
  );
}

function RegisterAsk({
  look,
  onYes,
  onClose,
}: Readonly<{ look: HookLook; onYes: () => void; onClose: () => void }>) {
  const touched = look.here.filter((one) => one.takes && one.installed);
  return (
    <Modal title={fill("agentHookAskTitle", look.name)} wide onClose={onClose}>
      <p className="text-[13px] leading-relaxed text-soft">
        {fill("agentHookAskSaid", t(EVENT_WORD[look.event]).toLowerCase())}
      </p>
      <Script text={look.script} />
      <p className="mt-3 text-[12.5px] font-medium">{t("agentHookAskFiles")}</p>
      <ul className="mt-1 text-[12.5px] text-soft">
        {touched.map((one) => (
          <li key={one.agent} className="[overflow-wrap:anywhere]">
            {one.name} · {one.at}
          </li>
        ))}
      </ul>
      <div className="mt-4 flex justify-end gap-2">
        <button type="button" onClick={onClose} className={button}>
          {t("agentHookNotNow")}
        </button>
        <button type="button" onClick={onYes} className={primary}>
          {t("agentHookAskYes")}
        </button>
      </div>
    </Modal>
  );
}

function VerifyPanel({
  look,
  onTry,
  onUnregister,
  onClose,
}: Readonly<{
  look: HookLook;
  onTry: () => Promise<Tried>;
  onUnregister: () => void;
  onClose: () => void;
}>) {
  const [asking, setAsking] = useState(false);
  const [tried, setTried] = useState<Tried | null>(null);
  const [trouble, setTrouble] = useState<string | null>(null);
  const run = () => {
    setAsking(false);
    onTry()
      .then(setTried)
      .catch((problem) => setTrouble(saidPlainly(problem)));
  };
  const registered = look.state === "registered";
  return (
    <Modal title={`${t("agentHookVerify")} · ${look.name}`} wide closable onClose={onClose}>
      <p className="text-[13px] font-medium">{t(STATE_WORD[look.state])}</p>
      {!look.door && <p className="mt-1 text-[12.5px] text-hue-amber">{t("agentHookDoorShut")}</p>}
      {registered && !look.kept && (
        <p className="mt-1 text-[12.5px] text-hue-amber">{t("agentHookScriptGone")}</p>
      )}
      <ul className="mt-2 flex flex-col gap-1 text-[12.5px]">
        {look.here.map((one) => (
          <li key={one.agent} className="[overflow-wrap:anywhere]">
            <span className="font-medium">{one.name}</span>
            <span className="text-soft"> · {t(agentSaid(one))}</span>
            {one.present && one.trustedByHand && (
              <span className="block text-soft">{t("agentGroupHooksCodex")}</span>
            )}
          </li>
        ))}
      </ul>
      {asking && (
        <div className="mt-3 rounded-md border border-hue-amber/40 px-3 py-2">
          <p className="text-[12.5px] text-hue-amber">{t("agentHookTrySaid")}</p>
          {!registered && <Script text={look.script} />}
          <div className="mt-2 flex justify-end gap-2">
            <button type="button" onClick={() => setAsking(false)} className={button}>
              {t("agentHookNotNow")}
            </button>
            <button type="button" onClick={run} className={primary}>
              {t("agentHookTryYes")}
            </button>
          </div>
        </div>
      )}
      {tried && (
        <div className="mt-3 text-[12.5px]">
          <p className="font-medium">
            {triedSaid(tried)} · {fill("agentHookTriedTook", String(tried.tookMs))}
          </p>
          {tried.said && <Script text={tried.said} />}
          <p className="mt-1 text-soft">{t("agentHookTriedPath")}</p>
        </div>
      )}
      {trouble && <p className="mt-2 text-[12.5px] text-urgent">{trouble}</p>}
      <div className="mt-4 flex flex-wrap justify-end gap-2">
        {registered && (
          <button type="button" onClick={onUnregister} className={button}>
            {t("agentHookUnregister")}
          </button>
        )}
        {registrable(look) && !asking && (
          <button type="button" onClick={() => setAsking(true)} className={button}>
            {t("agentHookTry")}
          </button>
        )}
      </div>
    </Modal>
  );
}

function HookActions({ doc, rowOf }: Readonly<{ doc: Filed; rowOf: RowOf }>) {
  const [look, setLook] = useState<HookLook | null>(null);
  const [open, setOpen] = useState<"register" | "verify" | null>(null);
  const [trouble, setTrouble] = useState<string | null>(null);
  const failed = (problem: unknown) => setTrouble(saidPlainly(problem));
  const looked = useCallback(() => {
    setTrouble(null);
    return hookLook(doc.file)
      .then(setLook)
      .catch((problem) => setTrouble(saidPlainly(problem)));
  }, [doc.file]);
  useEffect(() => {
    looked();
  }, [looked]);
  const opened = (which: "register" | "verify") => looked().then(() => setOpen(which));
  const register = (shown: HookLook) => {
    setOpen(null);
    hookRegister(shown.file, shown.print).then(setLook).catch(failed);
  };
  const unregister = (shown: HookLook) => {
    setOpen(null);
    hookUnregister(shown.file).then(setLook).catch(failed);
  };
  return (
    <div className="flex flex-wrap items-center gap-1.5 pb-1 pl-7 text-[11.5px] text-faint">
      {look && <span className="truncate">{t(STATE_WORD[look.state])}</span>}
      <button
        type="button"
        {...rowOf(`hook-register:${doc.file}`)}
        disabled={!look || !registrable(look)}
        onClick={() => opened("register")}
        className={button}
      >
        {t("agentHookRegister")}
      </button>
      <button
        type="button"
        {...rowOf(`hook-verify:${doc.file}`)}
        disabled={!look}
        onClick={() => opened("verify")}
        className={button}
      >
        {t("agentHookVerify")}
      </button>
      {trouble && <span className="basis-full text-urgent">{trouble}</span>}
      {look && open === "register" && (
        <RegisterAsk look={look} onYes={() => register(look)} onClose={() => setOpen(null)} />
      )}
      {look && open === "verify" && (
        <VerifyPanel
          look={look}
          onTry={() => hookTry(look.file, look.print).finally(looked)}
          onUnregister={() => unregister(look)}
          onClose={() => setOpen(null)}
        />
      )}
    </div>
  );
}

/// The person's own hooks under Global › Hooks: each opens as a page, and says here whether it runs.
export default function UserHookRows({
  hooks,
  depth,
  paper,
  rowOf,
}: Readonly<{
  hooks: readonly Filed[];
  depth: number;
  paper: (doc: Filed, depth: number) => ReactNode;
  rowOf: RowOf;
}>) {
  return (
    <>
      {hooks.map((doc) => (
        <li key={`hook:${doc.id}`}>
          <ul>
            {paper(doc, depth)}
            <li>
              <HookActions doc={doc} rowOf={rowOf} />
            </li>
          </ul>
        </li>
      ))}
    </>
  );
}
