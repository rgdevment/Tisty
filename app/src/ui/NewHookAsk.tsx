import { useId, useState } from "react";
import { t, type Word } from "../locales";
import { saidPlainly } from "../refusal";
import {
  HOOK_AGENTS,
  HOOK_EVENTS,
  HOOK_EXTENSIONS,
  HOOK_TOOLS,
  type HookEvent,
  type HookTools,
  hookNameFits,
  hookNew,
  onTools,
} from "../userHooks";
import Modal from "./Modal";
import { onWindows } from "./macKeys";
import { EVENT_WORD } from "./UserHooks";

const TOOLS_WORD: Record<HookTools, Word> = {
  any: "agentHookToolsAny",
  shell: "agentHookToolsShell",
  write: "agentHookToolsWrite",
  mcp: "agentHookToolsMcp",
};

const DEFAULT_TIMEOUT = 10;
const TIMEOUT_AT_MOST = 600;
const field = "w-full rounded-md border border-line bg-bg px-2 py-1 text-[13px]";
const label = "mt-3 block text-[12.5px] font-medium";

interface Props {
  connected: readonly string[];
  onMade: (doc: { id: string; title: string }) => void;
  onClose: () => void;
}

/// A new hook's name and moment: its script is written afterwards on its own page.
export default function NewHookAsk({ connected, onMade, onClose }: Readonly<Props>) {
  const ids = useId();
  const [stem, setStem] = useState("");
  const [ext, setExt] = useState<string>(onWindows ? "ps1" : "sh");
  const [event, setEvent] = useState<HookEvent>("before-tool");
  const [tools, setTools] = useState<HookTools>("shell");
  const [timeout, setTimeoutSeconds] = useState(DEFAULT_TIMEOUT);
  const [agents, setAgents] = useState<string[]>(
    connected.length > 0 ? [...connected] : [HOOK_AGENTS[0][0]],
  );
  const [hereOnly, setHereOnly] = useState(true);
  const [trouble, setTrouble] = useState<string | null>(null);
  const fits =
    hookNameFits(stem) && agents.length > 0 && timeout >= 1 && timeout <= TIMEOUT_AT_MOST;
  const make = () =>
    hookNew({
      name: `${stem}.${ext}`,
      event,
      tools: onTools(event) ? tools : "any",
      timeout,
      agents,
      everyone: !hereOnly,
    })
      .then(onMade)
      .catch((problem) => setTrouble(saidPlainly(problem)));
  const flip = (id: string) =>
    setAgents((now) => (now.includes(id) ? now.filter((one) => one !== id) : [...now, id]));
  return (
    <Modal title={t("agentHookNew")} wide onClose={onClose}>
      <p className="text-[13px] leading-relaxed text-soft">{t("agentHookNewSaid")}</p>
      <label htmlFor={`${ids}-name`} className={label}>
        {t("agentHookName")}
      </label>
      <div className="mt-1 flex gap-2">
        <input
          id={`${ids}-name`}
          value={stem}
          onChange={(e) => setStem(e.target.value.toLowerCase())}
          placeholder={t("agentHookNameHint")}
          className={field}
        />
        <select
          aria-label={t("agentHookKindOf")}
          value={ext}
          onChange={(e) => setExt(e.target.value)}
          className="rounded-md border border-line bg-bg px-2 text-[13px]"
        >
          {HOOK_EXTENSIONS.map((one) => (
            <option key={one} value={one}>
              .{one}
            </option>
          ))}
        </select>
      </div>
      {stem && !hookNameFits(stem) && (
        <p className="mt-1 text-[12.5px] text-urgent">{t("agentHookNameBad")}</p>
      )}
      <label htmlFor={`${ids}-when`} className={label}>
        {t("agentHookWhen")}
      </label>
      <select
        id={`${ids}-when`}
        value={event}
        onChange={(e) => setEvent(e.target.value as HookEvent)}
        className={field}
      >
        {HOOK_EVENTS.map((one) => (
          <option key={one} value={one}>
            {t(EVENT_WORD[one])}
          </option>
        ))}
      </select>
      {onTools(event) && (
        <>
          <label htmlFor={`${ids}-tools`} className={label}>
            {t("agentHookTools")}
          </label>
          <select
            id={`${ids}-tools`}
            value={tools}
            onChange={(e) => setTools(e.target.value as HookTools)}
            className={field}
          >
            {HOOK_TOOLS.map((one) => (
              <option key={one} value={one}>
                {t(TOOLS_WORD[one])}
              </option>
            ))}
          </select>
        </>
      )}
      <label htmlFor={`${ids}-timeout`} className={label}>
        {t("agentHookTimeout")}
      </label>
      <input
        id={`${ids}-timeout`}
        type="number"
        min={1}
        max={TIMEOUT_AT_MOST}
        value={timeout}
        onChange={(e) => setTimeoutSeconds(Number(e.target.value))}
        className={field}
      />
      <fieldset className="mt-3">
        <legend className="text-[12.5px] font-medium">{t("agentHookAgents")}</legend>
        {HOOK_AGENTS.map(([id, name]) => (
          <label key={id} className="mt-1 flex items-center gap-2 text-[13px]">
            <input type="checkbox" checked={agents.includes(id)} onChange={() => flip(id)} />
            {name}
          </label>
        ))}
      </fieldset>
      <label className="mt-3 flex items-center gap-2 text-[13px]">
        <input type="checkbox" checked={hereOnly} onChange={() => setHereOnly(!hereOnly)} />
        {t("agentHookHereOnly")}
      </label>
      {trouble && <p className="mt-2 text-[12.5px] text-urgent">{trouble}</p>}
      <div className="mt-4 flex justify-end gap-2">
        <button
          type="button"
          onClick={onClose}
          className="rounded-md border border-line px-2.5 py-1 text-[12.5px] text-soft"
        >
          {t("agentHookNotNow")}
        </button>
        <button
          type="button"
          disabled={!fits}
          onClick={make}
          className="rounded-md bg-accent px-2.5 py-1 text-[12.5px] text-bg disabled:bg-hair disabled:text-soft"
        >
          {t("agentHookMake")}
        </button>
      </div>
    </Modal>
  );
}
