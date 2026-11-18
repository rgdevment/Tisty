import { useEffect, useState } from "react";
import {
  type AgentsLook,
  type AssistantSeen,
  agentsBring,
  agentsKeep,
  agentsLook,
  type Both,
  type Brought,
  type FileSeen,
  type Refused,
} from "../agentsSync";
import { fill, t, type Word } from "../locales";
import { saidPlainly } from "../refusal";
import Modal from "./Modal";

const button = "rounded-[10px] px-3 py-1.5 text-faint hover:text-ink";
const primary = "rounded-[10px] bg-accent px-3.5 py-1.5 font-semibold text-bg disabled:opacity-60";

const takeable = (one: FileSeen) => !one.skip && !one.held;

const skipWord = (skip: string) => `agentsSkip_${skip}` as Word;
const kindWord = (kind: FileSeen["kind"]) => `agentsKind_${kind}` as Word;

// An agent's memory travels with sync once it is in Tisty, so the person ticks it by hand.
const chosenFirst = (one: FileSeen) => takeable(one) && one.kind !== "memory";

const SAID = new Set(["noConsent", "secret", "tooBig", "notText", "linked", "name"]);

const reasonSaid = (one: Refused): string => {
  if (one.reason.startsWith("skip_")) return t(skipWord(one.reason.slice("skip_".length)));
  if (SAID.has(one.reason)) return t(`agentsRefused_${one.reason}` as Word);
  return fill("agentsRefused_other", one.why);
};

const listed = (names: string[]) => names.join(", ");

const counted = (one: AssistantSeen, count: number): string => {
  if (count === 0 && one.files.some((file) => file.held))
    return fill("agentsFirstAllHeld", one.name);
  if (count === 1) return fill("agentsFirstCountOne", one.name);
  return fill("agentsFirstCount", one.name, String(count));
};

const fileSaid = (file: FileSeen): string => {
  if (file.held) return t("agentsFirstHeld");
  if (file.skip) return t(skipWord(file.skip));
  return t(kindWord(file.kind));
};

const broughtSaid = (made: number): string => {
  if (made === 0) return t("agentsBroughtNone");
  if (made === 1) return t("agentsBroughtOne");
  return fill("agentsBroughtMany", String(made));
};

const pickSaid = (assistants: AssistantSeen[], found: boolean): string => {
  if (found) return t("agentsFirstPick");
  if (assistants.some((one) => one.files.some((file) => file.held))) return t("agentsFirstAllHere");
  return t("agentsFirstNone");
};

const bringSaid = (busy: boolean, chosen: number): string => {
  if (busy) return t("agentsBringing");
  return chosen > 0 ? t("agentsBringSync") : t("agentsAllow");
};

function Assistant({
  one,
  chosen,
  onToggle,
}: Readonly<{ one: AssistantSeen; chosen: Set<string>; onToggle: (at: string) => void }>) {
  const [open, setOpen] = useState(false);
  const count = one.files.filter(takeable).length;
  return (
    <li className="border-t border-hair first:border-t-0">
      <button
        type="button"
        aria-expanded={open}
        onClick={() => setOpen(!open)}
        className="flex w-full items-center gap-2 px-3 py-2 text-left text-[13px] font-medium hover:bg-hover"
      >
        <span className="flex-1">{counted(one, count)}</span>
        <span className="text-faint">{open ? "▾" : "▸"}</span>
      </button>
      {open && (
        <ul className="pb-1.5">
          {one.files.map((file) => (
            <li key={file.at}>
              <label
                title={file.at}
                className={`flex items-start gap-2.5 px-3 py-1 text-[12.5px] ${takeable(file) ? "cursor-pointer hover:bg-hover" : "opacity-60"}`}
              >
                <input
                  type="checkbox"
                  aria-label={`${file.name} · ${fileSaid(file)}`}
                  checked={chosen.has(file.at)}
                  disabled={!takeable(file)}
                  onChange={() => onToggle(file.at)}
                  className="mt-0.5 accent-accent"
                />
                <span className="min-w-0 flex-1">
                  <span className="block truncate font-mono text-[11.5px] text-ink">
                    {file.name}
                  </span>
                  <span className="block text-[11.5px] text-soft">{fileSaid(file)}</span>
                </span>
              </label>
            </li>
          ))}
        </ul>
      )}
    </li>
  );
}

function Compared({ both }: Readonly<{ both: Both }>) {
  const mark = { same: " ", tisty: "−", file: "+" } as const;
  const tone = { same: "text-soft", tisty: "text-hue-red", file: "text-hue-green" } as const;
  return (
    <div className="mt-3 rounded-[10px] border border-hair">
      <div className="flex gap-4 border-b border-hair px-3 py-1.5 text-[11.5px] text-faint">
        <span className="text-hue-red">− {t("agentsInTisty")}</span>
        <span className="text-hue-green">+ {t("agentsInFile")}</span>
      </div>
      <pre className="max-h-64 overflow-auto px-3 py-2 font-mono text-[11.5px] leading-relaxed whitespace-pre-wrap [overflow-wrap:anywhere]">
        {both.lines.map((line, n) => (
          // biome-ignore lint/suspicious/noArrayIndexKey: a compared line is told once and never moves
          <span key={`${n}-${line.side}`} className={`block ${tone[line.side]}`}>
            {mark[line.side]} {line.text}
          </span>
        ))}
      </pre>
    </div>
  );
}

interface Props {
  onClose: () => void;
  onChanged: () => void;
}

export default function AgentsFirst({ onClose, onChanged }: Readonly<Props>) {
  const [look, setLook] = useState<AgentsLook>();
  const [chosen, setChosen] = useState<Set<string>>(new Set());
  const [busy, setBusy] = useState(false);
  const [trouble, setTrouble] = useState<string>();
  const [brought, setBrought] = useState<Brought>();
  const [both, setBoth] = useState<Both[]>([]);
  const [reviewing, setReviewing] = useState(false);

  useEffect(() => {
    agentsLook()
      .then((seen) => {
        setLook(seen);
        if (seen.consented && seen.docs > 0 && seen.both.length > 0) {
          setBoth(seen.both);
          setReviewing(true);
        }
        setChosen(
          new Set(
            seen.assistants.flatMap((one) => one.files.filter(chosenFirst).map((file) => file.at)),
          ),
        );
      })
      .catch((e) => setTrouble(saidPlainly(e)));
  }, []);

  const toggle = (at: string) => {
    const next = new Set(chosen);
    if (next.has(at)) next.delete(at);
    else next.add(at);
    setChosen(next);
  };

  const bring = () => {
    setBusy(true);
    setTrouble(undefined);
    agentsBring([...chosen])
      .then((done) => {
        setBrought(done);
        setBoth(done.round.both);
        onChanged();
      })
      .catch((e) => setTrouble(saidPlainly(e)))
      .finally(() => setBusy(false));
  };

  const keep = (one: Both, tisty: boolean) => {
    setBusy(true);
    setTrouble(undefined);
    agentsKeep(one.at, tisty)
      .then(() => {
        setBoth(both.filter((other) => other.at !== one.at));
        onChanged();
      })
      .catch((e) => setTrouble(saidPlainly(e)))
      .finally(() => setBusy(false));
  };

  const shown = both[0];
  useEffect(() => {
    if (reviewing && !shown) onClose();
  }, [reviewing, shown, onClose]);
  if (reviewing && !shown) return null;
  if (shown) {
    const name =
      look?.assistants.find((one) => one.id === shown.assistant)?.name ?? shown.assistant;
    return (
      <Modal title={t("agentsBothTitle")} wide>
        <p className="text-[13px] leading-relaxed text-soft">{fill("agentsBothLead", name)}</p>
        <p className="mt-1.5 font-mono text-[11.5px] break-all text-faint">{shown.at}</p>
        <Compared both={shown} />
        {shown.composed && (
          <p className="mt-2.5 text-[12.5px] text-soft">{t("agentsBothComposed")}</p>
        )}
        {trouble && <p className="mt-2.5 text-[11.5px] text-urgent">{trouble}</p>}
        <div className="mt-5 flex flex-wrap justify-end gap-2 text-[12.5px]">
          <button
            type="button"
            disabled={busy}
            onClick={() => setBoth(both.slice(1))}
            className={button}
          >
            {t("agentsLater")}
          </button>
          {!shown.composed && (
            <button
              type="button"
              disabled={busy}
              onClick={() => keep(shown, false)}
              className={button}
            >
              {t("agentsKeepFile")}
            </button>
          )}
          <button
            type="button"
            disabled={busy}
            onClick={() => keep(shown, true)}
            className={primary}
          >
            {t("agentsKeepTisty")}
          </button>
        </div>
      </Modal>
    );
  }

  if (brought) {
    return (
      <Modal title={t("agentsFirstTitle")} wide onClose={onClose}>
        <p className="text-[13px] leading-relaxed text-soft">{broughtSaid(brought.made)}</p>
        {brought.refused.length > 0 && (
          <>
            <p className="mt-3 text-[12.5px] text-soft">{t("agentsRefused")}</p>
            <ul className="mt-1.5 text-[11.5px]">
              {brought.refused.map((one) => (
                <li key={one.at} className="py-0.5">
                  <span className="font-mono break-all text-ink">{one.at}</span>
                  <span className="block text-faint">{reasonSaid(one)}</span>
                </li>
              ))}
            </ul>
          </>
        )}
        <div className="mt-5 flex justify-end text-[12.5px]">
          <button type="button" onClick={onClose} className={primary}>
            {t("agentsDone")}
          </button>
        </div>
      </Modal>
    );
  }

  const assistants = look?.assistants ?? [];
  const found = assistants.some((one) => one.files.some(takeable));
  return (
    <Modal title={t("agentsFirstTitle")} wide onClose={onClose}>
      <p className="text-[13px] leading-relaxed text-soft">{t("agentsFirstLead")}</p>
      <ul className="mt-3 flex flex-col gap-1.5 text-[12.5px] text-ink">
        <li>🔒 {t("agentsFirstPrivate")}</li>
        {assistants.length > 0 && (
          <li>↻ {fill("agentsFirstSyncs", listed(assistants.map((one) => one.name)))}</li>
        )}
        <li>⧉ {t("agentsFirstBackup")}</li>
      </ul>
      {look && (
        <>
          <p className="mt-4 mb-1.5 text-[12.5px] font-semibold">{pickSaid(assistants, found)}</p>
          {assistants.some((one) => one.files.length > 0) && (
            <ul className="rounded-[10px] border border-hair bg-panel">
              {assistants
                .filter((one) => one.files.length > 0)
                .map((one) => (
                  <Assistant key={one.id} one={one} chosen={chosen} onToggle={toggle} />
                ))}
            </ul>
          )}
        </>
      )}
      {trouble && <p className="mt-2.5 text-[11.5px] text-urgent">{trouble}</p>}
      <div className="mt-5 flex justify-end gap-2 text-[12.5px]">
        <button type="button" onClick={onClose} className={button}>
          {t("agentsNotNow")}
        </button>
        <button type="button" disabled={!look || busy} onClick={bring} className={primary}>
          {bringSaid(busy, chosen.size)}
        </button>
      </div>
    </Modal>
  );
}
