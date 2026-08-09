import { useState } from "react";
import type { Change, List, Task } from "../core";
import { whenLabel } from "../format";
import { t } from "../locales";
import Calendar from "./Calendar";

interface Props {
  task: Task;
  lists: List[];
  known: string[];
  onPatch: (change: Change) => void;
}

type Slot = "date" | "deadline" | "priority" | "list" | "tags";

export default function Fields({ task, lists, known, onPatch }: Props) {
  const [open, setOpen] = useState<Slot | null>(null);
  const list = lists.find((l) => l.id === task.list);
  const close = () => setOpen(null);
  const apply = (change: Change) => {
    onPatch(change);
    close();
  };

  return (
    <div className="mb-5 flex flex-wrap items-center gap-1.5">
      <Held slot="list" open={open} onOpen={setOpen} tint="bg-mark-list" empty={list === undefined}>
        <span>@ {list?.name ?? t("fieldList")}</span>
        {open === "list" && (
          <Sheet onClose={close}>
            {list && <Row onPick={() => apply({ inbox: true })}>{t("noList")}</Row>}
            {lists.map((one) => (
              <Row key={one.id} onPick={() => apply({ list: one.id })}>
                {one.name}
              </Row>
            ))}
          </Sheet>
        )}
      </Held>

      {(["date", "deadline"] as const).map((slot) => {
        const spec = task[slot];
        return (
          <Held
            key={slot}
            slot={slot}
            open={open}
            onOpen={setOpen}
            tint={slot === "date" ? "bg-mark-date" : "bg-mark-deadline"}
            empty={!spec}
          >
            <span>
              {slot === "date" ? "☀" : "⚑"} {spec ? whenLabel(spec) : t(fieldOf(slot))}
            </span>
            {open === slot && (
              <Calendar
                value={spec?.at.slice(0, 10)}
                onPick={(iso) => apply(slot === "date" ? { date: iso } : { deadline: iso })}
                onClear={() => apply(slot === "date" ? { noDate: true } : { noDeadline: true })}
                onClose={close}
              />
            )}
          </Held>
        );
      })}

      <Held
        slot="priority"
        open={open}
        onOpen={setOpen}
        tint="bg-mark-priority"
        empty={task.priority === 4}
      >
        <span>! {task.priority < 4 ? t(named(task.priority)) : t("fieldPriority")}</span>
        {open === "priority" && (
          <Sheet onClose={close}>
            {([1, 2, 3, 4] as const).map((level) => (
              <Row key={level} onPick={() => apply({ priority: level })}>
                {level < 4 ? t(named(level)) : t("noPriority")}
              </Row>
            ))}
          </Sheet>
        )}
      </Held>

      {task.tags?.map((tag) => (
        <span
          key={tag}
          className="inline-flex items-center gap-1 rounded-md bg-mark-tag py-1 pr-1 pl-2.5 text-xs"
        >
          # {tag}
          <button
            type="button"
            aria-label={`${t("remove")} ${tag}`}
            onClick={() => onPatch({ tags: (task.tags ?? []).filter((one) => one !== tag) })}
            className="flex h-4 w-4 items-center justify-center rounded text-faint hover:bg-line hover:text-ink"
          >
            ×
          </button>
        </span>
      ))}

      <Held slot="tags" open={open} onOpen={setOpen} tint="bg-mark-tag" empty>
        <span># {t("fieldTag")}</span>
        {open === "tags" && (
          <Sheet onClose={close}>
            <Naming
              known={known}
              taken={task.tags ?? []}
              onName={(name) => apply({ tags: [...(task.tags ?? []), name] })}
            />
          </Sheet>
        )}
      </Held>
    </div>
  );
}

interface HeldProps {
  slot: Slot;
  open: Slot | null;
  tint: string;
  empty: boolean;
  onOpen: (slot: Slot | null) => void;
  children: React.ReactNode;
}

function Held({ slot, open, tint, empty, onOpen, children }: HeldProps) {
  return (
    <span className="relative inline-flex">
      <button
        type="button"
        onClick={() => onOpen(open === slot ? null : slot)}
        className={`inline-flex items-center gap-1.5 rounded-md px-2.5 py-1 text-xs ${
          empty ? "border border-dashed border-line text-faint" : `${tint} text-ink`
        }`}
      >
        {children}
      </button>
    </span>
  );
}

function Sheet({ children, onClose }: { children: React.ReactNode; onClose: () => void }) {
  return (
    <>
      <span className="fixed inset-0 z-10" onClick={onClose} />
      <div className="absolute top-7 left-0 z-20 max-h-64 w-56 overflow-auto rounded-[10px] border border-line bg-bg p-[5px] text-[12.5px] shadow-lg">
        {children}
      </div>
    </>
  );
}

function Row({ children, onPick }: { children: React.ReactNode; onPick: () => void }) {
  return (
    <button
      type="button"
      onClick={onPick}
      className="block w-full rounded-md px-2.5 py-1.5 text-left text-ink hover:bg-hover"
    >
      {children}
    </button>
  );
}

function Naming({
  known,
  taken,
  onName,
}: {
  known: string[];
  taken: string[];
  onName: (name: string) => void;
}) {
  const [text, setText] = useState("");
  const name = text.trim().replace(/^#/, "").toLowerCase();
  const offered = known.filter((one) => !taken.includes(one) && one.startsWith(name));

  return (
    <>
      <form
        onSubmit={(e) => {
          e.preventDefault();
          if (name && !taken.includes(name)) onName(name);
        }}
      >
        <input
          autoFocus
          value={text}
          placeholder={t("fieldTag")}
          aria-label={t("fieldTag")}
          onChange={(e) => setText(e.target.value)}
          className="mb-1 w-full rounded-md bg-hover px-2.5 py-1.5 outline-none placeholder:text-faint"
        />
      </form>
      {offered.length === 0 ? (
        <p className="px-2.5 py-1.5 text-faint">{t("noTagsHere")}</p>
      ) : (
        offered.map((one) => (
          <Row key={one} onPick={() => onName(one)}>
            # {one}
          </Row>
        ))
      )}
    </>
  );
}

const named = (level: number): "high" | "medium" | "low" =>
  level === 1 ? "high" : level === 2 ? "medium" : "low";

const fieldOf = (slot: "date" | "deadline"): "fieldDate" | "fieldDeadline" =>
  slot === "date" ? "fieldDate" : "fieldDeadline";
