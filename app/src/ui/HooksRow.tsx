import type { ButtonHTMLAttributes } from "react";
import type { Wired } from "../core";
import { fill, t } from "../locales";
import Glyph from "./Glyph";

/// One connected agent and whether Tisty's hooks run in it on this computer.
export interface HookLine {
  readonly id: string;
  readonly name: string;
  readonly on: boolean;
}

/// Tisty's own hooks, one line per connected agent; hooks brought by its plugin run too.
export const tistyHookLines = (agents: readonly Wired[] | null): HookLine[] =>
  (agents ?? [])
    .filter((one) => one.wired && !!one.hooks)
    .map((one) => ({
      id: one.id,
      name: one.name,
      on: !!(one.hooks?.on || one.hooks?.plugged),
    }));

/// How the row says where they run, in words: all, none, or only the agents that run them.
export const hooksStanding = (lines: readonly HookLine[]): { said: string; on: boolean } => {
  const on = lines.filter((one) => one.on);
  if (on.length === 0) return { said: t("agentGroupHooksAllOff"), on: false };
  if (on.length === lines.length) return { said: t("agentGroupHooksAllOn"), on: true };
  return { said: fill("agentGroupHooksOnlyIn", on.map((one) => one.name).join(", ")), on: true };
};

interface Props {
  /// What Settings already read of the connected agents; nothing more is read to draw this row.
  agents: readonly Wired[] | null;
  depth: number;
  step: number;
  onOpen: () => void;
  /// Makes the row one more of the tree it sits in, reached by its arrow keys.
  rowOf?: (key: string) => ButtonHTMLAttributes<HTMLButtonElement>;
}

/// Tisty's hooks as one read-only row: their switch lives in Settings, which a press opens.
export default function TistyHooksRow({ agents, depth, step, onOpen, rowOf }: Readonly<Props>) {
  const lines = tistyHookLines(agents);
  if (lines.length === 0) return null;
  const standing = hooksStanding(lines);
  const each = lines
    .map((one) => `${one.name}: ${t(one.on ? "agentGroupHooksOn" : "agentGroupHooksOff")}`)
    .join("; ");
  const said = `${t("agentGroupTistyHooks")}: ${each}`;
  // Codex runs them only once the person approves them there, which only Settings otherwise says.
  const approve = lines.some((one) => one.on && one.id.endsWith("codex"))
    ? ` · ${t("agentGroupHooksCodex")}`
    : "";
  return (
    <li className="relative">
      <div className="relative flex items-center rounded-md has-[:focus-visible]:bg-hover">
        <span
          aria-hidden="true"
          className="grid h-5 w-[18px] shrink-0 place-items-center text-faint"
          style={{ marginLeft: `${8 + depth * step}px` }}
        >
          <Glyph name="bolt" className="h-[13px] w-[13px]" />
        </span>
        <button
          type="button"
          {...rowOf?.("tisty-hooks")}
          onClick={onOpen}
          aria-label={said}
          title={`${said}${approve} · ${t("agentGroupHooksOpen")}`}
          className="flex min-w-0 flex-1 items-center gap-1.5 rounded-md py-1 pl-1.5 pr-2 text-left text-[13px] text-soft hover:bg-hover"
        >
          <span className="truncate">{t("agentGroupTistyHooks")}</span>
          <span
            className={`ml-auto shrink-0 whitespace-nowrap text-[11.5px] ${
              standing.on ? "text-hue-green" : "text-faint"
            }`}
          >
            {standing.said}
          </span>
        </button>
      </div>
    </li>
  );
}
