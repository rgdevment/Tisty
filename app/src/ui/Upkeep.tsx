import { useEffect, useRef, useState } from "react";
import {
  type About,
  type Carrying,
  checked,
  type Freeing,
  type Repeated,
  type Reviewed,
  repeatedLists,
  type Unvouching,
  unvouchedAttachments,
} from "../core";
import { stamped } from "../format";
import { fill, t } from "../locales";
import { saidPlainly } from "../refusal";
import { type Run, type Which, type Word, working } from "./Card";
import FreeUp from "./FreeUp";
import Leftovers, { lookAt } from "./Leftovers";
import Modal from "./Modal";
import Reporting from "./Reporting";
import { Line, mild } from "./Rows";
import Tools from "./Tools";
import { useSince } from "./useSince";

interface Props {
  audit: Reviewed | null;
  setAudit: (audit: Reviewed) => void;
  build: About | null;
  state: Carrying;
  freeing: Freeing | null;
  setFreeing: (freeing: Freeing | null) => void;
  freeDays: number;
  setFreeDays: (days: number) => void;
  busy: Which | null;
  busySince?: number;
  said?: Word;
  trouble?: Word;
  run: Run;
  quietly: Run;
  tell: (word?: Word) => void;
  fail: (word?: Word) => void;
  onSync: () => void;
  seen: Seen | null;
  setSeen: (seen: Seen) => void;
}

export interface Seen {
  twice: Repeated[];
  unproven: Unvouching | null;
  at: string;
  missed?: string;
}

type Found = [Reviewed, Repeated[] | null, Unvouching | null, string | undefined];

// One check that cannot run leaves the others standing, but never lets the whole read as in order.
const looked = async (): Promise<Found> => {
  let missed: string | undefined;
  const or = <T,>(asked: Promise<T>) =>
    asked.catch((e) => {
      missed ??= saidPlainly(e);
      return null;
    });
  const [now, lists, unvouched] = await Promise.all([
    checked(),
    or(repeatedLists()),
    or(unvouchedAttachments()),
  ]);
  return [now, lists, unvouched, missed];
};

export default function Upkeep({
  audit,
  setAudit,
  build,
  state,
  freeing,
  setFreeing,
  freeDays,
  setFreeDays,
  busy,
  busySince,
  said,
  trouble,
  run,
  quietly,
  tell,
  fail,
  onSync,
  seen,
  setSeen,
}: Props) {
  const held = busy !== null;
  const secs = useSince(held, busySince);
  const [reporting, setReporting] = useState(false);
  const twice = seen?.twice ?? [];
  const unproven = seen?.unproven ?? null;
  const at = seen?.at ?? null;

  const took = ([now, lists, unvouched, missed]: Found) => {
    setAudit(now);
    setSeen({ twice: lists ?? [], unproven: unvouched, at: new Date().toISOString(), missed });
  };

  const review = () => quietly("review", looked(), took);
  const relook = () => {
    looked()
      .then(took)
      .catch(() => undefined);
  };

  // Looked at once per visit to the settings, and never over another card that is still working.
  const first = useRef(() => {
    if (!seen && !held) review();
  });
  useEffect(() => first.current(), []);

  const many =
    audit && at ? Object.values(lookAt(audit, twice, unproven)).filter(Boolean).length : -1;
  const missed = seen?.missed;
  const health =
    !audit || !at
      ? t("reviewWhat")
      : missed && many === 0
        ? t("reviewPartly")
        : many === 0
          ? fill("healthy", stamped(at))
          : many === 1
            ? t("healthOne")
            : fill("healthMany", String(many));
  const [dot, ink] =
    many < 0
      ? ["bg-hair", "text-faint"]
      : many === 0 && !missed
        ? ["bg-hue-green", "text-hue-green"]
        : ["bg-hue-amber", "text-hue-amber"];
  const shown = { busy, secs, said, trouble, run, tell, fail };

  return (
    <>
      <section className="flex flex-wrap items-center gap-2.5 rounded-[10px] border border-hair px-4 py-3">
        <span aria-hidden="true" className={`h-2 w-2 shrink-0 rounded-full ${dot}`} />
        <span role="status" className={`min-w-0 flex-1 text-[12.5px] font-medium ${ink}`}>
          {health}
        </span>
        <button type="button" disabled={held} onClick={review} className={mild}>
          {busy === "review" ? working("reviewing", secs) : t("reviewAgain")}
        </button>
      </section>
      {trouble?.card === "review" && (
        <p className="mt-2 text-[11.5px] text-urgent">{trouble.text}</p>
      )}
      {missed && trouble?.card !== "review" && (
        <p className="mt-2 text-[11.5px] text-urgent">{missed}</p>
      )}

      {audit && at && (
        <Leftovers
          audit={audit}
          twice={twice}
          unproven={unproven}
          {...shown}
          setAudit={setAudit}
          relook={relook}
          onSync={onSync}
        />
      )}

      <FreeUp
        audit={audit}
        setAudit={setAudit}
        build={build}
        state={state}
        freeing={freeing}
        setFreeing={setFreeing}
        days={freeDays}
        setDays={setFreeDays}
        {...shown}
      />

      <Tools audit={audit} setAudit={setAudit} {...shown} />

      <div className="mt-5 border-t border-hair">
        <Line
          title={t("reportTitle")}
          why={t("reportLine")}
          which="report"
          said={reporting ? undefined : said}
          trouble={reporting ? undefined : trouble}
        >
          <button type="button" onClick={() => setReporting(true)} className={mild}>
            {t("reportPrepare")}
          </button>
        </Line>
      </div>

      {reporting && (
        <Modal title={t("reportTitle")} wide onClose={() => setReporting(false)}>
          <Reporting
            busy={busy}
            said={said}
            trouble={trouble}
            tell={tell}
            fail={fail}
            quietly={quietly}
          />
          <div className="mt-5 flex justify-end">
            <button type="button" onClick={() => setReporting(false)} className={mild}>
              {t("close")}
            </button>
          </div>
        </Modal>
      )}
    </>
  );
}
