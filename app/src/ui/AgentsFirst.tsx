import { useEffect, useState } from "react";
import {
  type AgentsLook,
  type AssistantSeen,
  agentsBring,
  agentsKeep,
  agentsLead,
  agentsLook,
  agentsLoose,
  type Brought,
  type FileSeen,
  type Refused,
  type Rounded,
} from "../agentsSync";
import { fill, t, type Word } from "../locales";
import { saidPlainly } from "../refusal";
import { skillDirs, skillOf } from "../skillFolders";
import {
  BothReview,
  button,
  LooseReview,
  primary,
  UnledReview,
  type Waiting,
  waitingOf,
} from "./AgentsReview";
import Modal from "./Modal";

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

/// What the list offers: a rule or a memory is one file, a skill is its whole folder.
export interface Offered {
  key: string;
  kind: FileSeen["kind"];
  name: string;
  files: FileSeen[];
}

export const offeredOf = (files: readonly FileSeen[]): Offered[] => {
  const rows: Offered[] = [];
  const skills = new Map<string, Offered>();
  const dirs = skillDirs(files.filter((file) => file.kind === "skill").map((file) => file.name));
  for (const file of files) {
    if (file.kind !== "skill") {
      rows.push({ key: file.at, kind: file.kind, name: file.name, files: [file] });
      continue;
    }
    const name = skillOf(file.name, dirs);
    const known = skills.get(name);
    if (known) {
      known.files.push(file);
      continue;
    }
    const row: Offered = { key: `skill:${name}`, kind: "skill", name, files: [file] };
    skills.set(name, row);
    rows.push(row);
  }
  return rows;
};

const takeableIn = (row: Offered) => row.files.filter(takeable);

const COUNT_WORDS: Record<FileSeen["kind"], [Word, Word]> = {
  rule: ["agentsCountRule", "agentsCountRules"],
  skill: ["agentsCountSkill", "agentsCountSkills"],
  memory: ["agentsCountMemory", "agentsCountMemories"],
};

const counted = (one: AssistantSeen, rows: Offered[]): string => {
  const ready = rows.filter((row) => takeableIn(row).length > 0);
  if (ready.length === 0 && one.files.some((file) => file.held))
    return fill("agentsFirstAllHeld", one.name);
  const parts = (["rule", "skill", "memory"] as const)
    .map((kind) => [kind, ready.filter((row) => row.kind === kind).length] as const)
    .filter(([, many]) => many > 0)
    .map(([kind, many]) =>
      many === 1 ? t(COUNT_WORDS[kind][0]) : fill(COUNT_WORDS[kind][1], String(many)),
    );
  if (parts.length === 0) return fill("agentsFirstNothing", one.name);
  return `${one.name} · ${parts.join(" · ")}`;
};

const filesSaid = (count: number) =>
  count === 1 ? t("agentsSkillFile") : fill("agentsSkillFiles", String(count));

const rowSaid = (row: Offered): string => {
  if (row.files.every((file) => file.held)) return t("agentsFirstHeld");
  const ready = takeableIn(row);
  const skipped = row.files.find((file) => file.skip)?.skip;
  if (ready.length === 0 && skipped) return t(skipWord(skipped));
  if (row.kind !== "skill") return t(kindWord(row.kind));
  return `${t(kindWord("skill"))} · ${filesSaid(ready.length)}`;
};

const broughtSaid = (made: number, syncs: boolean): string => {
  if (made === 0) return t(syncs ? "agentsBroughtNone" : "agentsBroughtNoneAlone");
  if (made === 1) return t(syncs ? "agentsBroughtOne" : "agentsBroughtOneAlone");
  return fill(syncs ? "agentsBroughtMany" : "agentsBroughtManyAlone", String(made));
};

const pickSaid = (assistants: AssistantSeen[], found: boolean, more: boolean): string => {
  if (more) return t(found ? "agentsMoreLead" : "agentsMoreNone");
  if (found) return t("agentsFirstPick");
  if (assistants.some((one) => one.files.some((file) => file.held))) return t("agentsFirstAllHere");
  return t("agentsFirstNone");
};

/// Which button brought, so only that one says it is bringing: null is «Bring another file».
type Going = boolean | null;

function Assistant({
  one,
  chosen,
  onToggle,
}: Readonly<{ one: AssistantSeen; chosen: Set<string>; onToggle: (at: string[]) => void }>) {
  const [open, setOpen] = useState(false);
  const rows = offeredOf(one.files);
  return (
    <li className="border-t border-hair first:border-t-0">
      <button
        type="button"
        aria-expanded={open}
        onClick={() => setOpen(!open)}
        className="flex w-full items-center gap-2 px-3 py-2 text-left text-[13px] font-medium hover:bg-hover"
      >
        <span className="flex-1">{counted(one, rows)}</span>
        <span className="text-faint">{open ? "▾" : "▸"}</span>
      </button>
      {open && (
        <ul className="pb-1.5">
          {rows.map((row) => {
            const ready = takeableIn(row);
            const ats = ready.map((file) => file.at);
            const on = ats.length > 0 && ats.every((at) => chosen.has(at));
            return (
              <li key={row.key}>
                <label
                  title={row.files[0].at}
                  className={`flex items-start gap-2.5 px-3 py-1 text-[12.5px] ${ats.length ? "cursor-pointer hover:bg-hover" : "opacity-60"}`}
                >
                  <input
                    type="checkbox"
                    aria-label={`${row.name} · ${rowSaid(row)}`}
                    checked={on}
                    disabled={ats.length === 0}
                    onChange={() => onToggle(ats)}
                    className="mt-0.5 accent-accent"
                  />
                  <span className="min-w-0 flex-1">
                    <span className="block truncate font-mono text-[11.5px] text-ink">
                      {row.name}
                    </span>
                    <span className="block text-[11.5px] text-soft">{rowSaid(row)}</span>
                  </span>
                </label>
              </li>
            );
          })}
        </ul>
      )}
    </li>
  );
}

interface Props {
  /** Bringing one more file once this computer already said yes, so nothing is ticked for the person. */
  more?: boolean;
  onClose: () => void;
  onChanged: () => void;
}

export default function AgentsFirst({ more = false, onClose, onChanged }: Readonly<Props>) {
  const [look, setLook] = useState<AgentsLook>();
  const [chosen, setChosen] = useState<Set<string>>(new Set());
  const [busy, setBusy] = useState(false);
  const [going, setGoing] = useState<Going>();
  const [trouble, setTrouble] = useState<string>();
  const [brought, setBrought] = useState<Brought>();
  const [waiting, setWaiting] = useState<Waiting[]>([]);
  const [reviewing, setReviewing] = useState(false);

  useEffect(() => {
    agentsLook()
      .then((seen) => {
        setLook(seen);
        // A folder that already syncs has nothing to pick: only what waits for the person, if anything.
        const synced = !more && seen.consented && seen.allOn && !!seen.folder;
        const queue = waitingOf(seen);
        if (synced) {
          setWaiting(queue);
          setReviewing(true);
        }
        const ticked = more && seen.consented ? () => false : chosenFirst;
        setChosen(
          new Set(
            seen.assistants.flatMap((one) => one.files.filter(ticked).map((file) => file.at)),
          ),
        );
      })
      .catch((e) => setTrouble(saidPlainly(e)));
  }, [more]);

  // A skill comes whole: its row ticks or unticks every file it holds.
  const toggle = (ats: string[]) => {
    const next = new Set(chosen);
    const all = ats.every((at) => next.has(at));
    for (const at of ats) {
      if (all) next.delete(at);
      else next.add(at);
    }
    setChosen(next);
  };

  // Each answer settles the one shown; what the round then finds still waiting is asked after it.
  const answered = (step: () => Promise<Rounded | Brought | unknown>) => {
    setBusy(true);
    setTrouble(undefined);
    step()
      .then(() => {
        setWaiting(waiting.slice(1));
        onChanged();
      })
      .catch((e) => setTrouble(typeof e === "string" ? e : saidPlainly(e)))
      .finally(() => setBusy(false));
  };

  // A file the bring turned away stays asked about, with the reason in the person's words.
  const bringOne = (at: string) =>
    agentsBring([at], null).then((done) => {
      if (done.refused.length > 0) throw reasonSaid(done.refused[0]);
      return done;
    });

  const bring = (sync: Going) => {
    setBusy(true);
    setGoing(sync);
    setTrouble(undefined);
    agentsBring([...chosen], sync)
      .then((done) => {
        setBrought(done);
        setWaiting(waitingOf(done.round));
        onChanged();
      })
      .catch((e) => setTrouble(saidPlainly(e)))
      .finally(() => setBusy(false));
  };

  const skip = () => {
    setTrouble(undefined);
    setWaiting(waiting.slice(1));
  };

  const shown = waiting[0];
  useEffect(() => {
    if (reviewing && !shown) onClose();
  }, [reviewing, shown, onClose]);
  if (reviewing && !shown) return null;

  const nameOf = (id: string) => look?.assistants.find((one) => one.id === id)?.name ?? id;
  if (shown?.kind === "both") {
    const at = shown.both.at;
    const bringable = !!look?.assistants.some((one) =>
      one.files.some((file) => file.at === at && takeable(file)),
    );
    return (
      <BothReview
        both={shown.both}
        name={nameOf(shown.both.assistant)}
        busy={busy}
        trouble={trouble}
        bringable={bringable}
        onLater={skip}
        onKeep={(tisty) => answered(() => agentsKeep(at, tisty))}
        onBring={() => answered(() => bringOne(at))}
      />
    );
  }
  if (shown?.kind === "loose") {
    return (
      <LooseReview
        loose={shown.loose}
        name={nameOf(shown.loose.assistant)}
        busy={busy}
        trouble={trouble}
        onAnswer={(away) => answered(() => agentsLoose(shown.loose.at, away))}
      />
    );
  }
  if (shown?.kind === "unled") {
    return (
      <UnledReview
        name={nameOf(shown.assistant)}
        busy={busy}
        trouble={trouble}
        onLater={skip}
        onLead={() => answered(agentsLead)}
      />
    );
  }

  if (brought) {
    return (
      <Modal title={t(more ? "agentsMoreTitle" : "agentsFirstTitle")} wide onClose={onClose}>
        <p className="text-[13px] leading-relaxed text-soft">
          {broughtSaid(brought.made, brought.syncs)}
        </p>
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
  const adding = more && !!look?.consented;
  return (
    <Modal
      title={t(adding ? "agentsMoreTitle" : "agentsFirstTitle")}
      wide
      closable={!adding}
      onClose={onClose}
    >
      {!adding && (
        <>
          <p className="text-[13px] leading-relaxed text-soft">{t("agentsFirstLead")}</p>
          <ul className="mt-3 flex flex-col gap-1.5 text-[12.5px] text-ink">
            <li>🔒 {t("agentsFirstPrivate")}</li>
            {assistants.length > 0 && (
              <li>↻ {fill("agentsFirstSyncs", listed(assistants.map((one) => one.name)))}</li>
            )}
            <li>⧉ {t("agentsFirstBackup")}</li>
          </ul>
        </>
      )}
      {look && (
        <>
          <p className={`${adding ? "" : "mt-4 "}mb-1.5 text-[12.5px] font-semibold`}>
            {pickSaid(assistants, found, adding)}
          </p>
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
      {adding ? (
        <div className="mt-5 flex justify-end gap-2 text-[12.5px]">
          <button type="button" onClick={onClose} className={button}>
            {t("agentsNotNow")}
          </button>
          <button
            type="button"
            disabled={!look || busy || chosen.size === 0}
            onClick={() => bring(null)}
            className={primary}
          >
            {busy ? t("agentsBringing") : t("agentsBring")}
          </button>
        </div>
      ) : (
        // No «Not now»: the X or Esc leaves without doing anything, and the folder asks again later; hiding is on its menu.
        <div className="mt-5 flex items-center justify-end gap-2 text-[12.5px]">
          <button
            type="button"
            disabled={!look || busy || chosen.size === 0}
            onClick={() => bring(false)}
            className={button}
          >
            {busy && going === false ? t("agentsBringing") : t("agentsBring")}
          </button>
          <button
            type="button"
            disabled={!look || busy}
            onClick={() => bring(true)}
            className={primary}
          >
            {busy && going === true ? t("agentsBringing") : t("agentsBringSync")}
          </button>
        </div>
      )}
    </Modal>
  );
}
