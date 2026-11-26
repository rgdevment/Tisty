import type { ButtonHTMLAttributes } from "react";
import type { Wired } from "../core";
import { fill, t } from "../locales";
import StateDot from "./StateDot";

/// One line of the Hooks group: whose hooks they are and whether they run on this computer.
export interface HookLine {
  readonly id: string;
  readonly name: string;
  readonly on: boolean;
}

/// Tisty's own hooks, one line per connected agent; hooks brought by its plugin run too.
export const tistyHookLines = (agents: readonly Wired[] | null): HookLine[] =>
  (agents ?? [])
    .filter((one) => one.wired && !!one.hooks)
    .map((one) => ({ id: one.id, name: one.name, on: !!(one.hooks?.on || one.hooks?.plugged) }));

interface Props {
  /// What Settings already read of the connected agents; nothing more is read to draw these rows.
  agents: readonly Wired[] | null;
  onOpen: () => void;
  /// Makes each line a row of the tree it sits in, reached by its arrow keys.
  rowOf?: (key: string) => ButtonHTMLAttributes<HTMLButtonElement>;
}

/// The lines of the Hooks group, read only: their switch lives in Settings, which a press opens.
export default function HooksRow({ agents, onOpen, rowOf }: Readonly<Props>) {
  const lines = tistyHookLines(agents);
  if (lines.length === 0)
    return <p className="px-2 py-1 text-[12.5px] text-faint">{t("agentGroupHooksNone")}</p>;
  return (
    <ul aria-label={t("agentGroupHooksLabel")}>
      {lines.map((one) => {
        const here = fill("agentGroupHooksHere", one.name);
        const said = `${here}: ${t(one.on ? "agentGroupHooksOn" : "agentGroupHooksOff")}`;
        return (
          <li key={one.id}>
            <button
              type="button"
              {...rowOf?.(`hooks:${one.id}`)}
              onClick={onOpen}
              aria-label={said}
              title={`${said} · ${t("agentGroupHooksOpen")}`}
              className="flex w-full items-center justify-between gap-2 rounded-md px-2 py-1 text-left text-[13px] text-ink hover:bg-hover"
            >
              <span className="truncate">{here}</span>
              <StateDot on={one.on} />
            </button>
          </li>
        );
      })}
    </ul>
  );
}
