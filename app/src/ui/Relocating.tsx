import { useEffect, useRef, useState } from "react";
import { letGoUnused } from "../cloudState";
import {
  type Carrying,
  cloudForget,
  type MovePlan,
  moveDropped,
  movePlan,
  type Toward,
} from "../core";
import { erase, forgetErasing, useErasing } from "../erasing";
import { filled, said, t } from "../locales";
import { saidPlainly } from "../refusal";
import { dismissMove, endMove, startMove, underway, useMove, watchMove } from "../relocation";
import { sure } from "../sure";
import type { Heading } from "../views";
import Clouding from "./Clouding";
import Keepers from "./Keepers";
import Modal from "./Modal";
import { Done, Pick, Plan, quiet, Underway, Waiting } from "./RelocatingSteps";

interface Props {
  state: Carrying;
  heading?: Heading;
  resume?: Toward;
  onClose: () => void;
}

type Step = "pick" | "folder" | "cloud" | "plan" | "moving" | "waiting" | "done";

const firstStep = (heading?: Heading, resume?: Toward): Step =>
  resume ? "plan" : heading === "cloud" ? "cloud" : heading === "folder" ? "folder" : "pick";

export default function Relocating({ state, heading, resume, onClose }: Props) {
  const now = useMove();
  const [chosen, setStep] = useState<Step>(now ? "plan" : firstStep(heading, resume));
  const [toward, setToward] = useState<Toward | undefined>(now?.toward ?? resume);
  const [plan, setPlan] = useState<MovePlan>();
  const [agreed, setAgreed] = useState(false);
  const [trouble, setTrouble] = useState<string>();
  const erasing = useErasing();
  const wiping = Boolean(erasing?.going);
  const busy = wiping;
  const [deciding, setDeciding] = useState(false);
  const settled = useRef(false);

  useEffect(watchMove, []);

  useEffect(() => {
    if (!now?.failed) return;
    setTrouble(saidPlainly(now.failed));
    setToward(now.toward);
    setStep("plan");
    endMove();
  }, [now]);

  const result = now?.failed ? undefined : now?.result;
  const step: Step =
    now && !now.failed
      ? underway(now)
        ? "moving"
        : result?.state === "done"
          ? "done"
          : "waiting"
      : chosen;

  useEffect(() => {
    if (step !== "plan" || !toward) return;
    setPlan(undefined);
    movePlan(toward)
      .then(setPlan)
      .catch((e) => setTrouble(saidPlainly(e)));
  }, [step, toward]);

  const chose = (next: Toward) => {
    setTrouble(undefined);
    setToward(next);
    setStep("plan");
  };

  const done = result?.state === "done" ? result : undefined;
  const leftCloud = done?.told ? done.leftCloud : undefined;
  const connectedHere = toward?.kind === "cloud" && !resume;

  const from = leftCloud?.provider ?? "google";
  const gone = erasing?.gone;
  const wiped =
    gone === undefined
      ? undefined
      : gone === 1
        ? said("moveWipedOne", from)
        : filled(said("moveWiped", from), String(gone));
  // The erasing lets go of the old cloud itself once its copies are gone.
  if (gone !== undefined) settled.current = true;

  useEffect(() => {
    const failed = erasing?.failed;
    if (failed === undefined) return;
    setTrouble(
      (failed as { code?: string })?.code === "wipeStopped"
        ? said("wipeStopped", from)
        : saidPlainly(failed),
    );
  }, [erasing?.failed, from]);

  const close = () => {
    // The erasing goes on without the dialog, and the notice brings it back to see where it stands.
    if (wiping) {
      onClose();
      return;
    }
    if (now && underway(now)) {
      onClose();
      return;
    }
    if (connectedHere && toward?.kind === "cloud" && !now) letGoUnused(toward);
    dismissMove(!settled.current);
    forgetErasing();
    onClose();
  };

  const move = () => {
    if (!toward || busy) return;
    setTrouble(undefined);
    startMove(toward, state.mode);
  };

  const drop = () => {
    void moveDropped().catch(() => undefined);
    if (toward?.kind === "cloud") letGoUnused(toward);
    endMove();
    onClose();
  };

  const wipe = async () => {
    if (!leftCloud || busy || !(await sure(said("moveWipeSure", from)))) return;
    setTrouble(undefined);
    erase(leftCloud.account, leftCloud.email, leftCloud.provider);
  };

  const keep = () => {
    if (!leftCloud) return;
    settled.current = true;
    void cloudForget(leftCloud.account, leftCloud.provider).catch(() => undefined);
    close();
  };

  return (
    <Modal title={t("moveTitle")} wide onClose={close}>
      {step === "pick" && (
        <Pick
          state={state}
          onAlone={() => chose({ kind: "alone" })}
          onCloud={() => setStep("cloud")}
          onFolder={() => setStep("folder")}
          onCancel={close}
          busy={busy}
        />
      )}

      {step === "folder" && (
        <div className="flex flex-col gap-3">
          <Keepers
            busy={busy}
            pickOnly
            onDeciding={setDeciding}
            onTrouble={(said) => setTrouble(said)}
            onDone={(at) => chose({ kind: "folder", at })}
          />
          {!deciding && (
            <button type="button" onClick={() => setStep("pick")} className={`self-start ${quiet}`}>
              {t("welcomeBack")}
            </button>
          )}
        </div>
      )}

      {step === "cloud" && (
        <Clouding
          clouds={state.clouds}
          busy={busy}
          onTrouble={setTrouble}
          onBack={() => setStep("pick")}
          onDone={() => undefined}
          onConnected={(account) =>
            chose({
              kind: "cloud",
              account: account.account,
              email: account.email,
              provider: account.provider,
            })
          }
        />
      )}

      {step === "plan" && toward && (
        <Plan
          state={state}
          toward={toward}
          plan={plan}
          agreed={agreed}
          busy={busy}
          onAgree={setAgreed}
          onCancel={close}
          onMove={move}
        />
      )}

      {step === "moving" && now && <Underway run={now} onHide={close} />}

      {step === "waiting" && result?.state === "waiting" && (
        <Waiting
          result={result}
          failing={now?.failing}
          toward={toward}
          busy={busy}
          onDrop={drop}
          onLater={close}
          onAgain={move}
        />
      )}

      {step === "done" && toward && (
        <Done
          toward={toward}
          untold={done?.told === false}
          wiped={wiped}
          wiping={wiping}
          erased={erasing?.erased}
          offersWipe={Boolean(leftCloud)}
          wipeFrom={leftCloud?.provider}
          busy={busy}
          onKeep={keep}
          onWipe={wipe}
          onClose={close}
        />
      )}

      {trouble && (
        <p role="alert" className="mt-3 text-[11.5px] text-urgent">
          {trouble}
        </p>
      )}
    </Modal>
  );
}
