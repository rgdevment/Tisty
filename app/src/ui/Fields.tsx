import { useEffect, useState } from "react";
import type { Change, List, Task } from "../core";
import { cadence, clockOf, whenLabel } from "../format";
import { t } from "../locales";
import { useEdge } from "./edge";
import Recall from "./Recall";
import When from "./When";

interface Props {
  task: Task;
  lists: List[];
  known: string[];
  onPatch: (change: Change) => void;
}

type Slot = "date" | "deadline" | "priority" | "list" | "tags" | "recall" | "repeat";

export default function Fields({ task, lists, known, onPatch }: Props) {
  const [open, setOpen] = useState<Slot | null>(null);
  const [until, setUntil] = useState(false);
  const list = lists.find((l) => l.id === task.list);
  const close = () => setOpen(null);
  const apply = (change: Change) => {
    onPatch(change);
    close();
  };

  return (
    <div className="mb-5 flex flex-wrap items-center gap-1.5">
      <Held
        slot="list"
        open={open}
        onOpen={setOpen}
        tint="bg-mark-list"
        empty={list === undefined}
        label={`@ ${list?.name ?? t("fieldList")}`}
      >
        <Sheet onClose={close}>
          {list && <Row onPick={() => apply({ inbox: true })}>{t("noList")}</Row>}
          {lists.map((one) => (
            <Row key={one.id} onPick={() => apply({ list: one.id })}>
              {one.name}
            </Row>
          ))}
          <Filing onName={(name) => apply({ listNamed: name })} />
        </Sheet>
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
            label={`${slot === "date" ? "☀" : "⚑"} ${spec ? whenLabel(spec) : t(fieldOf(slot))}`}
          >
            <Sheet roomy onClose={close}>
              <When
                value={spec?.at.slice(0, 10)}
                clock={spec?.has_time ? spec.at.slice(11, 16) : undefined}
                onPick={(at) => apply(slot === "date" ? { date: at } : { deadline: at })}
                onClear={() => apply(slot === "date" ? { noDate: true } : { noDeadline: true })}
                onClose={close}
              />
            </Sheet>
          </Held>
        );
      })}

      <Held
        slot="priority"
        open={open}
        onOpen={setOpen}
        tint="bg-mark-priority"
        empty={task.priority === 4}
        label={`! ${task.priority < 4 ? t(named(task.priority)) : t("fieldPriority")}`}
      >
        <Sheet onClose={close}>
          {([1, 2, 3, 4] as const).map((level) => (
            <Row key={level} onPick={() => apply({ priority: level })}>
              {level < 4 ? t(named(level)) : t("noPriority")}
            </Row>
          ))}
        </Sheet>
      </Held>

      {until && task.repeat && (
        <Sheet roomy onClose={() => setUntil(false)}>
          <When
            never
            confirm={t("endsOn")}
            value={task.repeat.until ?? undefined}
            onPick={(at) => {
              setUntil(false);
              apply({ repeat: { ...task.repeat!, until: at.slice(0, 10) } });
            }}
            onClear={() => setUntil(false)}
            onClose={() => setUntil(false)}
          />
        </Sheet>
      )}

      <Held
        slot="repeat"
        open={open}
        onOpen={setOpen}
        tint="bg-mark-repeat"
        empty={!task.repeat}
        label={`↻ ${task.repeat ? cadence(task.repeat) : t("fieldRepeat")}`}
      >
        <Sheet onClose={close}>
          {CADENCES.map(({ every, unit }) => (
            <Row
              key={`${every}${unit}`}
              onPick={() =>
                apply({
                  repeat: {
                    from: task.repeat?.from ?? (task.date ? "due" : "done"),
                    each: { every, unit },
                  },
                })
              }
            >
              {cadence({ from: "due", each: { every, unit } })}
            </Row>
          ))}
          {task.repeat && (
            <>
              <Row onPick={() => setUntil(true)}>{t("endsOn")}</Row>
              {task.repeat.until && (
                <Row onPick={() => apply({ repeat: { ...task.repeat!, until: null } })}>
                  {t("noEnd")}
                </Row>
              )}
              <Row onPick={() => apply({ noRepeat: true })}>{t("endRepeat")}</Row>
            </>
          )}
        </Sheet>
      </Held>

      {task.tags?.map((tag) => (
        <Worn
          key={tag}
          tint="bg-mark-tag"
          label={`# ${tag}`}
          onDrop={() => onPatch({ untag: tag })}
        />
      ))}

      <Held
        slot="tags"
        open={open}
        onOpen={setOpen}
        tint="bg-mark-tag"
        empty
        label={`# ${t("fieldTag")}`}
      >
        <Sheet onClose={close}>
          <Naming
            known={known}
            taken={task.tags ?? []}
            onName={(name) => apply({ addTag: name })}
          />
        </Sheet>
      </Held>

      {task.reminders?.map((at) => (
        <Worn
          key={at.at}
          tint={task.repeat ? "bg-mark-repeat" : "bg-hover"}
          label={
            task.repeat
              ? `⏰↻ ${cadence(task.repeat)}${clockOf(at) ? ` · ${clockOf(at)}` : ""}`
              : `⏰ ${whenLabel(at)}`
          }
          onDrop={() => onPatch({ unremind: civil(at.at) })}
        />
      ))}

      <Held
        slot="recall"
        open={open}
        onOpen={setOpen}
        tint="bg-hover"
        empty
        label={`⏰ ${t("reminder")}`}
      >
        <Sheet roomy onClose={close}>
          <Recall
            on={task.date ?? task.deadline}
            due={!task.date && !!task.deadline}
            taken={(task.reminders ?? []).map((one) => civil(one.at))}
            onAdd={(at) => apply({ remind: at })}
            onClose={close}
          />
        </Sheet>
      </Held>
    </div>
  );
}

interface HeldProps {
  slot: Slot;
  open: Slot | null;
  tint: string;
  empty: boolean;
  label: string;
  onOpen: (slot: Slot | null) => void;
  children: React.ReactNode;
}

function Held({ slot, open, tint, empty, label, onOpen, children }: HeldProps) {
  return (
    <span className="relative inline-flex">
      <button
        type="button"
        aria-expanded={open === slot}
        aria-haspopup="menu"
        onClick={() => onOpen(open === slot ? null : slot)}
        className={`inline-flex items-center gap-1.5 rounded-md px-2.5 py-1 text-xs ${
          empty ? "border border-dashed border-line text-faint" : `${tint} text-ink`
        }`}
      >
        {empty && <span className="text-[13px] leading-none">＋</span>}
        {label}
      </button>
      {open === slot && children}
    </span>
  );
}

function Worn({ label, tint, onDrop }: { label: string; tint: string; onDrop: () => void }) {
  return (
    <span className={`inline-flex items-center gap-1 rounded-md py-1 pr-1 pl-2.5 text-xs ${tint}`}>
      {label}
      <button
        type="button"
        aria-label={`${t("remove")} ${label}`}
        onClick={onDrop}
        className="flex h-4 w-4 items-center justify-center rounded text-faint hover:bg-line hover:text-ink"
      >
        ×
      </button>
    </span>
  );
}

function Sheet({
  children,
  roomy,
  onClose,
}: {
  children: React.ReactNode;
  roomy?: boolean;
  onClose: () => void;
}) {
  const { box, away } = useEdge<HTMLDivElement>();

  useEffect(() => {
    const at = box.current;
    const came = document.activeElement as HTMLElement | null;
    const wants = at?.querySelector<HTMLElement>("input, textarea");
    if (wants) {
      wants.focus();
    } else if (!at?.contains(document.activeElement)) {
      at?.querySelector<HTMLElement>("button")?.focus();
    }
    return () => came?.focus?.();
  }, [box]);

  return (
    <>
      <span className="fixed inset-0 z-10" onClick={onClose} />
      <div
        ref={box}
        onKeyDown={(event) => {
          if (event.key !== "Escape") return;
          event.preventDefault();
          event.stopPropagation();
          onClose();
        }}
        className={`absolute z-20 rounded-[10px] border border-line bg-bg p-[5px] text-[12.5px] shadow-lift ${
          away.right ? "right-0" : "left-0"
        } ${away.up ? "bottom-7" : "top-7"} ${roomy ? "w-[248px]" : "max-h-64 w-56 overflow-auto"}`}
      >
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

function Filing({ onName }: { onName: (name: string) => void }) {
  const [text, setText] = useState("");

  return (
    <form
      onSubmit={(e) => {
        e.preventDefault();
        const name = text.trim();
        if (name) onName(name);
      }}
    >
      <input
        value={text}
        placeholder={t("namedList")}
        aria-label={t("namedList")}
        onChange={(e) => setText(e.target.value)}
        className="w-full rounded-md bg-hover px-2.5 py-1.5 outline-none placeholder:text-faint"
      />
    </form>
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
  const typed = text.trim().replace(/^#/, "").toLowerCase();
  const offered = known.filter((one) => !taken.includes(one) && one.startsWith(typed));

  return (
    <>
      <form
        onSubmit={(e) => {
          e.preventDefault();
          if (typed) onName(typed);
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

const civil = (at: string): string => `${at.slice(0, 16).replace(" ", "T")}:00`;

const named = (level: number): "high" | "medium" | "low" =>
  level === 1 ? "high" : level === 2 ? "medium" : "low";

const CADENCES = [
  { every: 1, unit: "day" },
  { every: 1, unit: "week" },
  { every: 2, unit: "week" },
  { every: 1, unit: "month" },
  { every: 1, unit: "year" },
] as const;

const fieldOf = (slot: "date" | "deadline"): "fieldDate" | "fieldDeadline" =>
  slot === "date" ? "fieldDate" : "fieldDeadline";
