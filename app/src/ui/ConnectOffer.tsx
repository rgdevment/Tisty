import { useState } from "react";
import { createPortal } from "react-dom";
import { SKILLED } from "../agentsOffered";
import type { Wired } from "../core";
import { fill, t } from "../locales";
import Hooking from "./Hooking";
import { TistySkillsCard, useTistySkills } from "./TistySkills";

const small =
  "shrink-0 rounded-md border border-line px-2.5 py-0.5 text-[12.5px] text-soft hover:border-accent hover:text-accent";

function SkillsLine({ one }: Readonly<{ one: Wired }>) {
  const [skills, setSkills] = useTistySkills();
  const [open, setOpen] = useState(false);
  const agent = SKILLED[one.id];
  if (!agent || !skills || skills.skills.length === 0) return null;
  let said = fill("connectOfferSkills", one.name);
  if (skills.plugged.includes(agent)) said = fill("connectOfferSkillsPlugged", one.name);
  else if (skills.on) said = t("connectOfferSkillsOn");
  const settled = skills.plugged.includes(agent) || skills.on;
  return (
    <div className="mt-2 flex items-center gap-3">
      <span className="min-w-0 flex-1 text-[12.5px] text-soft">{said}</span>
      {!settled && (
        <button type="button" onClick={() => setOpen(true)} className={small}>
          {t("connectOfferSkillsSee")}
        </button>
      )}
      {open &&
        createPortal(
          <TistySkillsCard skills={skills} onTaken={setSkills} onClose={() => setOpen(false)} />,
          document.body,
        )}
    </div>
  );
}

export default function ConnectOffer({
  one,
  held,
  onHook,
  onDone,
}: Readonly<{
  one: Wired;
  held: boolean;
  onHook: (one: Wired, on: boolean) => void;
  onDone: () => void;
}>) {
  const hooks = one.hooks;
  return (
    <section
      aria-label={fill("connectOfferTitle", one.name)}
      className="mt-2 rounded-[10px] border border-accent/40 bg-panel px-3 py-2.5"
    >
      <p className="text-[12.5px] font-semibold">{fill("connectOfferTitle", one.name)}</p>
      {hooks && (
        <p className="mt-1.5 text-[12.5px] text-soft">
          {fill(hooks.plugged && !hooks.on ? "connectOfferPlugged" : "connectOfferHooks", one.name)}
        </p>
      )}
      {hooks && (
        <div className="mt-2 rounded-[10px] border border-hair">
          <Hooking one={one} held={held} onHook={onHook} />
        </div>
      )}
      <SkillsLine one={one} />
      <div className="mt-2.5 flex justify-end">
        <button type="button" onClick={onDone} className={small}>
          {t("agentsNotNow")}
        </button>
      </div>
    </section>
  );
}
