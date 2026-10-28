import { useId, useState } from "react";
import { useAttended } from "../attended";
import { t } from "../locales";
import { nameOf, STARTS, type Start } from "./drawing/starting";
import Modal from "./Modal";

/** Names a new diagram and chooses its kind; the name follows the kind until the person types one. */
export default function DiagramMaking({
  onMake,
  onClose,
}: Readonly<{ onMake: (name: string, kind: Start) => void; onClose: () => void }>) {
  const [kind, setKind] = useState<Start>("flowchart");
  const [name, setName] = useState(nameOf("flowchart"));
  const [typed, setTyped] = useState(false);
  const held = useAttended<HTMLInputElement>();
  const group = useId();

  const done = () => {
    const wanted = name.trim();
    if (wanted) onMake(wanted, kind);
  };

  return (
    <Modal title={t("newDiagram")} onClose={onClose}>
      <input
        ref={held}
        onFocus={(e) => e.target.select()}
        value={name}
        onChange={(e) => {
          setName(e.target.value);
          setTyped(true);
        }}
        onKeyDown={(e) => e.key === "Enter" && done()}
        placeholder={t("diagramName")}
        aria-label={t("diagramName")}
        className="w-full rounded-[10px] bg-hover px-3 py-2 text-[13px] outline-none placeholder:text-faint"
      />
      <fieldset className="mt-2.5 flex flex-col gap-0.5 border-0 p-0">
        <legend className="mb-1 text-[11.5px] tracking-[0.04em] text-faint uppercase">
          {t("diagramKindAsk")}
        </legend>
        {STARTS.map((one) => (
          <label
            key={one}
            className="flex cursor-pointer items-center gap-2 rounded-md px-2 py-1 text-[13px] hover:bg-hover"
          >
            <input
              type="radio"
              name={group}
              checked={kind === one}
              onChange={() => {
                setKind(one);
                if (!typed) setName(nameOf(one));
              }}
            />
            {nameOf(one)}
          </label>
        ))}
      </fieldset>
      <div className="mt-3 flex gap-2">
        <button
          type="button"
          onClick={done}
          className="rounded-[10px] bg-accent px-3 py-1.5 text-[12.5px] text-bg"
        >
          {t("create")}
        </button>
        <button
          type="button"
          onClick={onClose}
          className="rounded-[10px] px-3 py-1.5 text-[12.5px] text-soft hover:bg-hover"
        >
          {t("cancel")}
        </button>
      </div>
    </Modal>
  );
}
