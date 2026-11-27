import { type ReactNode, useEffect, useId, useRef, useState } from "react";
import { allParts, filesOf, filesOut, type Parts } from "../agentsAmiss";
import { takesHooks } from "../agentsOffered";
import { partsSaid, pathsSaid, placeNamed } from "../agentsSaid";
import type { TistySkills } from "../agentsSync";
import type { Wired } from "../core";
import { fill, t, type Word } from "../locales";
import { mild, strong } from "./Rows";
import { agentsTaught, skillFolders, teaches } from "./TistySkills";

export function Files({ files }: Readonly<{ files: string[] }>) {
  return files.map((at) => (
    <span key={at} className="block font-mono text-[11.5px] break-all text-faint">
      {at}
    </span>
  ));
}

export function PluginSaid({ one, parts }: Readonly<{ one: Wired; parts: Parts }>) {
  if (parts.hooks.length === 0) return null;
  const asks = one.id.endsWith("claude-code") ? "hooksAskPlugin" : undefined;
  const said = one.hooks?.plugged ? "mendHooksGo" : asks;
  return said ? <span className="block text-[11.5px] text-soft">{t(said)}</span> : null;
}

export function Ticked({
  on,
  onTurn,
  label,
  children,
}: Readonly<{
  on: boolean;
  onTurn: (on: boolean) => void;
  label: string;
  children: ReactNode;
}>) {
  const said = useId();
  return (
    <div>
      <label className="flex cursor-pointer items-start gap-2 font-medium">
        <input
          type="checkbox"
          checked={on}
          aria-describedby={said}
          onChange={(event) => onTurn(event.target.checked)}
          className="mt-0.5"
        />
        <span className="min-w-0 [overflow-wrap:anywhere]">{label}</span>
      </label>
      <p id={said} className="mt-1 pl-[22px] leading-relaxed text-soft [overflow-wrap:anywhere]">
        {children}
      </p>
    </div>
  );
}

export function ExtrasTicks({
  hooks,
  skills,
  withHooks,
  onHooks,
  withSkills,
  onSkills,
}: Readonly<{
  hooks: readonly Wired[];
  skills: TistySkills | null;
  withHooks: boolean;
  onHooks: (on: boolean) => void;
  withSkills: boolean;
  onSkills: (on: boolean) => void;
}>) {
  return (
    <>
      {hooks.length > 0 && (
        <Ticked
          on={withHooks}
          onTurn={onHooks}
          label={fill("extrasHooks", hooks.map((one) => placeNamed(one)).join(", "))}
        >
          {fill("hooksAsk", hooks.map((one) => one.hooks?.at ?? "").join(", "))}
          {hooks.some((one) => one.id === "claude-code") && (
            <span className="block">{t("hooksAskPlugin")}</span>
          )}
        </Ticked>
      )}
      {skills && (
        <Ticked
          on={withSkills}
          onTurn={onSkills}
          label={fill("extrasSkills", agentsTaught(skills).join(", "))}
        >
          <span className="block">{t("tistySkillsLead")}</span>
          <Files files={skillFolders(skills)} />
          <span className="block">{t("tistySkillsAsk")}</span>
        </Ticked>
      )}
    </>
  );
}

interface Answered {
  held: boolean;
  onYes: () => void;
  onNo: () => void;
}

interface Asking extends Answered {
  title: string;
  yes: Word;
  dress: string;
  children: ReactNode;
}

function Ask({ title, yes, dress, children, held, onYes, onNo }: Readonly<Asking>) {
  const asked = useRef<HTMLElement>(null);
  useEffect(() => asked.current?.focus(), []);
  return (
    <section
      ref={asked}
      tabIndex={-1}
      aria-label={title}
      className={`mt-2 rounded-[10px] border px-3 py-2.5 text-[12.5px] ${dress}`}
    >
      <p className="font-semibold">{title}</p>
      {children}
      <p className="mt-1.5 text-[11.5px] text-faint">{t("mendKept")}</p>
      <div className="mt-2 flex gap-2">
        <button type="button" disabled={held} onClick={onYes} className={strong}>
          {t(yes)}
        </button>
        <button type="button" onClick={onNo} className={mild}>
          {t("agentsNotNow")}
        </button>
      </div>
    </section>
  );
}

export function OutAsk({ one, ...answered }: Readonly<Answered & { one: Wired }>) {
  return (
    <Ask title={fill("outAsk", one.name)} yes="outYes" dress="border-line" {...answered}>
      <p className="mt-1.5 text-soft">{t("mendWrites")}</p>
      <Files files={filesOut(one)} />
    </Ask>
  );
}

export interface JoinWith {
  hooks: boolean;
  skills: boolean;
}

export function JoinAsk({
  all,
  skills,
  held,
  onYes,
  onNo,
}: Readonly<{
  all: Wired[];
  skills?: TistySkills | null;
  held: boolean;
  onYes: (withThem: JoinWith) => void;
  onNo: () => void;
}>) {
  const [withHooks, setWithHooks] = useState(true);
  const [withSkills, setWithSkills] = useState(true);
  const taught = skills && teaches(all, skills) ? skills : undefined;
  const hooking = all.filter(takesHooks);
  const files = hooking.map((one) => one.hooks?.at ?? "");
  const written = [...all.map((one) => one.at), ...(withHooks ? files : [])];
  const title =
    all.length === 1 ? fill("joinTitle", all[0].name) : fill("joinTitleAll", String(all.length));
  return (
    <Ask
      title={title}
      yes="joinYes"
      dress="border-accent/40"
      held={held}
      onYes={() => onYes({ hooks: withHooks, skills: withSkills && !!taught })}
      onNo={onNo}
    >
      <p className="mt-1.5 text-soft">{t("mendWrites")}</p>
      <Files files={[...new Set(written)]} />
      {hooking.length > 0 && (
        <div className="mt-2">
          <Ticked
            on={withHooks}
            onTurn={setWithHooks}
            label={fill("joinHooks", hooking.map((one) => one.name).join(", "))}
          >
            {fill("hooksAsk", files.join(", "))}
            {hooking.some((one) => one.id.endsWith("claude-code")) && (
              <span className="block">{t("hooksAskPlugin")}</span>
            )}
          </Ticked>
        </div>
      )}
      {taught && (
        <div className="mt-2">
          <Ticked
            on={withSkills}
            onTurn={setWithSkills}
            label={fill("extrasSkills", agentsTaught(taught).join(", "))}
          >
            <Files files={skillFolders(taught)} />
            <span className="block">{t("tistySkillsAsk")}</span>
          </Ticked>
        </div>
      )}
    </Ask>
  );
}

export default function MendAsk({
  one,
  folders,
  ...answered
}: Readonly<Answered & { one: Wired; folders: string[] }>) {
  const parts = allParts(one);
  const title = fill("mendAsk", one.name, partsSaid(parts));
  return (
    <Ask title={title} yes="mendYes" dress="border-hue-amber/40" {...answered}>
      {pathsSaid(one, parts).map((said) => (
        <p key={said} className="text-soft [overflow-wrap:anywhere]">
          {said}
        </p>
      ))}
      <p className="mt-1.5 text-soft">{t("mendWrites")}</p>
      <Files files={[...filesOf(one, parts), ...folders]} />
      <PluginSaid one={one} parts={parts} />
    </Ask>
  );
}
