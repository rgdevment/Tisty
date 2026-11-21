import {
  checked,
  docAdopt,
  docDrop,
  docLetGo,
  type Gone,
  type Repeated,
  type Reviewed,
  type Stray,
  type Unvouching,
} from "../core";
import { weigh } from "../format";
import { fill, t } from "../locales";
import { saidPlainly } from "../refusal";
import Card, { type Run, type Which, type Word } from "./Card";
import { useCareful } from "./Careful";
import { DANGERS } from "./dangers";
import { astrayKey, hushedName, named } from "./Keys";
import { useRows } from "./RowNotes";
import { Band, dated, mild, risky } from "./Rows";
import Tidying from "./Tidying";
import { held as isHeld, type Working } from "./working";

interface Props {
  audit: Reviewed;
  twice: Repeated[];
  unproven: Unvouching | null;
  busy: Working;
  run: Run;
  tell: (word?: Word) => void;
  fail: (word?: Word) => void;
  setAudit: (audit: Reviewed) => void;
  relook: () => void;
  onSecurity: () => void;
}

export const lookAt = (audit: Reviewed, twice: Repeated[], unproven: Unvouching | null) => ({
  rebuild: !audit.agrees,
  recover: audit.stranded.length > 0,
  missing: audit.missing.length > 0,
  repeated: twice.length > 0,
  unvouched: (unproven?.found.length ?? 0) > 0,
  unread: audit.unread != null || unproven?.unread != null,
  keys: audit.machines.some(astrayKey),
});

export default function Leftovers({
  audit,
  twice,
  unproven,
  busy,
  run,
  tell,
  fail,
  setAudit,
  relook,
  onSecurity,
}: Props) {
  const { said, trouble } = useRows();
  const careful = useCareful();
  const recovering = isHeld(busy, "recover");
  const forgetting = isHeld(busy, "missing");
  const found = lookAt(audit, twice, unproven);
  const spoke = (card: Which) => Boolean(said[card] || trouble[card]);
  const open = (card: keyof typeof found) => found[card] || spoke(card);
  if (!(Object.keys(found) as (keyof typeof found)[]).some(open)) return null;
  const quiet = hushedName(audit.machines);

  const takeIn = (file: string) =>
    run(
      "recover",
      docAdopt(file).then(async (made) => ({ made, now: await checked() })),
      (both) => {
        setAudit(both.now);
        tell({ card: "recover", text: fill("upkeepTakenIn", both.made.title || both.made.id) });
      },
    );

  const takeInAll = (strays: Stray[]) =>
    run(
      "recover",
      strays
        .reduce((so, one) => so.then(() => docAdopt(one.file).then(() => {})), Promise.resolve())
        .then(checked),
      setAudit,
    );

  const letGoOfPaper = (one: Stray) => {
    if (recovering) return;
    careful
      .ask(DANGERS.stranded(one.title || one.file))
      .then(
        (yes) =>
          yes &&
          run("recover", docLetGo(one.file).then(checked), (now) => {
            setAudit(now);
            tell({ card: "recover", text: t("upkeepDropped") });
          }),
      )
      .catch((e) => fail({ card: "recover", text: saidPlainly(e) }));
  };

  const forgetMissing = (one: Gone) => {
    if (forgetting) return;
    careful
      .ask(DANGERS.missing(one.title || one.file))
      .then(
        (yes) =>
          yes &&
          run("missing", docDrop(one.id).then(checked), (now) => {
            setAudit(now);
            tell({ card: "missing", text: t("upkeepForgotten") });
          }),
      )
      .catch((e) => fail({ card: "missing", text: saidPlainly(e) }));
  };

  return (
    <>
      {careful.shown}
      <Band label={t("toLookAt")} />

      {open("rebuild") && (
        <Card item title={t("cacheOffTitle")} which="rebuild">
          <p className="text-[12.5px] leading-relaxed text-soft">{t("cacheOffWhat")}</p>
        </Card>
      )}

      {open("recover") && (
        <Card item title={t("upkeepLook")} which="recover">
          <p className="text-[12.5px] leading-relaxed text-soft">{t("upkeepLookWhat")}</p>
          {found.recover && (
            <>
              <ul className="scroller mt-2 flex max-h-[22rem] flex-col gap-1 overflow-y-auto text-[12.5px]">
                {audit.stranded.map((one) => (
                  <li key={one.file} className="flex items-baseline justify-between gap-4">
                    <span className="min-w-0">
                      <span className="block truncate">{one.title || t("untitledDoc")}</span>
                      <span className="block font-mono text-[10.5px] text-faint">
                        {`${one.file} · ${weigh(one.bytes)} · ${dated(one.when)}`}
                      </span>
                    </span>
                    <span className="flex shrink-0 items-baseline gap-2.5">
                      <button
                        type="button"
                        disabled={recovering}
                        onClick={() => takeIn(one.file)}
                        className={mild}
                      >
                        {t("upkeepTakeIn")}
                      </button>
                      <button
                        type="button"
                        disabled={recovering}
                        onClick={() => letGoOfPaper(one)}
                        className={risky}
                      >
                        {t("upkeepDropIt")}
                      </button>
                    </span>
                  </li>
                ))}
              </ul>
              <div className="mt-2.5">
                <button
                  type="button"
                  disabled={recovering}
                  onClick={() => takeInAll(audit.stranded)}
                  className={mild}
                >
                  {t("upkeepTakeInAll")}
                </button>
              </div>
            </>
          )}
        </Card>
      )}

      {open("missing") && (
        <Card item title={t("missingTitle")} which="missing">
          <p className="text-[12.5px] leading-relaxed text-soft">{t("upkeepWaitingWhat")}</p>
          {found.missing && (
            <ul className="scroller mt-2 flex max-h-[22rem] flex-col gap-1 overflow-y-auto text-[12.5px]">
              {audit.missing.map((one) => (
                <li key={one.file} className="flex items-baseline justify-between gap-4">
                  <span className="min-w-0">
                    <span className="block truncate">{one.title || t("untitledDoc")}</span>
                    <span className="block font-mono text-[10.5px] text-faint">{one.file}</span>
                  </span>
                  <span className="flex shrink-0 items-baseline gap-2.5">
                    {quiet ? (
                      <span className="text-[11.5px] text-faint">
                        {fill("upkeepForgetWaits", quiet)}
                      </span>
                    ) : (
                      <button
                        type="button"
                        disabled={forgetting}
                        onClick={() => forgetMissing(one)}
                        className={risky}
                      >
                        {t("upkeepForget")}
                      </button>
                    )}
                  </span>
                </li>
              ))}
            </ul>
          )}
        </Card>
      )}

      <Tidying
        twice={twice}
        unproven={unproven}
        open={open}
        busy={busy}
        run={run}
        tell={tell}
        relook={relook}
      />

      {open("unread") && (
        <Card item title={t("unreadTitle")} which="unread">
          {audit.unread != null && (
            <p className="text-[12.5px] leading-relaxed text-soft">
              {fill("looseUnread", audit.unread || t("untitledDoc"))}
            </p>
          )}
          {unproven?.unread != null && (
            <p className="mt-1 text-[12.5px] leading-relaxed text-soft">
              {fill("unvouchedUnread", unproven.unread || t("untitledDoc"))}
            </p>
          )}
        </Card>
      )}

      {open("keys") && (
        <Card item title={t("keysTitle")} which="keys">
          <p className="text-[12.5px] leading-relaxed text-soft">
            {fill(
              "keysWhat",
              audit.machines
                .filter(astrayKey)
                .map((one) => named(one))
                .join(", "),
            )}
          </p>
          <div className="mt-2.5">
            <button type="button" onClick={onSecurity} className={mild}>
              {t("keysSee")}
            </button>
          </div>
        </Card>
      )}
    </>
  );
}
