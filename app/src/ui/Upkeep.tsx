import { ask } from "@tauri-apps/plugin-dialog";
import { useState } from "react";
import {
  type About,
  type Astray,
  checked,
  confirmMachineKey,
  docAdopt,
  docDrop,
  docLetGo,
  type Gone,
  type Machine,
  type Reviewed,
  readTags,
  rebuild,
  removeMachine,
  retireAttachment,
  retireAttachments,
  type Stray,
} from "../core";
import { weigh } from "../format";
import { fill, t } from "../locales";
import { saidPlainly } from "../refusal";
import Card, { type Run, type Which, type Word } from "./Card";
import { Asked, MachineList } from "./Keys";
import Leftovers from "./Leftovers";
import Reporting from "./Reporting";
import { dated, Group, mild, strong } from "./Rows";
import Tidying from "./Tidying";

interface Props {
  audit: Reviewed | null;
  setAudit: (audit: Reviewed) => void;
  build: About | null;
  busy: Which | null;
  said?: Word;
  trouble?: Word;
  run: Run;
  quietly: Run;
  tell: (word?: Word) => void;
  fail: (word?: Word) => void;
}

export default function Upkeep({
  audit,
  setAudit,
  build,
  busy,
  said,
  trouble,
  run,
  quietly,
  tell,
  fail,
}: Props) {
  const held = busy !== null;
  const [keyOf, setKeyOf] = useState<Machine | null>(null);
  const [astray, setAstray] = useState<Machine | null>(null);

  const letGoOfAll = (astray: Astray[]) => {
    if (held || astray.length === 0) return;
    ask(fill("upkeepSafeAllSure", String(astray.length)), { kind: "warning" })
      .then((sure) => {
        if (!sure) return;
        run("review", retireAttachments(astray.map((one) => one.at)).then(checked), (now) => {
          setAudit(now);
          tell({ card: "review", text: t("looseDropped") });
        });
      })
      .catch((e) => fail({ card: "review", text: saidPlainly(e) }));
  };

  const forgetMissing = (one: Gone) => {
    if (held) return;
    ask(fill("dropDocSure", one.title || one.file), { kind: "warning" })
      .then(
        (sure) =>
          sure &&
          run("review", docDrop(one.id).then(checked), (now) => {
            setAudit(now);
            tell({ card: "review", text: t("looseDropped") });
          }),
      )
      .catch((e) => fail({ card: "review", text: saidPlainly(e) }));
  };

  const takeInAll = (strays: Stray[]) =>
    run(
      "review",
      strays
        .reduce((so, one) => so.then(() => docAdopt(one.file).then(() => {})), Promise.resolve())
        .then(checked),
      setAudit,
    );

  const takeIn = (file: string) =>
    run(
      "review",
      docAdopt(file).then(async (made) => ({ made, now: await checked() })),
      (both) => {
        setAudit(both.now);
        tell({ card: "review", text: fill("upkeepTakenIn", both.made.title || both.made.id) });
      },
    );

  const letGoOfPaper = (one: Stray) => {
    if (held) return;
    ask(fill("upkeepDropSure", one.title || one.file), { kind: "warning" })
      .then(
        (sure) =>
          sure &&
          run("review", docLetGo(one.file).then(checked), (now) => {
            setAudit(now);
            tell({ card: "review", text: t("upkeepDropped") });
          }),
      )
      .catch((e) => fail({ card: "review", text: saidPlainly(e) }));
  };

  const letGo = (reference: string, shared?: boolean) => {
    if (held) return;
    const named = reference.split("/").pop() ?? reference;
    ask(fill(shared ? "looseDropSharedSure" : "looseDropSure", named), { kind: "warning" })
      .then(
        (sure) =>
          sure &&
          run("review", retireAttachment(reference).then(checked), (now) => {
            setAudit(now);
            tell({ card: "review", text: t("looseDropped") });
          }),
      )
      .catch((e) => fail({ card: "review", text: saidPlainly(e) }));
  };

  // A refusal says "look again", so the window looks rather than leave a live button on a stale key.
  const confirmKey = (one: Machine) => {
    if (held || !one.signs) return;
    const said = one.signs;
    setKeyOf(null);
    run(
      "machines",
      confirmMachineKey(one.id, said)
        .then(() => null)
        .catch((e) => saidPlainly(e))
        .then((amiss) => checked().then((now) => ({ amiss, now }))),
      ({ amiss, now }) => {
        setAudit(now);
        if (amiss) fail({ card: "machines", text: amiss });
        else tell({ card: "machines", text: t("machineKeyDone") });
      },
    );
  };

  const dropMachine = (one: Machine) => {
    if (held) return;
    const said = `${fill("machineDropSure", one.called)}\n\n${fill(
      "machineDropWhen",
      one.when === 0 ? t("machineNever") : dated(one.when),
    )}`;
    ask(said, { kind: "warning" })
      .then(
        (sure) =>
          sure &&
          run("machines", removeMachine(one.id).then(checked), (now) => {
            setAudit(now);
            tell({ card: "machines", text: t("machineDropped") });
          }),
      )
      .catch((e) => fail({ card: "machines", text: saidPlainly(e) }));
  };

  return (
    <>
      <Asked
        keyOf={keyOf}
        astray={astray}
        busy={held}
        onConfirm={confirmKey}
        onClose={() => {
          setKeyOf(null);
          setAstray(null);
        }}
      />
      <Group label={t("theStore")} />

      <Card title={t("review")} which="review" busy={busy} said={said} trouble={trouble}>
        <p className="text-[12.5px] leading-relaxed text-soft">{t("reviewWhat")}</p>
        {audit && (
          <dl className="mt-2 grid grid-cols-[auto_minmax(0,1fr)] gap-x-4 gap-y-0.5 text-[12.5px]">
            <dt className="text-faint">{t("inTheLog")}</dt>
            <dd className="text-soft">
              {[
                fill("reviewCount", String(audit.tasks)),
                fill("reviewLists", String(audit.lists)),
                `${audit.events} ${t("wordEvents")}`,
              ].join(" · ")}
            </dd>
            <dt className="text-faint">{t("cacheIs")}</dt>
            <dd className={audit.agrees ? "text-accent" : "text-urgent"}>
              {t(audit.agrees ? "cacheAgrees" : "cacheDiverged")}
            </dd>
            <dt className="text-faint">{t("weighsLog")}</dt>
            <dd className="tabular-nums text-soft">{weigh(audit.logBytes)}</dd>
            <dt className="text-faint">{t("weighsDocs")}</dt>
            <dd className="tabular-nums text-soft">{weigh(audit.docsBytes)}</dd>
            <dt className="text-faint">{t("weighsHeld")}</dt>
            <dd className="tabular-nums text-soft">
              {`${audit.heldFiles} · ${weigh(audit.heldBytes)}`}
            </dd>
          </dl>
        )}
        <div className="mt-2.5 flex flex-wrap items-center gap-2.5">
          <button
            type="button"
            disabled={held}
            onClick={() => run("review", checked(), setAudit)}
            className={mild}
          >
            {t(audit ? "reviewAgain" : "reviewRun")}
          </button>
          {audit && !audit.agrees && (
            <button
              type="button"
              disabled={held}
              onClick={() =>
                run("review", rebuild().then(checked), (now) => {
                  setAudit(now);
                  tell({ card: "review", text: t("reviewRebuilt") });
                })
              }
              className={strong}
            >
              {t("reviewRedo")}
            </button>
          )}
        </div>
      </Card>

      <Group label={t("theMachines")} />

      <Card title={t("theMachines")} which="machines" busy={busy} said={said} trouble={trouble}>
        <p className="text-[12.5px] leading-relaxed text-soft">{t("machinesWhat")}</p>
        <MachineList
          all={audit?.machines ?? null}
          busy={held}
          onKey={setKeyOf}
          onAstray={setAstray}
          onDrop={dropMachine}
        />
      </Card>

      <Card title={t("tagsRead")} which="tagging" busy={busy} said={said} trouble={trouble}>
        <p className="text-[12.5px] leading-relaxed text-soft">{t("tagsReadWhy")}</p>
        <div className="mt-2.5 flex items-center gap-2.5">
          <button
            type="button"
            disabled={held}
            onClick={() =>
              run("tagging", readTags(), (many) =>
                tell({ card: "tagging", text: fill("tagsReadDone", String(many)) }),
              )
            }
            className={mild}
          >
            {t("tagsReadDo")}
          </button>
        </div>
      </Card>

      <Tidying busy={busy} said={said} trouble={trouble} run={run} tell={tell} mild={mild} />

      <Leftovers
        audit={audit}
        build={build}
        busy={busy}
        said={said}
        trouble={trouble}
        run={run}
        fail={fail}
        letGo={letGo}
        letGoOfAll={letGoOfAll}
        takeIn={takeIn}
        takeInAll={takeInAll}
        letGoOfPaper={letGoOfPaper}
        forgetMissing={forgetMissing}
      />

      <Reporting
        busy={busy}
        said={said}
        trouble={trouble}
        tell={tell}
        fail={fail}
        quietly={quietly}
      />
    </>
  );
}
