import { useState } from "react";
import { t } from "../locales";
import Modal from "./Modal";
import Pick from "./Pick";

interface Props {
  title: string;
  invite: string;
  called?: string;
  drawn?: string;
  painted?: string;
  action?: string;
  onName: (name: string, icon?: string, colour?: string) => void;
  onDrop?: () => void;
  dropWord?: string;
  onClose: () => void;
}

export default function Naming({
  title,
  invite,
  called,
  drawn,
  painted,
  action,
  onName,
  onDrop,
  dropWord,
  onClose,
}: Props) {
  const [name, setName] = useState(called ?? "");
  const [icon, setIcon] = useState<string | undefined>(drawn);
  const [colour, setColour] = useState<string | undefined>(painted);

  const done = () => {
    const wanted = name.trim();
    if (wanted) onName(wanted, icon, colour);
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
      <div className="mt-2.5">
        <Pick icon={icon} colour={colour} onIcon={setIcon} onColour={setColour} />
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
        {onDrop && dropWord && (
          <button
            type="button"
            onClick={onDrop}
            className="ml-auto rounded-lg px-3 py-1.5 text-[12.5px] text-urgent hover:bg-hover"
          >
            {dropWord}
          </button>
        )}
      </div>
    </Modal>
  );
}
