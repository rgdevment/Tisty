import { useState } from "react";
import { t } from "../locales";
import Icons from "./Icons";
import Modal from "./Modal";

interface Props {
  title: string;
  invite: string;
  called?: string;
  drawn?: string;
  action?: string;
  onName: (name: string, icon?: string) => void;
  onClose: () => void;
}

export default function Naming({ title, invite, called, drawn, action, onName, onClose }: Props) {
  const [name, setName] = useState(called ?? "");
  const [icon, setIcon] = useState<string | undefined>(drawn);

  const done = () => {
    const wanted = name.trim();
    if (wanted) onName(wanted, icon);
  };

  return (
    <Modal title={title} onClose={onClose}>
      <input
        autoFocus
        onFocus={(e) => e.target.select()}
        value={name}
        onChange={(e) => setName(e.target.value)}
        onKeyDown={(e) => e.key === "Enter" && done()}
        placeholder={invite}
        aria-label={invite}
        className="w-full rounded-lg bg-hover px-3 py-2 text-[13.5px] outline-none placeholder:text-faint"
      />
      <div className="scroller mt-2.5 max-h-52">
        <Icons chosen={icon} onPick={setIcon} />
      </div>
      <div className="mt-3 flex gap-2">
        <button
          type="button"
          onClick={done}
          className="rounded-lg bg-accent px-3 py-1.5 text-[12.5px] text-bg"
        >
          {action ?? t("create")}
        </button>
        <button
          type="button"
          onClick={onClose}
          className="rounded-lg px-3 py-1.5 text-[12.5px] text-soft hover:bg-hover"
        >
          {t("cancel")}
        </button>
      </div>
    </Modal>
  );
}
