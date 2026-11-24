import { type ReactNode, useEffect, useRef } from "react";
import { allParts, filesOf, filesOut, type Parts } from "../agentsAmiss";
import { partsSaid, pathsSaid } from "../agentsSaid";
import type { Wired } from "../core";
import { fill, t, type Word } from "../locales";
import { mild, strong } from "./Rows";

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
