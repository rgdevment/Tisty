import { useState } from "react";
import type { DateSpec, Edits, Parsed } from "../core";
import { whenLabel } from "../format";
import { t } from "../locales";
import Calendar from "./Calendar";

type Slot = "date" | "deadline" | "list" | "priority" | { tag: string };

interface Chip {
  slot: Slot;
  glyph: string;
  value: string;
  tint: string;
  guessed: boolean;
  /** The day it stands on, so the calendar opens on its month and not on this one. */
  on?: string;
  /** Seen but not taken; its button applies it instead of removing it. */
  offer?: Edits;
}

interface Props {
  seen: Parsed;
  edits: Edits;
  onEdit: (edits: Edits) => void;
  empty: React.ReactNode;
}

export default function Chips({ seen, edits, onEdit, empty }: Props) {
  const [open, setOpen] = useState<string | null>(null);
  const chips = shown(seen, edits);

  if (chips.length === 0) return empty;

  return (
    <div className="flex flex-wrap items-center gap-1.5 text-[12px]">
      {chips.map((chip) => {
        const key = name(chip.slot);
        const pickable = chip.on !== undefined && chip.offer === undefined;
        return (
          <span
            key={key}
            title={chip.offer ? t("offered") : chip.guessed ? t("guessed") : undefined}
            className={`relative inline-flex items-center rounded-md ${
              chip.offer ? "border border-dashed border-line py-px pr-0.5 pl-1.5" : `${chip.tint} py-0.5 pr-0.5 pl-1.5`
            }`}
          >
            <span className="mr-1 w-3 text-center text-[11px] text-soft">{chip.glyph}</span>
            {pickable ? (
              <button
                type="button"
                onClick={() => setOpen(open === key ? null : key)}
                className={`text-ink ${chip.guessed ? "underline decoration-dotted underline-offset-[3px]" : ""}`}
              >
                {chip.value}
              </button>
            ) : (
              <span className={chip.offer ? "text-soft" : "text-ink"}>{chip.value}</span>
            )}
            <button
              type="button"
              aria-label={`${chip.offer ? t("accept") : t("remove")} ${chip.value}`}
              onClick={() => onEdit(chip.offer ?? without(edits, chip.slot))}
              className={`ml-1 flex h-4 w-4 items-center justify-center rounded ${
                chip.offer
                  ? "text-accent hover:bg-accent-soft"
                  : "text-faint hover:bg-line hover:text-ink"
              }`}
            >
              {chip.offer ? "＋" : "×"}
            </button>
            {open === key && (
              <Calendar
                value={chip.on}
                onPick={(iso) => {
                  onEdit({ ...edits, ...set(chip.slot, iso) });
                  setOpen(null);
                }}
                onClear={() => {
                  onEdit(without(edits, chip.slot));
                  setOpen(null);
                }}
                onClose={() => setOpen(null)}
              />
            )}
          </span>
        );
      })}
    </div>
  );
}

const name = (slot: Slot): string => (typeof slot === "string" ? slot : `tag:${slot.tag}`);

/** Same glyphs as the `/` menu and the same tints as the marks in the text. */
function shown(seen: Parsed, edits: Edits): Chip[] {
  const chips: Chip[] = [];
  const guessed = seen.spans.some(
    (s) => s.certainty === "assumed" && (s.mark === "date" || s.mark === "deadline"),
  );

  const when = (slot: "date" | "deadline", glyph: string, dropped?: boolean, picked?: string) => {
    const spec = picked ? undefined : seen[slot];
    if (!picked && (dropped || !spec)) return;
    chips.push({
      slot,
      glyph,
      value: picked ? plainly(picked) : whenLabel(spec as DateSpec),
      tint: "bg-mark-date",
      guessed: !picked && guessed,
      on: picked ?? (spec as DateSpec).at.slice(0, 10),
    });
  };

  when("date", "☀", edits.noDate, edits.date);
  when("deadline", "⚑", edits.noDeadline, edits.deadline);

  if (seen.list && !edits.noList) {
    chips.push({
      slot: "list",
      glyph: "#",
      value: seen.list,
      tint: "bg-mark-list",
      guessed: false,
    });
  }
  for (const tag of seen.tags) {
    if (edits.noTags?.includes(tag)) continue;
    chips.push({
      slot: { tag },
      glyph: "@",
      value: tag,
      tint: "bg-mark-tag",
      guessed: false,
    });
  }

  const level = seen.priority;
  if (level && level < 4 && !edits.noPriority) {
    chips.push({
      slot: "priority",
      glyph: "!",
      value: t(level === 1 ? "urgent" : level === 2 ? "high" : "medium"),
      tint: "bg-mark-priority",
      guessed: false,
    });
  }

  // Only while still an offer; once accepted it becomes a date chip above.
  if (!edits.date && !edits.deadline) {
    for (const offer of seen.offers) {
      chips.push({
        slot: "date",
        glyph: "☀",
        value: whenLabel(offer.date),
        tint: "bg-mark-date",
        guessed: false,
        offer: {
          ...edits,
          noDate: false,
          date: offer.date.at.slice(0, 10),
          takeOffer: true,
        },
      });
    }
  }
  return chips;
}

function without(edits: Edits, slot: Slot): Edits {
  if (typeof slot !== "string") {
    return { ...edits, noTags: [...(edits.noTags ?? []), slot.tag] };
  }
  switch (slot) {
    case "date":
      return { ...edits, noDate: true, date: undefined };
    case "deadline":
      return { ...edits, noDeadline: true, deadline: undefined };
    case "list":
      return { ...edits, noList: true };
    case "priority":
      return { ...edits, noPriority: true };
  }
}

const set = (slot: Slot, iso: string): Edits =>
  slot === "deadline" ? { noDeadline: false, deadline: iso } : { noDate: false, date: iso };

/** A picked day has no clock and no zone, so it is read where the user is. */
const plainly = (iso: string): string =>
  whenLabel({ at: `${iso}T00:00:00`, tz: "", floating: true, has_time: false });
