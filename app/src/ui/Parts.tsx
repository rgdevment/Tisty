import { useState } from "react";
import { useAsked } from "../asked";
import { partsOf, type Task, type Whole } from "../core";
import { fill, t } from "../locales";

interface Props {
  task: Task;
  whole?: Whole;
  onAdd: (title: string) => void;
  onOpen: (id: string) => void;
  onComplete: (id: string, title: string) => void;
  onError?: (problem: unknown) => void;
}

export default function Parts({ task, whole, onAdd, onOpen, onComplete, onError }: Props) {
  const parts =
    useAsked(() => partsOf(task.id), [task.id, whole?.open, whole?.closed], onError) ?? [];
  const [closedShown, setClosedShown] = useState(false);
  const [adding, setAdding] = useState("");
  const open = parts.filter((one) => one.status === "open");
  const closed = parts.filter((one) => one.status !== "open");

  return (
    <>
      {open.map((part) => (
        <Row key={part.id} part={part} onOpen={onOpen} onComplete={onComplete} />
      ))}
      {closed.length > 0 && (
        <button
          type="button"
          aria-expanded={closedShown}
          onClick={() => setClosedShown(!closedShown)}
          className="flex w-full items-center gap-2.5 rounded-md py-1 pl-[25px] text-left text-[12.5px] text-faint hover:text-ink"
        >
          {fill("partsClosedShown", String(closed.length))}
          <span aria-hidden="true" className="text-[9px]">
            {closedShown ? "▾" : "▸"}
          </span>
        </button>
      )}
      {closedShown &&
        closed.map((part) => (
          <Row key={part.id} part={part} onOpen={onOpen} onComplete={onComplete} />
        ))}
      <form
        onSubmit={(e) => {
          e.preventDefault();
          if (adding.trim()) {
            onAdd(adding.trim());
            setAdding("");
          }
        }}
        className="flex items-center gap-2.5 py-1"
      >
        <span className="h-[15px] w-[15px] shrink-0 rounded-full border-[1.5px] border-dashed border-line" />
        <input
          value={adding}
          placeholder={t("addPart")}
          aria-label={t("addPart")}
          onChange={(e) => setAdding(e.target.value)}
          className="min-w-0 flex-1 bg-transparent text-[13px] outline-none placeholder:text-faint"
        />
      </form>
    </>
  );
}

function Row({
  part,
  onOpen,
  onComplete,
}: {
  part: Task;
  onOpen: (id: string) => void;
  onComplete: (id: string, title: string) => void;
}) {
  const open = part.status === "open";
  const steps = part.volume?.steps ? `${part.volume.steps_done ?? 0}/${part.volume.steps}` : "";

  return (
    <div className="flex items-center gap-2.5 py-1 text-[13px]">
      {open ? (
        <button
          type="button"
          aria-label={fill("completeIt", part.title)}
          title={fill("completeIt", part.title)}
          onClick={() => onComplete(part.id, part.title)}
          className="h-[15px] w-[15px] shrink-0 rounded-full border-[1.5px] border-faint hover:border-accent"
        />
      ) : (
        <span aria-hidden="true" className="w-[15px] shrink-0 text-center text-accent">
          {part.status === "dropped" ? "⨯" : "✓"}
        </span>
      )}
      <button
        type="button"
        onClick={() => onOpen(part.id)}
        className={`min-w-0 flex-1 truncate rounded-md text-left hover:text-accent ${
          open ? "" : "text-faint line-through"
        }`}
      >
        {part.title}
      </button>
      {part.resolved && open ? (
        <span className="shrink-0 text-[11.5px] text-hue-teal">◆ {t("agentBand")}</span>
      ) : (
        steps && <span className="shrink-0 text-[11.5px] text-faint">{steps}</span>
      )}
    </div>
  );
}
