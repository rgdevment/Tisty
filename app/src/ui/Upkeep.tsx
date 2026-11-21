import { useEffect, useRef, useState } from "react";
import type { Checked } from "../attention";
import {
  checked,
  type Repeated,
  type Reviewed,
  repeatedLists,
  type Unvouching,
  unvouchedAttachments,
} from "../core";
import { stamped } from "../format";
import { fill, t } from "../locales";
import { saidPlainly } from "../refusal";
import { settingAt } from "../settingsRows";
import { type Run, type Word, working } from "./Card";
import { waiting } from "./Keys";
import Leftovers, { lookAt } from "./Leftovers";
import { ReportOpen } from "./Reporting";
import { Feedback, Heeded, Note, useRows } from "./RowNotes";
import { Band, Line, mild } from "./Rows";
import Tools from "./Tools";
import { useSince } from "./useSince";
import { held as isHeld, type Working } from "./working";

interface Props {
  audit: Reviewed | null;
  setAudit: (audit: Reviewed) => void;
  busy: Working;
  run: Run;
  quietly: Run;
  tell: (word?: Word) => void;
  fail: (word?: Word) => void;
  onSecurity: () => void;
  seen: Seen | null;
  setSeen: (seen: Seen) => void;
}

export interface Seen {
  twice: Repeated[];
  unproven: Unvouching | null;
  at: string;
  missed?: string;
}

export const waitingIn = (audit: Reviewed): string[] =>
  audit.machines.filter(waiting).map((one) => one.name ?? one.called);

export const checkedOf = (audit: Reviewed, seen: Seen): Checked => ({
  many: Object.values(lookAt(audit, seen.twice, seen.unproven)).filter(Boolean).length,
  at: seen.at,
  missed: seen.missed,
  waiting: waitingIn(audit),
});

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
  busy,
  run,
  quietly,
  tell,
  fail,
  onSecurity,
  seen,
  setSeen,
}: Props) {
  const held = busy.size > 0;
  const secs = useSince(busy.has("review"), busy.get("review"));
  const { trouble } = useRows();
  const [reporting, setReporting] = useState(false);
  const twice = seen?.twice ?? [];
  const unproven = seen?.unproven ?? null;
  const at = seen?.at ?? null;

  const took = ([now, lists, unvouched, missed]: Found) => {
    setAudit(now);
    setSeen({ twice: lists ?? [], unproven: unvouched, at: new Date().toISOString(), missed });
  };

  const review = () => quietly("review", looked, took);
  const relook = () => {
    looked()
      .then(took)
      .catch((e) => fail({ card: "review", text: saidPlainly(e) }));
  };

  // Looked at once per visit to the settings, after any card still working has landed.
  const asked = useRef(seen !== null);
  const first = useRef(review);
  first.current = review;
  useEffect(() => {
    if (held || asked.current) return;
    asked.current = true;
    first.current();
  }, [held]);

  const many = audit && seen ? checkedOf(audit, seen).many : -1;
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
  const shown = { busy, run, tell, fail };

  return (
    <>
      <section
        {...settingAt("review")}
        className="flex flex-wrap items-center gap-2.5 rounded-[10px] border border-hair px-4 py-3"
      >
        <span aria-hidden="true" className={`h-2 w-2 shrink-0 rounded-full ${dot}`} />
        <span role="status" className={`min-w-0 flex-1 text-[12.5px] font-medium ${ink}`}>
          {health}
        </span>
        <button type="button" disabled={isHeld(busy, "review")} onClick={review} className={mild}>
          {busy.has("review") ? working("reviewing", secs) : t("reviewAgain")}
        </button>
        <span className="w-full">
          <Heeded row="review" />
          {missed && !trouble.review && <Note kind="warn">{missed}</Note>}
          <Feedback which="review" quiet hideWorking />
        </span>
      </section>

      {audit && at && (
        <Leftovers
          audit={audit}
          twice={twice}
          unproven={unproven}
          {...shown}
          setAudit={setAudit}
          relook={relook}
          onSecurity={onSecurity}
        />
      )}

      <Tools audit={audit} setAudit={setAudit} {...shown} />

      <Band label={t("bandTrouble")} />
      <div className="border-t border-hair">
        <Line row="report" why={t("reportLine")} which="report" hushed={reporting}>
          <button type="button" onClick={() => setReporting(true)} className={mild}>
            {t("reportPrepare")}
          </button>
        </Line>
      </div>

      {reporting && (
        <ReportOpen
          busy={busy}
          tell={tell}
          fail={fail}
          quietly={quietly}
          onClose={() => setReporting(false)}
        />
      )}
    </>
  );
}
