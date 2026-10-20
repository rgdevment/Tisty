import { useEffect, useRef, useState } from "react";
import {
  type Carrying,
  checked,
  confirmMachineKey,
  type Machine,
  type Reviewed,
  removeMachine,
} from "../core";
import { fill, t } from "../locales";
import { saidPlainly } from "../refusal";
import { sure } from "../sure";
import type { Run, Which, Word } from "./Card";
import Here from "./Here";
import { Asked, MachineList } from "./Keys";
import { ConfirmHere } from "./Knocking";
import { Band, dated } from "./Rows";

interface Props {
  state: Carrying;
  audit: Reviewed | null;
  setAudit: (audit: Reviewed) => void;
  busy: Which | null;
  said?: Word;
  trouble?: Word;
  run: Run;
  tell: (word?: Word) => void;
  fail: (word?: Word) => void;
}

export default function Computers({
  state,
  audit,
  setAudit,
  busy,
  said,
  trouble,
  run,
  tell,
  fail,
}: Props) {
  const held = busy !== null;
  const [keyOf, setKeyOf] = useState<Machine | null>(null);
  const [astray, setAstray] = useState<Machine | null>(null);
  const synced = Boolean(state.chosen) || state.mode === "cloud";

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
    const said = `${fill("machineDropSure", one.called)}\n\n${fill(
      "machineDropWhen",
      one.when === 0 ? t("machineNever") : dated(one.when),
    )}`;
    sure(said)
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
      <Band label={t("theMachines")} />
      <section className="rounded-[10px] border border-hair px-4 py-3.5">
        <p className="text-[12.5px] leading-relaxed text-soft">{t("machinesWhat")}</p>
        {synced && <ConfirmHere />}
        <MachineList
          all={audit?.machines ?? null}
          busy={held}
          onKey={setKeyOf}
          onAstray={setAstray}
          onDrop={dropMachine}
        />
        {synced && <Here held={held} />}
        {trouble?.card === "machines" && (
          <p className="mt-2 text-[11.5px] text-urgent">{trouble.text}</p>
        )}
        {said?.card === "machines" && <p className="mt-2 text-[11.5px] text-faint">{said.text}</p>}
      </section>
    </>
  );
}
