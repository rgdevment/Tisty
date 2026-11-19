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

const pickSaid = (assistants: AssistantSeen[], found: boolean, more: boolean): string => {
  if (more) return t(found ? "agentsMoreLead" : "agentsMoreNone");
  if (found) return t("agentsFirstPick");
  if (assistants.some((one) => one.files.some((file) => file.held))) return t("agentsFirstAllHere");
  return t("agentsFirstNone");
};

const bringSaid = (busy: boolean, chosen: number, more: boolean): string => {
  if (busy) return t("agentsBringing");
  if (more) return t("agentsBring");
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
  const [trouble, setTrouble] = useState<string>();
  const [brought, setBrought] = useState<Brought>();
  const [waiting, setWaiting] = useState<Waiting[]>([]);
  const [reviewing, setReviewing] = useState(false);

  useEffect(() => {
    agentsLook()
      .then((seen) => {
        setLook(seen);
        const picking = more || !seen.consented;
        const queue = waitingOf(seen);
        if (!picking && queue.length > 0) {
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

  const toggle = (at: string) => {
    const next = new Set(chosen);
    if (next.has(at)) next.delete(at);
    else next.add(at);
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
    agentsBring([at]).then((done) => {
      if (done.refused.length > 0) throw reasonSaid(done.refused[0]);
      return done;
    });

  const bring = () => {
    setBusy(true);
    setTrouble(undefined);
    agentsBring([...chosen])
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
  const adding = more && !!look?.consented;
  return (
    <Modal title={t(adding ? "agentsMoreTitle" : "agentsFirstTitle")} wide onClose={onClose}>
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
      <div className="mt-5 flex justify-end gap-2 text-[12.5px]">
        <button type="button" onClick={onClose} className={button}>
          {t("agentsNotNow")}
        </button>
        <button
          type="button"
          disabled={!look || busy || (adding && chosen.size === 0)}
          onClick={bring}
          className={primary}
        >
          {bringSaid(busy, chosen.size, adding)}
        </button>
      </div>
    </Modal>
  );
}
