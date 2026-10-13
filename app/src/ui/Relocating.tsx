import { useEffect, useRef, useState } from "react";
import { letGoUnused } from "../cloudState";
import { useComing } from "../coming";
import {
  type Carrying,
  cloudForget,
  cloudWipe,
  type MovePlan,
  moveDropped,
  movePlan,
  type Toward,
} from "../core";
import { fill, t } from "../locales";
import { saidPlainly } from "../refusal";
import { endMove, startMove, underway, useMove, watchMove } from "../relocation";
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
  onMoved: () => void;
}

type Step = "pick" | "folder" | "cloud" | "plan" | "moving" | "waiting" | "done";

const firstStep = (heading?: Heading, resume?: Toward): Step =>
  resume ? "plan" : heading === "cloud" ? "cloud" : heading === "folder" ? "folder" : "pick";

export default function Relocating({ state, heading, resume, onClose, onMoved }: Props) {
  const now = useMove();
  const along = useComing();
  const [chosen, setStep] = useState<Step>(now ? "plan" : firstStep(heading, resume));
  const [toward, setToward] = useState<Toward | undefined>(now?.toward ?? resume);
  const [plan, setPlan] = useState<MovePlan>();
  const [agreed, setAgreed] = useState(false);
  const [trouble, setTrouble] = useState<string>();
  const [wiped, setWiped] = useState<string>();
  const [busy, setBusy] = useState(false);
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

  const close = () => {
    if (busy) return;
    if (now && underway(now)) {
      onClose();
      return;
    }
    if (connectedHere && toward?.kind === "cloud" && !now) letGoUnused(toward.account);
    if (leftCloud && !settled.current) void cloudForget(leftCloud.account).catch(() => undefined);
    if (done) onMoved();
    endMove();
    onClose();
  };

  const move = () => {
    if (!toward || busy) return;
    setTrouble(undefined);
    startMove(toward);
  };

  const drop = () => {
    void moveDropped().catch(() => undefined);
    if (toward?.kind === "cloud") letGoUnused(toward.account);
    endMove();
    onClose();
  };

  const wipe = async () => {
    if (!leftCloud || busy || !(await sure(t("moveWipeSure")))) return;
    setBusy(true);
    cloudWipe(leftCloud.account, leftCloud.email)
      .then((gone) => {
        settled.current = true;
        setWiped(fill("moveWiped", String(gone)));
      })
      .catch((e) => setTrouble(saidPlainly(e)))
      .finally(() => setBusy(false));
  };

  const keep = () => {
    if (!leftCloud) return;
    settled.current = true;
    void cloudForget(leftCloud.account).catch(() => undefined);
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
        />
      )}

      {step === "folder" && (
        <div className="flex flex-col gap-3">
          <Keepers
            busy={busy}
            pickOnly
            onTrouble={(said) => setTrouble(said)}
            onDone={(at) => chose({ kind: "folder", at })}
          />
          <button type="button" onClick={() => setStep("pick")} className={`self-start ${quiet}`}>
            {t("welcomeBack")}
          </button>
        </div>
      )}

      {step === "cloud" && (
        <Clouding
          busy={busy}
          onTrouble={setTrouble}
          onBack={() => setStep("pick")}
          onDone={() => undefined}
          onConnected={(account) =>
            chose({ kind: "cloud", account: account.account, email: account.email })
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

      {step === "moving" && (
        <Underway along={along ?? undefined} pausedUntil={now?.pausedUntil} onHide={close} />
      )}

      {step === "waiting" && result?.state === "waiting" && (
        <Waiting result={result} busy={busy} onDrop={drop} onLater={close} onAgain={move} />
      )}

      {step === "done" && toward && (
        <Done
          toward={toward}
          untold={done?.told === false}
          wiped={wiped}
          offersWipe={Boolean(leftCloud)}
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
