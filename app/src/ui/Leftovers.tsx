import {
  checked,
  docAdopt,
  docDrop,
  docLetGo,
  type Gone,
  type Repeated,
  type Reviewed,
  rebuild,
  type Stray,
  type Unvouching,
} from "../core";
import { weigh } from "../format";
import { fill, t } from "../locales";
import { saidPlainly } from "../refusal";
import { sure } from "../sure";
import Card, { type Run, type Which, type Word } from "./Card";
import { astrayKey, hushedName, named } from "./Keys";
import { Band, dated, mild, risky, strong } from "./Rows";
import Tidying from "./Tidying";

interface Props {
  audit: Reviewed;
  twice: Repeated[];
  unproven: Unvouching | null;
  busy: Which | null;
  secs: number;
  said?: Word;
  trouble?: Word;
  run: Run;
  tell: (word?: Word) => void;
  fail: (word?: Word) => void;
  setAudit: (audit: Reviewed) => void;
  relook: () => void;
  onSync: () => void;
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
  secs,
  said,
  trouble,
  run,
  tell,
  fail,
  setAudit,
  relook,
  onSync,
}: Props) {
  const held = busy !== null;
  const shown = { busy, said, trouble };
  const found = lookAt(audit, twice, unproven);
  const spoke = (card: Which) => said?.card === card || trouble?.card === card;
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
    if (held) return;
    sure(fill("upkeepDropSure", one.title || one.file))
      .then(
        (sure) =>
          sure &&
          run("recover", docLetGo(one.file).then(checked), (now) => {
            setAudit(now);
            tell({ card: "recover", text: t("upkeepDropped") });
          }),
      )
      .catch((e) => fail({ card: "recover", text: saidPlainly(e) }));
  };

  const forgetMissing = (one: Gone) => {
    if (held) return;
    sure(fill("dropDocSure", one.title || one.file))
      .then(
        (sure) =>
          sure &&
          run("missing", docDrop(one.id).then(checked), (now) => {
            setAudit(now);
            tell({ card: "missing", text: t("upkeepForgotten") });
          }),
      )
      .catch((e) => fail({ card: "missing", text: saidPlainly(e) }));
  };

  return (
    <>
      <Band label={t("toLookAt")} />

      {open("rebuild") && (
        <Card title={t("cacheOffTitle")} which="rebuild" {...shown}>
          <p className="text-[12.5px] leading-relaxed text-soft">{t("cacheOffWhat")}</p>
          {found.rebuild && (
            <div className="mt-2.5">
              <button
                type="button"
                disabled={held}
                onClick={() =>
                  run("rebuild", rebuild().then(checked), (now) => {
                    setAudit(now);
                    tell({ card: "rebuild", text: t("reviewRebuilt") });
                  })
                }
                className={strong}
              >
                {t("reviewRedo")}
              </button>
            </div>
          )}
        </Card>
      )}

      {open("recover") && (
        <Card title={t("upkeepLook")} which="recover" {...shown}>
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
                        disabled={held}
                        onClick={() => takeIn(one.file)}
                        className={mild}
                      >
                        {t("upkeepTakeIn")}
                      </button>
                      <button
                        type="button"
                        disabled={held}
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
                  disabled={held}
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
        <Card title={t("missingTitle")} which="missing" {...shown}>
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
                        disabled={held}
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
        secs={secs}
        said={said}
        trouble={trouble}
        run={run}
        tell={tell}
        relook={relook}
      />

      {open("unread") && (
        <Card title={t("unreadTitle")} which="unread" {...shown}>
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
        <Card title={t("keysTitle")} which="keys" {...shown}>
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
            <button type="button" onClick={onSync} className={mild}>
              {t("keysSee")}
            </button>
          </div>
        </Card>
      )}
    </>
  );
}
