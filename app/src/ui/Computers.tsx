import { useEffect, useRef, useState } from "react";
import {
  type Carrying,
  checked,
  confirmMachineKey,
  type Machine,
  type Reviewed,
  removeMachine,
} from "../core";
import { synced } from "../footing";
import { fill, t } from "../locales";
import { saidPlainly } from "../refusal";
import { settingAt } from "../settingsRows";
import type { Run, Word } from "./Card";
import { useCareful } from "./Careful";
import { DANGERS } from "./dangers";
import Here from "./Here";
import { Asked, MachineList } from "./Keys";
import { ConfirmHere } from "./Knocking";
import { Feedback, Heeded } from "./RowNotes";
import { Band, dated } from "./Rows";
import { held as isHeld, type Working } from "./working";

interface Props {
  state: Carrying;
  audit: Reviewed | null;
  setAudit: (audit: Reviewed) => void;
  busy: Working;
  run: Run;
  tell: (word?: Word) => void;
  fail: (word?: Word) => void;
  ours?: ReadonlySet<string>;
}

export default function Computers({ state, audit, setAudit, busy, run, tell, fail, ours }: Props) {
  const held = isHeld(busy, "machines");
  const careful = useCareful();
  const [keyOf, setKeyOf] = useState<Machine | null>(null);
  const [astray, setAstray] = useState<Machine | null>(null);
  const shared = synced(state);

  // Read once a visit, after any card still working has landed, and not when the settings already hold it.
  const asked = useRef(audit !== null);
  useEffect(() => {
    if (held || asked.current) return;
    asked.current = true;
    checked()
      .then(setAudit)
      .catch((e) => fail({ card: "machines", text: saidPlainly(e) }));
  }, [held, setAudit, fail]);

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
    const last = fill("machineDropWhen", one.when === 0 ? t("machineNever") : dated(one.when));
    careful
      .ask(DANGERS.drop(one.called, last))
      .then(
        (yes) =>
          yes &&
          run("machines", removeMachine(one.id).then(checked), (now) => {
            setAudit(now);
            tell({ card: "machines", text: t("machineDropped") });
          }),
      )
      .catch((e) => fail({ card: "machines", text: saidPlainly(e) }));
  };

  return (
    <>
      {careful.shown}
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
      <div {...settingAt("machines")}>
        <Band label={t("theMachines")} />
        <section className="rounded-[10px] border border-hair px-4 py-3.5">
          <p className="text-[12.5px] leading-relaxed text-soft">{t("machinesWhat")}</p>
          {shared && <ConfirmHere />}
          <MachineList
            all={audit?.machines ?? null}
            ours={ours}
            busy={held}
            onKey={setKeyOf}
            onAstray={setAstray}
            onDrop={dropMachine}
          />
          {shared && <Here held={held} />}
          <Heeded row="machines" />
          <Feedback which="machines" quiet />
        </section>
      </div>
    </>
  );
}
