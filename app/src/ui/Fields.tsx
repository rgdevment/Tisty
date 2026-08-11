import { useEffect, useLayoutEffect, useRef, useState } from "react";
import type { Change, List, Task } from "../core";
import { cadence, whenLabel } from "../format";
import { t } from "../locales";
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
                apply({ repeat: { from: task.repeat?.from ?? (task.date ? "due" : "done"), each: { every, unit } } })
              }
            >
              {cadence({ from: "due", each: { every, unit } })}
            </Row>
          ))}
          {task.repeat && <Row onPick={() => apply({ noRepeat: true })}>{t("endRepeat")}</Row>}
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
          <Naming known={known} taken={task.tags ?? []} onName={(name) => apply({ addTag: name })} />
        </Sheet>
      </Held>

      {task.reminders?.map((at) => (
        <Worn
          key={at.at}
          tint="bg-hover"
          label={`⏰ ${whenLabel(at)}`}
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
            on={task.date}
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
  const box = useRef<HTMLDivElement>(null);
  const [away, setAway] = useState({ right: false, up: false });

  // Anchored bottom-left of the chip with no regard for the window edge, half
  // of the calendar fell outside the three-column layout, where the detail is
  // a narrow strip. Measured after the first paint and flipped if it does not
  // fit — `useLayoutEffect` so nobody sees the wrong side.
  useLayoutEffect(() => {
    const at = box.current?.getBoundingClientRect();
    if (!at) return;
    setAway({
      right: at.right > window.innerWidth - EDGE,
      up: at.bottom > window.innerHeight - EDGE,
    });
  }, []);

  useEffect(() => {
    const came = document.activeElement as HTMLElement | null;
    // Only if nothing inside asked for it first: the tag sheet autofocuses its
    // input, and grabbing the first button put the focus on a suggestion —
    // where Space applied a tag nobody chose. In the date sheet the first
    // button is «previous month», which is nobody's destination either.
    const wants = box.current?.querySelector<HTMLElement>("input, textarea");
    if (wants) {
      wants.focus();
    } else if (!box.current?.contains(document.activeElement)) {
      box.current?.querySelector<HTMLElement>("button")?.focus();
    }
    return () => came?.focus?.();
  }, []);

  return (
    <>
      {/* The catcher is for the mouse; Escape is what closes this for anyone
          who never touched it. */}
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
        } ${away.up ? "bottom-7" : "top-7"} ${
          roomy ? "w-[248px]" : "max-h-64 w-56 overflow-auto"
        }`}
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

/// Breathing room against the window edge, so a flipped sheet does not sit
/// flush against it.
const EDGE = 8;
